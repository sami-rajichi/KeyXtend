//! Quick-fill's take: what the clipboard holds now, read within the take's cap, whatever the history kept. The newest
//! entry is forgotten when the text was read in full and the entry is that copy and not pinned, else the next copy
//! keeps nothing if it is that text; no text, a text too long or a busy clipboard each answer as such, and nothing is
//! logged.

use kx_module_api::ports::{
    ClipAct, ClipError, ClipNote, ClipTake, ClipTaken, ClipText, MAX_FIELD_CHARS,
};
use kx_module_api::{Redacted, Zeroizing};
use kx_platform_fake::ShelfCall;
use toml::Value;

use super::super::Msg;
use super::ears::Heard;
use super::logs::lines_of;
use super::rig::Rig;
use crate::fixture::{app, image, text as part};

/// A password copied from a note.
const PASSWORD: &str = "zebra-quartz";
/// The characters of the field Paste was asked on.
const FIELD: usize = 50;

/// Asks the take of round `round` for a field of `FIELD` characters.
fn take(rig: &mut Rig, round: u32) {
    rig.send(Msg::Take(ClipTake::new(round, FIELD)));
}

/// The answer that carries `text`.
fn text(text: &str) -> ClipText {
    ClipText::Text(Redacted::new(Zeroizing::new(text.to_owned())))
}

/// Every answer, each as its round and what it found.
fn answers(rig: &Rig) -> Vec<(u32, ClipText)> {
    let taken = rig.all().into_iter().filter_map(|h| match h {
        Heard::Taken(ClipTaken { round, text }) => Some((round, text)),
        _ => None,
    });
    taken.collect()
}

/// The titles the history shows.
fn kept(rig: &mut Rig) -> Vec<String> {
    rig.open();
    let titles = rig.titles();
    rig.send(Msg::Act(ClipAct::Close));
    titles
}

#[test]
fn a_take_answers_the_text_held_now_and_forgets_the_newest_entry_that_is_this_copy() {
    let mut rig = Rig::new();
    rig.text("older");
    rig.text(PASSWORD);
    rig.fakes.clip.set_text(Some(PASSWORD));
    take(&mut rig, 3);
    assert_eq!(answers(&rig), [(3, text(PASSWORD))]);
    assert_eq!(kept(&mut rig), ["older"]);
    assert_eq!(
        rig.fakes.clip.take_cap(),
        Some(ClipTake::new(3, FIELD).cap())
    );
}

#[test]
fn the_read_stops_at_the_takes_cap_never_past_the_longest_fields_room() {
    let mut rig = Rig::new();
    let wide = ClipTake {
        round: 1,
        max_chars: usize::MAX,
    };
    rig.send(Msg::Take(wide));
    let most = ClipTake::new(1, MAX_FIELD_CHARS).cap();
    assert_eq!(rig.fakes.clip.take_cap(), Some(most));
}

#[test]
fn a_text_past_the_cap_is_too_long_and_forgets_nothing() {
    let mut rig = Rig::new();
    rig.text(PASSWORD);
    rig.fakes.clip.set_text(Some(PASSWORD));
    let short = ClipTake {
        round: 2,
        max_chars: PASSWORD.chars().count() - 1,
    };
    rig.send(Msg::Take(short));
    assert_eq!(answers(&rig), [(2, ClipText::TooLong)]);
    assert_eq!(kept(&mut rig), [PASSWORD]);
}

#[test]
fn a_copy_left_out_of_history_is_answered_and_forgets_nothing() {
    let mut rig = Rig::new();
    rig.text("older");
    rig.push(ClipNote::Marked { app: None });
    rig.fakes.clip.set_text(Some(PASSWORD));
    take(&mut rig, 1);
    assert_eq!(answers(&rig), [(1, text(PASSWORD))]);
    assert_eq!(kept(&mut rig), ["older"]);
}

#[test]
fn an_older_entry_holding_the_same_text_stays() {
    let mut rig = Rig::new();
    rig.text(PASSWORD);
    rig.text("newer");
    rig.fakes.clip.set_text(Some(PASSWORD));
    take(&mut rig, 1);
    assert_eq!(kept(&mut rig), ["newer", PASSWORD]);
}

#[test]
fn a_pinned_newest_entry_stays_as_the_user_kept_it() {
    let mut rig = Rig::new();
    rig.text(PASSWORD);
    rig.open();
    rig.tick_row(0);
    rig.send(Msg::Act(ClipAct::Pin));
    rig.send(Msg::Act(ClipAct::Close));
    rig.fakes.clip.set_text(Some(PASSWORD));
    take(&mut rig, 1);
    assert_eq!(answers(&rig), [(1, text(PASSWORD))]);
    assert_eq!(kept(&mut rig), [PASSWORD]);
}

#[test]
fn no_text_answers_empty_and_an_empty_history_forgets_nothing() {
    let mut rig = Rig::new();
    rig.fakes.clip.set_text(Some(PASSWORD));
    take(&mut rig, 1);
    rig.text("older");
    rig.fakes.clip.set_text(None);
    take(&mut rig, 2);
    assert_eq!(answers(&rig), [(1, text(PASSWORD)), (2, ClipText::Empty)]);
    assert_eq!(kept(&mut rig), ["older"]);
}

#[test]
fn a_held_refused_or_unreadable_clipboard_answers_busy_and_logs_only_its_kind() {
    let mut rig = Rig::new();
    rig.text(PASSWORD);
    rig.fakes.clip.set_text(Some(PASSWORD));
    let lines = lines_of(|| {
        let errors = [ClipError::Busy, ClipError::Refused, ClipError::Io];
        for (round, error) in (1..).zip(errors) {
            rig.fakes.clip.fail_text(error);
            take(&mut rig, round);
        }
    });
    let busy = [
        (1, ClipText::Busy),
        (2, ClipText::Busy),
        (3, ClipText::Busy),
    ];
    assert_eq!(answers(&rig), busy);
    assert_eq!(kept(&mut rig), [PASSWORD]);
    assert!(lines.iter().all(|l| !l.contains(PASSWORD)), "{lines:?}");
    assert_eq!(lines.len(), 3, "{lines:?}");
}

#[test]
fn a_take_and_its_forget_log_no_text() {
    let mut rig = Rig::new();
    rig.text(PASSWORD);
    rig.fakes.clip.set_text(Some(PASSWORD));
    let lines = lines_of(|| take(&mut rig, 1));
    assert!(lines.iter().all(|l| !l.contains(PASSWORD)), "{lines:?}");
}

#[test]
fn a_copys_note_that_comes_after_its_take_keeps_nothing() {
    let mut rig = Rig::new();
    rig.fakes.clip.set_text(Some(PASSWORD));
    let lines = lines_of(|| {
        take(&mut rig, 1);
        rig.text(PASSWORD);
    });
    assert_eq!(answers(&rig), [(1, text(PASSWORD))]);
    assert_eq!(rig.fakes.shelf.calls(ShelfCall::Save), 0, "nothing written");
    assert!(lines.iter().all(|l| !l.contains(PASSWORD)), "{lines:?}");
    assert!(kept(&mut rig).is_empty());
}

#[test]
fn a_late_note_with_other_parts_beside_the_text_keeps_nothing_either() {
    let mut rig = Rig::new();
    rig.fakes.clip.set_text(Some(PASSWORD));
    take(&mut rig, 1);
    rig.copy(vec![image(), part(PASSWORD)], None);
    assert_eq!(rig.fakes.shelf.calls(ShelfCall::Save), 0, "nothing written");
}

#[test]
fn a_different_copy_is_kept_and_ends_the_wait_so_the_same_text_after_it_is_kept() {
    let mut rig = Rig::new();
    rig.fakes.clip.set_text(Some(PASSWORD));
    take(&mut rig, 1);
    rig.text("other");
    assert_eq!(kept(&mut rig), ["other"]);
    rig.text(PASSWORD);
    assert_eq!(kept(&mut rig), [PASSWORD, "other"]);
}

#[test]
fn only_the_first_note_after_the_take_keeps_nothing() {
    let mut rig = Rig::new();
    rig.fakes.clip.set_text(Some(PASSWORD));
    take(&mut rig, 1);
    rig.text(PASSWORD);
    rig.text(PASSWORD);
    assert_eq!(kept(&mut rig), [PASSWORD]);
}

#[test]
fn a_skipped_apps_copy_ends_the_wait_too() {
    let skip = Value::Array(vec!["KeePass.exe".into()]);
    let mut rig = Rig::build(|_, host| host.put("skip_apps", skip));
    rig.fakes.clip.set_text(Some(PASSWORD));
    take(&mut rig, 1);
    rig.copy(vec![part("other")], Some(app("keepass.exe")));
    rig.text(PASSWORD);
    assert_eq!(kept(&mut rig), [PASSWORD]);
}

#[test]
fn a_later_take_replaces_the_text_that_waits() {
    let mut rig = Rig::new();
    rig.fakes.clip.set_text(Some("first"));
    take(&mut rig, 1);
    rig.fakes.clip.set_text(Some(PASSWORD));
    take(&mut rig, 2);
    rig.text("first");
    assert_eq!(kept(&mut rig), ["first"], "no longer waited for");
}

#[test]
fn a_later_take_that_forgets_its_copy_leaves_nothing_waiting() {
    let mut rig = Rig::new();
    rig.text(PASSWORD);
    rig.fakes.clip.set_text(Some("first"));
    take(&mut rig, 1);
    rig.fakes.clip.set_text(Some(PASSWORD));
    take(&mut rig, 2);
    rig.text("first");
    assert_eq!(kept(&mut rig), ["first"]);
}

#[test]
fn a_later_take_with_no_text_leaves_the_text_waiting() {
    let mut rig = Rig::new();
    rig.fakes.clip.set_text(Some(PASSWORD));
    take(&mut rig, 1);
    rig.fakes.clip.set_text(None);
    take(&mut rig, 2);
    rig.text(PASSWORD);
    assert_eq!(rig.fakes.shelf.calls(ShelfCall::Save), 0, "nothing written");
}
