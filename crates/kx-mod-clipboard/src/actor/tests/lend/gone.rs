//! A copy that left the history never goes back (owner's check, 2026-11-21): after a take, a delete or a sweep of the
//! entry on the clipboard, or a take whose note came late or not yet, Paste at once pastes and then leaves the
//! clipboard empty. With no text there it is typed, as it may be a picture.

use kx_module_api::ModuleId;
use kx_module_api::ports::{ClipAct, ClipPart, ClipTake};

use super::super::super::Msg;
use super::super::rig::Rig;
use super::{SAID, holding, html, lend, lent};
use crate::fixture::{date, image, text};

/// A key copied from a service's page.
const KEY: &str = "sk-zebra-quartz";
/// Quick-fill's module, then the voice key page's: both send the same take.
const ASKERS: [&str; 2] = ["vault", "voice"];
/// The characters of the field a take is asked for.
const FIELD: usize = 50;

/// `asker`'s Paste takes the clipboard's text.
fn take(rig: &mut Rig, asker: &'static str) {
    rig.send(Msg::Take(ClipTake::new(ModuleId::new(asker), 1, FIELD)));
}

/// Lends, the keyboard pastes and the gap passes; checks the text was pasted, not typed, and gives the writes since.
fn pasted(rig: &mut Rig) -> Vec<Vec<ClipPart>> {
    let (writes, pastes) = (rig.fakes.writer.writes().len(), rig.pastes().len());
    lend(rig);
    assert_eq!(rig.pastes().len(), pastes + 1, "pasted, not typed");
    rig.answer(true);
    assert_eq!(lent(rig).last(), Some(&true));
    rig.advance(rig.settings.paste_gap());
    rig.fakes.writer.writes().split_off(writes)
}

/// The text pasted, then an empty clipboard.
fn emptied() -> Vec<Vec<ClipPart>> {
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
