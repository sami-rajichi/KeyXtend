//! Saves and moves: content found gone at a move or a save deletes its entry, and a failed save tries again after
//! `save_ms`, logged once.

use kx_module_api::ports::{BlobName, ClipAct, SealedError};
use kx_platform_fake::ShelfCall;

use super::super::Msg;
use super::logs::lines_of;
use super::rig::{LIST, Rig, TODAY};

/// The log line of a failed save.
const UNSAVED: &str = "was not saved";

/// A rig holding text copies of `texts`, the last at the top, with the panel open.
fn opened(texts: &[&str]) -> Rig {
    let mut rig = Rig::new();
    for t in texts {
        rig.text(t);
    }
    rig.open();
    rig
}

#[test]
fn a_pin_whose_content_is_gone_deletes_the_entry() {
    let mut rig = opened(&["a", "b"]);
    rig.lose("a");
    rig.tick_row(1);
    rig.send(Msg::Act(ClipAct::Pin));
    assert_eq!(rig.titles(), ["b"]);
}

#[test]
fn a_waiting_move_whose_content_went_deletes_the_entry_at_the_next_save() {
    let mut rig = opened(&["a"]);
    rig.fakes.shelf.fail(ShelfCall::Save, SealedError::Io);
    rig.tick_row(0);
    rig.send(Msg::Act(ClipAct::Pin));
    assert_eq!(rig.titles(), ["a"], "the move waits");
    rig.fakes
        .shelf
        .delete_by_hand(&BlobName::new(TODAY).unwrap());
    rig.advance(rig.settings.save());
    assert!(rig.page().empty);
}

#[test]
fn a_failed_save_tries_again_after_save_ms_and_logs_once() {
    let mut rig = Rig::new();
    rig.text("a");
    let lines = lines_of(|| {
        for _ in 0..2 {
            rig.fakes.shelf.fail(ShelfCall::Save, SealedError::Io);
            rig.advance(rig.settings.save());
        }
    });
    let unsaved = lines.iter().filter(|l| l.contains(UNSAVED)).count();
    assert_eq!(unsaved, 1, "{lines:?}");
    assert!(!rig.names(TODAY).contains(&LIST.to_owned()));
    rig.advance(rig.settings.save());
    assert!(
        rig.names(TODAY).contains(&LIST.to_owned()),
        "the third try saves"
    );
}
