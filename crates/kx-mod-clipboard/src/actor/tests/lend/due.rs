//! A take's copy whose note is due (lend review, known #2): every note of it the watch tells within `take_wait_ms`
//! keeps nothing, from any app and whatever comes between, and it is kept again after; at most `queue_copies` takes
//! wait.

use std::time::Duration;

use kx_module_api::ports::ClipNote;

use super::super::rig::Rig;
use super::gone::{KEY, emptied, pasted, take};
use super::{holding, html};
use crate::fixture::{app, text};

/// The voice key page, whose Paste takes the clipboard's text.
const ASKER: &str = "voice";
/// The clock's step just short of or just past a wait's end.
pub(super) const STEP: Duration = Duration::from_millis(1);

/// The texts the history keeps, oldest first.
pub(super) fn texts(rig: &Rig) -> Vec<String> {
    let mut entries = rig.actor.history.entries().to_vec();
    entries.sort_by_key(|e| e.seq);
    entries.iter().map(|e| e.title.expose().clone()).collect()
}

/// The review's case (b): an earlier copy's note, still due at the take, comes first and is kept; the taken copy's
/// note after it keeps nothing.
#[test]
fn an_earlier_copys_note_before_the_taken_ones_keeps_only_the_earlier_copy() {
    let mut rig = Rig::new();
    rig.fakes.clip.set_text(Some(KEY));
    take(&mut rig, ASKER);
    rig.text("earlier");
    rig.text(KEY);
    assert_eq!(texts(&rig), ["earlier"]);
    assert_eq!(pasted(&mut rig), emptied());
}

/// The review's case (c): an app sets the clipboard twice, so the taken copy is told again after the take forgot the
/// entry its first note kept; that second note keeps nothing.
#[test]
fn a_second_note_of_a_taken_copy_keeps_nothing() {
    let mut rig = holding(vec![text(KEY)]);
    take(&mut rig, ASKER);
    rig.text(KEY);
    assert!(texts(&rig).is_empty(), "never kept again");
    assert_eq!(pasted(&mut rig), emptied());
}

/// After the take, a second note of the key from another app keeps nothing, so Paste at once never puts it back.
#[test]
fn a_second_note_after_the_take_keeps_nothing() {
    let mut rig = Rig::new();
    rig.fakes.clip.set_text(Some(KEY));
    take(&mut rig, ASKER);
    rig.text(KEY);
    rig.copy(vec![text(KEY)], Some(app("notepad.exe")));
    assert!(texts(&rig).is_empty(), "never kept");
    assert_eq!(pasted(&mut rig), emptied());
}

/// After the take, the app writes the key again with HTML beside it; its text part keeps nothing either.
#[test]
fn a_rich_second_write_after_the_take_keeps_nothing() {
    let mut rig = Rig::new();
    rig.fakes.clip.set_text(Some(KEY));
    take(&mut rig, ASKER);
    rig.text(KEY);
    rig.copy(vec![text(KEY), html(KEY)], None);
    assert!(texts(&rig).is_empty(), "never kept");
    assert_eq!(pasted(&mut rig), emptied());
}

#[test]
fn a_taken_copys_note_keeps_nothing_within_take_wait_ms_and_is_kept_after() {
    let wait = Rig::new().settings.take_wait();
    let short = wait.checked_sub(STEP).expect("a wait past one step");
    for (by, kept) in [(short, false), (wait, true)] {
        let mut rig = Rig::new();
        rig.fakes.clip.set_text(Some(KEY));
        take(&mut rig, ASKER);
        rig.advance(by);
        rig.text(KEY);
        assert_eq!(texts(&rig) == [KEY], kept, "{by:?}");
    }
}

/// The wait counts to when the watch read the copy, so a stall on the actor's thread past the wait keeps nothing.
#[test]
fn a_note_read_within_the_wait_keeps_nothing_after_a_stall() {
    let mut rig = Rig::new();
    rig.fakes.clip.set_text(Some(KEY));
    take(&mut rig, ASKER);
    let parts = vec![text(KEY)];
    rig.fakes.clip.push(ClipNote::Copied { parts, app: None });
    rig.advance(rig.settings.take_wait() + STEP);
    assert!(texts(&rig).is_empty(), "read within the wait");
}

#[test]
fn past_queue_copies_takes_the_oldest_taken_copy_is_no_longer_due() {
    let mut rig = Rig::new();
    let keys: Vec<String> = (0..=rig.settings.queue_copies)
        .map(|n| format!("{KEY}{n}"))
        .collect();
    for key in &keys {
        rig.fakes.clip.set_text(Some(key));
        take(&mut rig, ASKER);
    }
    for key in &keys {
        rig.text(key);
    }
    assert_eq!(texts(&rig), keys[..1]);
}
