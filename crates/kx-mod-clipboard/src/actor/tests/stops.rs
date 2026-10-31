//! Inserts that stop with a note, the panel left open: a refused or failed write, nothing left to paste, a files
//! thread gone or past `check_ms`, and damaged content at Insert or Preview.

use std::time::Duration;

use kx_module_api::keys;
use kx_module_api::ports::{ClipAct, ClipError, FileFact, PanelNote, SealedError};
use kx_platform_fake::ShelfCall;

use super::super::Msg;
use super::rig::Rig;
use crate::fixture::files;

/// A rig holding a text copy of `t`, with the panel open and its row ticked.
fn ticked(t: &str) -> Rig {
    let mut rig = Rig::new();
    rig.text(t);
    rig.open();
    rig.tick_row(0);
    rig
}

/// Clicks Insert.
fn insert(rig: &mut Rig) {
    rig.send(Msg::Act(ClipAct::Insert));
}

/// The insert stopped with `note`, the panel open to show it.
fn stopped(rig: &Rig, note: PanelNote) {
    assert!(rig.pastes().is_empty(), "nothing pasted");
    assert_eq!(rig.page().note, Some(note));
    assert_eq!(rig.panels().last(), Some(&true));
}

#[test]
fn a_refused_or_failed_write_stops_with_paste_failed_and_busy_stays_busy() {
    for (error, note) in [
        (ClipError::Refused, PanelNote::PasteFailed),
        (ClipError::Io, PanelNote::PasteFailed),
        (ClipError::Busy, PanelNote::Busy),
    ] {
        let mut rig = ticked("a");
        rig.fakes.writer.queue(Err(error));
        insert(&mut rig);
        stopped(&rig, note);
    }
}

#[test]
fn an_insert_whose_entries_are_all_gone_or_away_stays_open_with_a_note() {
    let mut rig = ticked("a");
    rig.lose("a");
    insert(&mut rig);
    stopped(&rig, PanelNote::PasteFailed);
    let mut rig = Rig::new();
    rig.fakes.clip.set_file("E:/song.mp3", FileFact::Size(9));
    rig.copy(vec![files(&["E:/song.mp3"])], None);
    rig.open();
    rig.fakes.clip.set_file("E:/song.mp3", FileFact::Away);
    rig.tick_row(0);
    insert(&mut rig);
    stopped(&rig, PanelNote::PasteFailed);
    assert_eq!(rig.titles(), ["song.mp3"], "out of reach, still kept");
}

#[test]
fn a_files_thread_gone_stops_the_insert_with_a_note() {
    let mut rig = Rig::new();
    rig.fakes.clip.set_file("C:\\a.txt", FileFact::Size(9));
    rig.copy(vec![files(&["C:\\a.txt"])], None);
    rig.open();
    rig.lose_files();
    rig.tick_row(0);
    insert(&mut rig);
    stopped(&rig, PanelNote::PasteFailed);
}

#[test]
fn a_file_check_past_check_ms_stops_the_insert_and_its_late_answer_is_dropped() {
    let mut rig = Rig::new();
    rig.fakes.clip.set_file("C:\\a.txt", FileFact::Size(9));
    rig.copy(vec![files(&["C:\\a.txt"])], None);
    rig.open();
    rig.hold_looks(true);
    rig.tick_row(0);
    insert(&mut rig);
    let wait = rig.settings.check_wait();
    rig.advance(wait.checked_sub(Duration::from_millis(1)).unwrap());
    assert_eq!(rig.page().note, None, "still checking");
    rig.advance(Duration::from_millis(1));
    stopped(&rig, PanelNote::PasteFailed);
    rig.hold_looks(false);
    assert_eq!(
        rig.fakes.writer.tries(),
        0,
        "the late answer pastes nothing"
    );
    assert!(rig.pastes().is_empty());
}

#[test]
fn damaged_content_at_insert_deletes_the_entry_and_tells_the_user() {
    let mut rig = ticked("a");
    rig.fakes.shelf.fail(ShelfCall::Load, SealedError::Damaged);
    insert(&mut rig);
    assert_eq!(rig.notices(), [keys::CLIP_DAMAGED]);
    assert!(rig.page().empty);
    assert_eq!(rig.page().note, Some(PanelNote::PasteFailed));
}

#[test]
fn damaged_content_at_preview_deletes_the_entry_and_tells_the_user() {
    let mut rig = Rig::new();
    rig.text("a");
    rig.open();
    rig.fakes.shelf.fail(ShelfCall::Load, SealedError::Damaged);
    rig.send(Msg::Act(ClipAct::Preview));
    assert_eq!(rig.notices(), [keys::CLIP_DAMAGED]);
    assert!(rig.page().empty && rig.page().preview.is_none());
}
