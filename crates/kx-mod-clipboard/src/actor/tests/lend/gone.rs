//! A copy that left the history never goes back (owner's check, 2026-11-21): after a take, a delete or a sweep of the
//! entry on the clipboard, or a take whose note came late or not yet, Paste at once pastes and then leaves the
//! clipboard empty. With no text there it is typed, as it may be a picture.

use kx_module_api::ModuleId;
use kx_module_api::ports::{ClipAct, ClipError, ClipNote, ClipPart, ClipTake, SealedError};
use kx_platform_fake::ShelfCall;

use super::super::super::Msg;
use super::super::rig::Rig;
use super::{SAID, holding, html, lend, lent};
use crate::fixture::{date, image, text};

/// A key copied from a service's page.
pub(super) const KEY: &str = "sk-zebra-quartz";
/// Quick-fill's module, then the voice key page's: both send the same take.
const ASKERS: [&str; 2] = ["vault", "voice"];
/// The characters of the field a take is asked for.
const FIELD: usize = 50;

/// `asker`'s Paste takes the clipboard's text.
pub(super) fn take(rig: &mut Rig, asker: &'static str) {
    rig.send(Msg::Take(ClipTake::new(ModuleId::new(asker), 1, FIELD)));
}

/// Lends, the keyboard pastes and the gap passes; checks the text was pasted, not typed, and gives the writes since.
pub(super) fn pasted(rig: &mut Rig) -> Vec<Vec<ClipPart>> {
    let (writes, pastes) = (rig.fakes.writer.writes().len(), rig.pastes().len());
    lend(rig);
    assert_eq!(rig.pastes().len(), pastes + 1, "pasted, not typed");
    rig.answer(true);
    assert_eq!(lent(rig).last(), Some(&true));
    rig.advance(rig.settings.paste_gap());
    rig.fakes.writer.writes().split_off(writes)
}

/// The text pasted, then an empty clipboard.
pub(super) fn emptied() -> Vec<Vec<ClipPart>> {
    vec![vec![text(SAID)], vec![]]
}

/// Ticks row `row` of the panel and deletes it.
fn delete_row(rig: &mut Rig, row: u8) {
    rig.open();
    rig.tick_row(row);
    rig.send(Msg::Act(ClipAct::Delete));
}

#[test]
fn a_taken_copy_is_pasted_then_the_clipboard_left_empty_for_either_asker() {
    for asker in ASKERS {
        let mut rig = holding(vec![text(KEY)]);
        take(&mut rig, asker);
        assert_eq!(pasted(&mut rig), emptied(), "{asker}");
        let all = rig.fakes.writer.writes();
        assert_eq!(all, emptied(), "{asker}: never the key");
    }
}

#[test]
fn a_take_before_its_copys_note_never_writes_the_taken_text_back() {
    let mut rig = Rig::new();
    rig.fakes.clip.set_text(Some(KEY));
    take(&mut rig, ASKERS[0]);
    rig.text(KEY);
    assert_eq!(pasted(&mut rig), emptied());
}

#[test]
fn a_take_whose_note_is_still_due_never_writes_the_taken_text_back() {
    let mut rig = holding(vec![text("older")]);
    rig.fakes.clip.set_text(Some(KEY));
    take(&mut rig, ASKERS[1]);
    assert_eq!(pasted(&mut rig), emptied());
}

/// The lend seed's steps (T24): a copy taken before its note, a lend that times out, the note, another copy, then the
/// same text copied again, its note still due, and a busy take; that text stays gone until a copy past `take_wait_ms`
/// is kept again.
#[test]
fn the_same_text_copied_again_after_its_take_stays_gone_until_kept_again() {
    let mut rig = Rig::new();
    rig.fakes.clip.set_text(Some(KEY));
    take(&mut rig, ASKERS[1]);
    lend(&mut rig);
    rig.fakes.clip.fail_text(ClipError::Busy);
    rig.advance(rig.settings.paste_wait());
    assert_eq!(rig.fakes.writer.writes(), emptied(), "the taken key never");
    rig.text(KEY);
    rig.fakes.clip.set_text(Some("next"));
    rig.text("next");
    rig.fakes.clip.set_text(Some(KEY));
    take(&mut rig, ASKERS[1]);
    assert_eq!(pasted(&mut rig), emptied(), "not kept again yet");
    rig.text(KEY);
    assert_eq!(pasted(&mut rig), emptied(), "still within the wait");
    rig.advance(rig.settings.take_wait());
    rig.text(KEY);
    assert_eq!(pasted(&mut rig), [vec![text(SAID)], vec![text(KEY)]]);
}

/// A sibling (T24): a text a take read comes back to the clipboard after another copy was kept, as a mark the board
/// takes for our echo; it was never kept again, so the lend leaves the clipboard empty.
#[test]
fn a_taken_text_back_on_the_clipboard_after_another_copy_never_goes_back() {
    let mut rig = Rig::new();
    rig.fakes.clip.set_text(Some(KEY));
    rig.push(ClipNote::Marked { app: None });
    take(&mut rig, ASKERS[1]);
    rig.fakes.clip.set_text(Some("next"));
    rig.text("next");
    lend(&mut rig);
    rig.advance(rig.settings.paste_wait());
    rig.fakes.clip.set_text(Some(KEY));
    rig.push(ClipNote::Marked { app: None });
    assert_eq!(pasted(&mut rig), emptied());
}

/// A sibling (T24): a key taken before its note, an entry inserted and then taken, then the key's late note; that note
/// does not undo the second take, so the lend leaves the clipboard empty, never the inserted text. The second take
/// never drops the first's wait either, so the key's note keeps nothing (lend review, case a).
#[test]
fn a_late_note_of_an_earlier_take_never_brings_back_a_text_taken_since() {
    let mut rig = holding(vec![text("older")]);
    rig.fakes.clip.set_text(Some(KEY));
    take(&mut rig, ASKERS[1]);
    rig.open();
    rig.tick_row(0);
    rig.send(Msg::Act(ClipAct::Insert));
    rig.fakes.clip.set_text(Some("older"));
    rig.advance(rig.settings.paste_wait());
    take(&mut rig, ASKERS[1]);
    rig.text(KEY);
    let entries = rig.actor.history.entries();
    assert!(
        entries.iter().all(|e| e.title.expose() != KEY),
        "never kept"
    );
    assert_eq!(pasted(&mut rig), emptied());
}

/// A sibling of the lend seed (T24): the text a lend would put back is copied again and taken while the lend runs, its
/// note not yet come, so the lend ends with the clipboard empty, never that text.
#[test]
fn a_take_during_a_lend_of_the_text_it_would_put_back_leaves_the_clipboard_empty() {
    let mut rig = Rig::new();
    rig.fakes.clip.fail_text(ClipError::Busy);
    rig.fakes.clip.set_text(Some(KEY));
    take(&mut rig, ASKERS[1]);
    lend(&mut rig);
    rig.fakes.clip.set_text(Some(KEY));
    take(&mut rig, ASKERS[1]);
    rig.advance(rig.settings.paste_wait());
    assert_eq!(rig.fakes.writer.writes(), emptied());
}

/// Another sibling (T24): a text kept, the clipboard cleared, then the same text copied again and taken before its
/// note; its older entry goes, and the lend leaves the clipboard empty, never that text.
#[test]
fn a_take_of_a_kept_text_copied_again_before_its_note_never_comes_back() {
    let mut rig = Rig::new();
    lend(&mut rig);
    rig.fakes.clip.fail_text(ClipError::Busy);
    rig.fakes.clip.set_text(Some(KEY));
    take(&mut rig, ASKERS[1]);
    rig.text(KEY);
    rig.fakes.clip.set_text(None);
    rig.push(ClipNote::Cleared { app: None });
    rig.fakes.clip.set_text(Some(KEY));
    take(&mut rig, ASKERS[1]);
    assert!(
        rig.actor.history.entries().is_empty(),
        "the older entry goes"
    );
    assert_eq!(pasted(&mut rig), emptied());
}

/// The lend review's I1 (T24): a copy whose save failed was never kept, so it never left the history and goes back.
#[test]
fn a_copy_whose_save_failed_still_goes_back() {
    let mut rig = Rig::new();
    rig.fakes.shelf.fail(ShelfCall::Save, SealedError::Io);
    rig.fakes.clip.set_text(Some("unsaved"));
    rig.text("unsaved");
    assert!(rig.actor.history.entries().is_empty(), "never kept");
    assert_eq!(pasted(&mut rig), [vec![text(SAID)], vec![text("unsaved")]]);
}

#[test]
fn the_next_copy_after_a_take_comes_back_whole() {
    let mut rig = holding(vec![text(KEY)]);
    take(&mut rig, ASKERS[0]);
    rig.fakes.clip.set_text(Some("next"));
    rig.text("next");
    assert_eq!(pasted(&mut rig), [vec![text(SAID)], vec![text("next")]]);
}

#[test]
fn a_delete_or_delete_all_of_the_entry_on_the_clipboard_pastes_then_leaves_it_empty() {
    let mut rig = holding(vec![text("kept")]);
    delete_row(&mut rig, 0);
    assert_eq!(pasted(&mut rig), emptied(), "Delete");
    let mut rig = holding(vec![text("kept")]);
    rig.open();
    rig.send(Msg::Act(ClipAct::DeleteAll));
    rig.send(Msg::Act(ClipAct::Confirm(true)));
    assert_eq!(pasted(&mut rig), emptied(), "Delete all");
}

#[test]
fn deleting_an_older_entry_still_puts_the_newest_back_whole() {
    let mut rig = Rig::new();
    rig.text("older");
    let rich = vec![text("kept"), html("<b>kept</b>")];
    rig.fakes.clip.set_text(Some("kept"));
    rig.copy(rich.clone(), None);
    delete_row(&mut rig, 1);
    assert_eq!(rig.titles(), ["kept"]);
    assert_eq!(pasted(&mut rig), [vec![text(SAID)], rich]);
}

#[test]
fn a_swept_entry_on_the_clipboard_pastes_then_leaves_it_empty() {
    let mut rig = holding(vec![text("kept")]);
    rig.set_time(date(2026, 11, 7), 0, 0);
    assert_eq!(pasted(&mut rig), emptied(), "past keep_days");
}

#[test]
fn a_deleted_picture_on_the_clipboard_is_still_typed() {
    let mut rig = holding(vec![image()]);
    delete_row(&mut rig, 0);
    lend(&mut rig);
    assert_eq!((lent(&rig), rig.fakes.writer.tries()), (vec![false], 0));
}

#[test]
fn an_entry_deleted_during_the_gap_is_never_written_back() {
    let mut rig = holding(vec![text("kept")]);
    lend(&mut rig);
    rig.answer(true);
    delete_row(&mut rig, 0);
    rig.advance(rig.settings.paste_gap());
    assert_eq!(rig.fakes.writer.writes(), emptied());
}

/// A sibling (T24): a kept key is copied again, its note still due, so the board names another entry; deleting the
/// key's entry before the lend leaves the clipboard empty after the paste, never that key.
#[test]
fn a_text_deleted_while_the_board_names_another_entry_never_goes_back() {
    let mut rig = holding(vec![text(KEY)]);
    rig.fakes.clip.set_text(Some("next"));
    rig.text("next");
    rig.fakes.clip.set_text(Some(KEY));
    delete_row(&mut rig, 1);
    assert_eq!(pasted(&mut rig), emptied());
}

/// The same sibling with the delete while the lend runs, as the seed's steps had it.
#[test]
fn a_text_deleted_during_a_lend_that_saved_it_as_unnamed_never_goes_back() {
    let mut rig = holding(vec![text(KEY)]);
    rig.fakes.clip.set_text(Some("next"));
    rig.text("next");
    rig.fakes.clip.set_text(Some(KEY));
    lend(&mut rig);
    delete_row(&mut rig, 1);
    rig.answer(true);
    rig.advance(rig.settings.paste_gap());
    assert_eq!(rig.fakes.writer.writes(), emptied());
}
