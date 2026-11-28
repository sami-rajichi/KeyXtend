//! The hand's Esc (`SnipClick::Esc`, ADR-0059): while the screens freeze or show frozen it does what our keyboard's Esc
//! does, so the user always has a way out; at rest, during a countdown or a recording, in the editor, or with a system
//! dialog in front it does nothing, so that Esc is the app's or the dialog's.

use kx_module_api::ports::{BarKey, FolderKey, SnipAside, SnipClick, SnipLit, SnipStage, StripKey};

use super::super::Msg;
use super::rig::{Heard, Rig, held};

/// The light and the step aside a snip's end tells: the key goes dark and the keyboard comes back.
const ENDED: [Heard; 2] = [Heard::Lit(SnipLit::Off), Heard::Aside(SnipAside::Back)];

/// What `rig` told of its light and step aside after `from` of them.
fn sides_since(rig: &Rig, from: usize) -> Vec<Heard> {
    rig.sides().split_off(from)
}

#[test]
fn it_ends_a_snip_being_picked_so_the_frozen_screens_go_and_the_keyboard_comes_back() {
    let mut rig = Rig::new();
    rig.key();
    assert!(matches!(rig.stage(), SnipStage::Pick(_)));
    let from = rig.sides().len();
    rig.act(SnipClick::Esc);
    assert_eq!(rig.stage(), SnipStage::Idle);
    assert!(rig.page().frozen.is_empty(), "the frozen screens drop");
    assert_eq!(sides_since(&rig, from), ENDED);
}

#[test]
fn it_ends_a_snip_still_freezing_which_then_never_freezes() {
    let mut rig = Rig::new();
    rig.send(Msg::Key);
    let (from, froze) = (rig.sides().len(), rig.fake.freeze.caps().len());
    rig.act(SnipClick::Esc);
    assert_eq!(sides_since(&rig, from), ENDED);
    rig.act(SnipClick::Aside);
    assert_eq!(
        rig.fake.freeze.caps().len(),
        froze,
        "a late answer freezes nothing"
    );
}

#[test]
fn on_a_strip_over_frozen_screens_it_closes_and_a_second_esc_drops_a_picture_not_kept() {
    let mut rig = Rig::new();
    rig.take(BarKey::VideoRegion);
    rig.act(SnipClick::Esc);
    assert_eq!(rig.stage(), SnipStage::Idle, "nothing held yet");
    rig.snap();
    rig.act(SnipClick::Esc);
    assert!(rig.page().ask.is_some(), "a picture not kept asks first");
    let from = rig.sides().len();
    rig.act(SnipClick::Esc);
    assert_eq!(rig.stage(), SnipStage::Idle, "Esc means Don't save there");
    assert!(rig.page().frozen.is_empty() && rig.page().ask.is_none());
    assert_eq!(sides_since(&rig, from), ENDED);
}

#[test]
fn over_frozen_screens_it_closes_the_snip_panel_the_tray_opened_first() {
    let mut rig = Rig::new();
    rig.key();
    rig.act(SnipClick::Open);
    assert!(rig.page().panel.is_some());
    rig.act(SnipClick::Esc);
    assert!(rig.page().panel.is_none());
    assert!(matches!(rig.stage(), SnipStage::Pick(_)), "the snip stays");
    rig.act(SnipClick::Esc);
    assert_eq!(rig.stage(), SnipStage::Idle);
}

#[test]
fn while_a_system_dialog_shows_over_frozen_screens_the_esc_is_the_dialogs() {
    let mut rig = Rig::new();
    rig.key();
    rig.act(SnipClick::Open);
    rig.actor.ports.spawn = held;
    rig.act(SnipClick::Folder(FolderKey::Change));
    assert!(rig.page().dialog, "the folder dialog shows");
    rig.act(SnipClick::Esc);
    assert!(
        rig.page().panel.is_some(),
        "the panel stays behind the dialog"
    );
    assert!(matches!(rig.stage(), SnipStage::Pick(_)));
    rig.drain();
}

#[test]
fn at_rest_counting_down_recording_or_editing_it_does_nothing() {
    let mut rig = Rig::new();
    rig.act(SnipClick::Open);
    rig.act(SnipClick::Esc);
    assert!(rig.page().panel.is_some(), "the panel at rest stays");
    let mut rig = Rig::new();
    rig.take(BarKey::VideoRegion);
    rig.act(SnipClick::Strip(StripKey::Record));
    rig.act(SnipClick::Esc);
    assert!(
        matches!(rig.stage(), SnipStage::Count(_)),
        "the countdown goes on"
    );
    let mut rig = Rig::new();
    rig.record();
    rig.act(SnipClick::Esc);
    assert!(
        matches!(rig.stage(), SnipStage::Record(_)),
        "the recording goes on"
    );
    let mut rig = Rig::new();
    rig.snap();
    rig.act(SnipClick::Strip(StripKey::Edit));
    let from = rig.views().len();
    rig.act(SnipClick::Esc);
    assert!(matches!(rig.stage(), SnipStage::Edit(_)));
    assert_eq!(rig.views().len(), from, "the editor heard nothing");
}
