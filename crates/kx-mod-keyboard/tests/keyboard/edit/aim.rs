//! An edit takes an open panel's aim before it sends, as a paste does, so a refused send or a terminal's Cut still
//! leaves the next letters going to the app.

use super::{Chips, done};
use crate::rig::STATE;
use kx_module_api::ports::{ClipPanel, Edit, FaceAction, ShortcutError};

/// Opens the clipboard panel, aimed at, asks for `edit` in round 1, then checks the answer and the aim.
fn aim_goes(c: &Chips, edit: Edit, terminal: bool, ok: bool) {
    c.b.send(&ClipPanel { open: true });
    assert_eq!(c.b.rig.state()["aimed"], true);
    c.ask(1, edit, terminal);
    assert_eq!(c.done(), [done(1, ok)], "{edit:?}");
    assert_eq!(c.b.rig.act(FaceAction::Relabel), Ok(STATE), "{edit:?}");
    assert_eq!(c.b.rig.state()["aimed"], false, "{edit:?}");
}

#[test]
fn an_edit_takes_the_aim_from_an_open_panel_as_its_keys_went_to_the_app() {
    aim_goes(&Chips::quiet(), Edit::Paste, false, true);
}

#[test]
fn a_refused_edit_takes_the_aim_too() {
    let c = Chips::quiet();
    let refused = ShortcutError::Refused { sent: 0, total: 4 };
    c.b.rig.fakes.shortcuts.set_error(Some(refused));
    aim_goes(&c, Edit::Copy, false, false);
    assert!(c.pieces().is_empty(), "nothing reached the app");
}

#[test]
fn a_terminal_cut_sends_nothing_and_takes_the_aim_too() {
    let c = Chips::quiet();
    aim_goes(&c, Edit::Cut, true, false);
    assert!(c.sent().is_empty() && c.pieces().is_empty());
}
