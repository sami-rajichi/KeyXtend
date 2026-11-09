//! The fake selection watch, held to the port: told changes with fresh boxes, scripted answers, reads cut in whole
//! characters, closed boxes never read, our own focus refused, failures on cue, and every call counted.

use super::*;
use kx_module_api::Redacted;
use kx_module_api::ports::Rect;
use std::thread::{self, ThreadId};

mod ids;
mod ours;
mod watch;

/// Test values; the real ones come from the selection module's settings.
const CONFIG: SelConfig = SelConfig {
    settle_ms: 1,
    read_ms: 1,
    looks_per_s: 1,
};

/// A one-line selection's box, and the caret at its end.
const LINE: Rect = Rect {
    x: 10,
    y: 20,
    w: 300,
    h: 18,
};
const CARET: Rect = Rect {
    x: 310,
    y: 20,
    w: 2,
    h: 18,
};
/// The screen that holds them.
const SCREEN: Screen = Screen {
    rect: Rect {
        x: 0,
        y: 0,
        w: 1920,
        h: 1040,
    },
    dpi: 144,
    primary: true,
};

/// A selected text, in English and in Arabic ("hello world").
const TEXT: &str = "Hello world";
const ARABIC: &str = "مرحبا بالعالم";

/// For each tell of a test watch: the thread it ran on, and whether the fake's log was unlocked then.
type Heard = Arc<Mutex<Vec<(ThreadId, bool)>>>;

/// A started watch whose `tell` records each change; it only tries the fake's log, so a regression fails, never hangs.
fn watched(fake: &FakeSelectionWatch) -> (Box<dyn SelRun>, Heard) {
    let heard = Heard::default();
    let log = Arc::clone(&heard);
    let shared = Arc::clone(&fake.shared);
    let tell = move || {
        let unlocked = shared.log.try_lock().is_ok();
        log.lock().unwrap().push((thread::current().id(), unlocked));
    };
    let run = fake.watch(CONFIG, Box::new(tell));
    (run.ok().unwrap(), heard)
}

/// An editable box holding a one-line selection; a made-up id the fake replaces with its own.
fn selected() -> SelSpot {
    SelSpot {
        box_id: 77,
        kind: SelKind::Editable,
        selected: true,
        first: Some(LINE),
        last: Some(LINE),
        caret: Some(CARET),
        screen: SCREEN,
    }
}

/// The same box at a bare caret.
fn caret() -> SelSpot {
    SelSpot {
        selected: false,
        first: None,
        last: None,
        ..selected()
    }
}

/// A box of `kind` holding the selection.
fn of(kind: SelKind) -> SelSpot {
    SelSpot { kind, ..selected() }
}

/// `spot` under the id `box_id`.
fn under(box_id: u64, spot: SelSpot) -> SelSpot {
    SelSpot { box_id, ..spot }
}

/// The text a run reads from the box the fake has focused, unwrapped.
fn read(
    fake: &FakeSelectionWatch,
    run: &mut dyn SelRun,
    max_chars: u16,
) -> Result<String, SelError> {
    let id = fake.spot().box_id;
    run.read(id, max_chars).map(Redacted::into_inner)
}

#[test]
fn it_starts_on_no_box_which_is_closed_and_holds_nothing() {
    let fake = FakeSelectionWatch::new();
    let (mut run, heard) = watched(&fake);
    let none = SelSpot {
        box_id: 0,
        kind: SelKind::Closed,
        selected: false,
        first: None,
        last: None,
        caret: None,
        screen: Screen::default(),
    };
    assert_eq!(run.look(), Ok(none));
    assert!(!run.still(0), "no selection");
    assert_eq!(read(&fake, run.as_mut(), 9), Err(SelError::Closed));
    assert!(!run.clip_holds(), "an empty clipboard");
    assert!(heard.lock().unwrap().is_empty());
}

#[test]
fn each_change_tells_at_once_on_this_thread_with_the_log_unlocked_and_a_new_box_has_a_fresh_id() {
    let fake = FakeSelectionWatch::new();
    let (mut run, heard) = watched(&fake);
    fake.focus(selected());
    let first = fake.spot().box_id;
    assert!(
        first != 0 && first != selected().box_id,
        "the fake gives the id"
    );
    assert_eq!(run.look(), Ok(under(first, selected())));
    fake.change(caret());
    assert_eq!(run.look(), Ok(under(first, caret())), "the same box");
    fake.focus(selected());
    let second = fake.spot().box_id;
    assert!(second != first && second != selected().box_id);
    let told = (thread::current().id(), true);
    assert_eq!(
        *heard.lock().unwrap(),
        [told; 3],
        "each tell finds the log free"
    );
}

#[test]
fn a_change_unheard_tells_nothing_and_the_next_look_finds_it() {
    let fake = FakeSelectionWatch::new();
    let (mut run, heard) = watched(&fake);
    fake.focus(selected());
    let id = fake.spot().box_id;
    fake.change_unheard(caret());
    assert_eq!(heard.lock().unwrap().len(), 1, "only the focus told");
    assert_eq!(run.look(), Ok(under(id, caret())), "its own id stays");
    assert!(!run.still(id), "the selection is gone");
}

#[test]
fn still_holds_only_for_the_focused_box_while_it_holds_a_selection_unless_scripted() {
    let fake = FakeSelectionWatch::new();
    let (mut run, _) = watched(&fake);
    fake.focus(selected());
    let id = fake.spot().box_id;
    assert!(run.still(id));
    assert!(!run.still(id + 1), "another box");
    fake.change(caret());
    assert!(!run.still(id), "a bare caret");
    fake.set_still(Some(true));
    assert!(run.still(id + 1), "as scripted");
    fake.change(selected());
    fake.set_still(Some(false));
    assert!(!run.still(id), "as scripted");
    fake.set_still(None);
    assert!(run.still(id), "the rule again");
}

#[test]
fn a_read_gives_the_selection_from_its_start_cut_in_whole_characters() {
    let fake = FakeSelectionWatch::new();
    let (mut run, _) = watched(&fake);
    fake.focus(selected());
    fake.set_text(TEXT);
    assert_eq!(read(&fake, run.as_mut(), 5), Ok("Hello".to_owned()));
    assert_eq!(read(&fake, run.as_mut(), u16::MAX), Ok(TEXT.to_owned()));
    assert_eq!(read(&fake, run.as_mut(), 0), Ok(String::new()));
    fake.set_text(ARABIC);
    let word = Ok("مرحبا".to_owned());
    assert_eq!(
        read(&fake, run.as_mut(), 5),
        word,
        "five letters of two bytes each"
    );
    fake.change(caret());
    assert_eq!(
        read(&fake, run.as_mut(), 5),
        Ok(String::new()),
        "a bare caret"
    );
}

#[test]
fn a_closed_box_is_never_read_and_its_read_is_not_counted() {
    let fake = FakeSelectionWatch::new();
    let (mut run, _) = watched(&fake);
    fake.set_text(TEXT);
    fake.focus(of(SelKind::Closed));
    assert_eq!(read(&fake, run.as_mut(), 9), Err(SelError::Closed));
    fake.fail_reads(SelError::TimedOut);
    assert_eq!(
        read(&fake, run.as_mut(), 9),
        Err(SelError::Closed),
        "never reached"
    );
    assert_eq!(fake.calls().reads, 0);
    fake.heal();
    for kind in [SelKind::Editable, SelKind::ReadOnly, SelKind::Terminal] {
        fake.focus(of(kind));
        assert_eq!(
            read(&fake, run.as_mut(), 5),
            Ok("Hello".to_owned()),
            "{kind:?}"
        );
    }
    assert_eq!(fake.calls().reads, 3);
}

#[test]
fn failed_looks_and_reads_last_across_boxes_until_healed_and_are_counted() {
    let fake = FakeSelectionWatch::new();
    let (mut run, _) = watched(&fake);
    fake.focus(selected());
    fake.set_text(TEXT);
    fake.fail_looks(SelError::TimedOut);
    fake.fail_reads(SelError::Refused);
    assert_eq!(run.look(), Err(SelError::TimedOut));
    fake.focus(selected());
    assert_eq!(run.look(), Err(SelError::TimedOut), "a new box too");
    assert_eq!(read(&fake, run.as_mut(), 5), Err(SelError::Refused));
    let calls = fake.calls();
    assert_eq!((calls.looks, calls.reads), (2, 1), "failed calls count too");
    fake.heal();
    assert_eq!(run.look(), Ok(under(fake.spot().box_id, selected())));
    assert_eq!(read(&fake, run.as_mut(), 5), Ok("Hello".to_owned()));
}

#[test]
fn the_clipboard_answers_as_scripted_and_no_other_call_counts_as_a_read() {
    let fake = FakeSelectionWatch::new();
    let (mut run, _) = watched(&fake);
    fake.set_text(TEXT);
    fake.focus(selected());
    fake.set_clip(true);
    assert!(run.clip_holds());
    fake.set_clip(false);
    assert!(!run.clip_holds());
    assert!(run.look().is_ok() && run.still(fake.spot().box_id));
    let want = SelCalls {
        watches: 1,
        looks: 1,
        stills: 1,
        clips: 2,
        ..SelCalls::default()
    };
    assert_eq!(fake.calls(), want, "no text was read");
}
