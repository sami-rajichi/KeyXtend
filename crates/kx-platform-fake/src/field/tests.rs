//! The fake field watch, held to the port: told focus changes, words cut before the cursor, closed boxes never read,
//! and failures on cue.

use super::*;
use std::thread::{self, ThreadId};

/// Test values, whose read cap holds every scripted text; the real ones come from the predict module's settings.
const CONFIG: FieldConfig = FieldConfig {
    read_ms: 1,
    read_chars: 64,
};

/// What a box holds before its cursor, ending mid-word.
const TEXT: &str = "I really like ap";

/// What each test watch heard, with the thread it heard it on.
type Heard = Arc<Mutex<Vec<(FieldChange, ThreadId)>>>;

/// A started watch whose `tell` records each change.
fn watched(fake: &FakeFieldWatch) -> (Box<dyn FieldRun>, Heard) {
    watched_with(fake, CONFIG)
}

/// Like `watched`, with `cfg`.
fn watched_with(fake: &FakeFieldWatch, cfg: FieldConfig) -> (Box<dyn FieldRun>, Heard) {
    let heard = Heard::default();
    let log = Arc::clone(&heard);
    let tell = move |change| log.lock().unwrap().push((change, thread::current().id()));
    let run = fake.watch(cfg, Box::new(tell));
    (run.ok().unwrap(), heard)
}

/// `CONFIG` with a read cap of `read_chars`.
fn capped(read_chars: u16) -> FieldConfig {
    FieldConfig {
        read_chars,
        ..CONFIG
    }
}

/// The text a run reads before the cursor, unwrapped.
fn read(run: &mut dyn FieldRun, max_words: u8) -> Result<Option<String>, FieldError> {
    run.before_cursor(max_words)
        .map(|text| text.map(Redacted::into_inner))
}

/// Asks the run whether the text ends with a word.
fn ends(run: &mut dyn FieldRun) -> Option<bool> {
    run.ends_with(&Redacted::new(String::from("ap")))
}

#[test]
fn each_focus_tells_the_watch_at_once_with_a_fresh_box() {
    let fake = FakeFieldWatch::new();
    let (run, heard) = watched(&fake);
    assert_eq!(run.kind(), FieldKind::None, "no box has focus yet");
    assert_eq!(fake.box_id(), 0);
    fake.focus(FieldKind::Text);
    let first = fake.box_id();
    fake.focus(FieldKind::Password);
    let second = fake.box_id();
    assert_ne!(first, second);
    let me = thread::current().id();
    let told = [
        (
            FieldChange {
                kind: FieldKind::Text,
                box_id: first,
            },
            me,
        ),
        (
            FieldChange {
                kind: FieldKind::Password,
                box_id: second,
            },
            me,
        ),
    ];
    assert_eq!(*heard.lock().unwrap(), told);
    assert_eq!(run.kind(), FieldKind::Password);
}

#[test]
fn words_before_the_cursor_are_cut_to_the_last_n() {
    let fake = FakeFieldWatch::new();
    let (mut run, _) = watched(&fake);
    fake.focus(FieldKind::Text);
    fake.set_before(Some(TEXT));
    assert_eq!(read(run.as_mut(), 2), Ok(Some("like ap".to_owned())));
    assert_eq!(read(run.as_mut(), 9), Ok(Some(TEXT.to_owned())));
    assert_eq!(read(run.as_mut(), 0), Ok(Some(String::new())));
    fake.set_before(Some("  I like "));
    assert_eq!(
        read(run.as_mut(), 1),
        Ok(Some("like ".to_owned())),
        "the space after the word stays"
    );
    assert_eq!(read(run.as_mut(), 5), Ok(Some("I like ".to_owned())));
    fake.set_before(Some("أنا أحب القراءة"));
    assert_eq!(read(run.as_mut(), 2), Ok(Some("أحب القراءة".to_owned())));
}

#[test]
fn a_text_past_the_read_cap_keeps_only_the_whole_words_inside_it() {
    let fake = FakeFieldWatch::new();
    fake.focus(FieldKind::Text);
    fake.set_before(Some(TEXT));
    fake.set_ends(Some(true));
    let read_with = |chars, max_words| {
        let (mut run, _) = watched_with(&fake, capped(chars));
        read(run.as_mut(), max_words)
    };
    assert_eq!(
        read_with(6, 2),
        Ok(Some("ap".to_owned())),
        "a split word goes"
    );
    assert_eq!(
        read_with(8, 2),
        Ok(Some("like ap".to_owned())),
        "a cut between words"
    );
    let whole = u16::try_from(TEXT.chars().count()).unwrap();
    assert_eq!(read_with(whole, 9), Ok(Some(TEXT.to_owned())));
    fake.set_before(Some("أنا أحب القراءة"));
    assert_eq!(read_with(9, 2), Ok(Some("القراءة".to_owned())));
    let (mut short, _) = watched_with(&fake, capped(1));
    assert_eq!(
        ends(short.as_mut()),
        None,
        "the asked end is longer than the cap"
    );
}

#[test]
fn a_password_unknown_or_empty_focus_reads_nothing_and_is_still_counted() {
    let fake = FakeFieldWatch::new();
    let (mut run, _) = watched(&fake);
    fake.set_before(Some(TEXT));
    fake.set_ends(Some(true));
    for kind in [FieldKind::Password, FieldKind::Unknown, FieldKind::None] {
        fake.focus(kind);
        assert_eq!(read(run.as_mut(), 3), Ok(None), "{kind:?} is never read");
        assert_eq!(ends(run.as_mut()), None, "{kind:?} is never read");
    }
    assert_eq!((fake.before_calls(), fake.ends_calls()), (3, 3));
    fake.focus(FieldKind::Text);
    assert_eq!(read(run.as_mut(), 1), Ok(Some("ap".to_owned())));
    assert_eq!(ends(run.as_mut()), Some(true));
}

#[test]
fn a_text_box_answers_as_scripted_and_none_when_the_app_does_not_tell() {
    let fake = FakeFieldWatch::new();
    let (mut run, _) = watched(&fake);
    fake.focus(FieldKind::Text);
    assert_eq!(read(run.as_mut(), 3), Ok(None), "nothing scripted");
    assert_eq!(ends(run.as_mut()), None, "nothing scripted");
    fake.set_ends(Some(false));
    assert_eq!(ends(run.as_mut()), Some(false));
    fake.set_before(Some(TEXT));
    fake.set_before(None);
    assert_eq!(read(run.as_mut(), 3), Ok(None));
}

#[test]
fn failed_reads_last_until_focus_moves_to_a_new_box() {
    let fake = FakeFieldWatch::new();
    let (mut run, _) = watched(&fake);
    fake.focus(FieldKind::Text);
    fake.set_before(Some(TEXT));
    fake.set_ends(Some(true));
    fake.fail_reads(FieldError::TimedOut);
    assert_eq!(read(run.as_mut(), 1), Err(FieldError::TimedOut));
    assert_eq!(read(run.as_mut(), 1), Err(FieldError::TimedOut));
    assert_eq!(ends(run.as_mut()), None);
    assert_eq!(
        (fake.before_calls(), fake.ends_calls()),
        (2, 1),
        "failed reads count too"
    );
    fake.focus(FieldKind::Text);
    assert_eq!(read(run.as_mut(), 1), Ok(Some("ap".to_owned())));
    assert_eq!(ends(run.as_mut()), Some(true));
}

#[test]
fn a_failed_watch_fails_once_then_watches_with_its_config() {
    let fake = FakeFieldWatch::new();
    fake.fail_watch(FieldError::Unavailable);
    let failed = fake.watch(CONFIG, Box::new(|_| {}));
    assert_eq!(failed.err(), Some(FieldError::Unavailable));
    assert!(!fake.watching());
    assert_eq!((fake.watches(), fake.config()), (0, None));
    let (_run, _) = watched(&fake);
    assert!(fake.watching());
    assert_eq!((fake.watches(), fake.config()), (1, Some(CONFIG)));
}

#[test]
fn dropping_the_run_stops_the_watch_and_its_tells() {
    let fake = FakeFieldWatch::new();
    let (run, heard) = watched(&fake);
    drop(run);
    assert!(!fake.watching());
    fake.focus(FieldKind::Text);
    assert!(
        heard.lock().unwrap().is_empty(),
        "a stopped watch tells nothing"
    );
    assert_eq!(fake.config(), Some(CONFIG), "kept after the stop");
}

#[test]
fn clones_share_the_script_and_serve_as_the_port_from_another_thread() {
    let fake = FakeFieldWatch::new();
    let port: Arc<dyn FieldWatch> = Arc::new(fake.clone());
    let run = thread::spawn(move || port.watch(CONFIG, Box::new(|_| {})).ok());
    let mut run = run.join().unwrap().unwrap();
    assert!(fake.watching());
    fake.focus(FieldKind::Text);
    fake.set_before(Some(TEXT));
    assert_eq!(run.kind(), FieldKind::Text);
    assert_eq!(read(run.as_mut(), 1), Ok(Some("ap".to_owned())));
}

#[test]
fn debug_never_prints_the_scripted_text() {
    let fake = FakeFieldWatch::new();
    fake.set_before(Some(TEXT));
    assert!(!format!("{fake:?}").contains(TEXT));
}

#[test]
fn it_keeps_the_last_text_ends_with_was_asked_and_never_shows_it() {
    let fake = FakeFieldWatch::new();
    assert_eq!(fake.last_ends(), None, "nothing asked yet");
    let (mut run, _) = watched(&fake);
    ends(run.as_mut());
    let asked = fake.last_ends().map(Redacted::into_inner);
    assert_eq!(asked.as_deref(), Some("ap"), "asked even of no box");
    run.ends_with(&Redacted::new(String::from("ق")));
    let asked = fake.last_ends().map(Redacted::into_inner);
    assert_eq!(asked.as_deref(), Some("ق"), "the latest wins");
    assert!(!format!("{fake:?}").contains('ق'));
}
