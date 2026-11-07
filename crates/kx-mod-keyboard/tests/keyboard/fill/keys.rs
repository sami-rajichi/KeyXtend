//! Our keys while Quick-fill is open (spec §3.3, §8.1): the moves, Space as `Tick`, Backspace as `Erase`, Delete and
//! the letters in the layout shown go to the panel, as they go to the clipboard panel; every other key types as usual.

use super::{Heard, Quick, text, unicode};
use crate::assist::scans;
use crate::rig::{HELD, STATE};
use kx_module_api::ports::{PanelKey, Scan};
use kx_platform_fake::Stroke;

/// The French layout, whose `^` key (0x1A) is dead.
const FR: &str = "fr-fr";
/// The scan code of the Delete key.
const DELETE: u16 = 0xE053;

#[test]
fn with_quick_fill_open_space_ticks_backspace_erases_delete_deletes_and_the_moves_drive_it() {
    let q = Quick::opened(None);
    for id in [
        "space", "bksp", "del", "up", "down", "left", "right", "enter",
    ] {
        assert_eq!(q.b.rig.press(id), Ok(STATE), "{id} never holds");
    }
    q.b.rig.tap("esc").unwrap();
    let keys = [
        PanelKey::Tick,
        PanelKey::Erase,
        PanelKey::Delete,
        PanelKey::Up,
        PanelKey::Down,
        PanelKey::Left,
        PanelKey::Right,
        PanelKey::Enter,
        PanelKey::Esc,
    ];
    assert_eq!(q.heard(), keys.map(Heard::Key));
    assert!(q.strokes().is_empty(), "nothing reaches the app");
    assert!(q.b.rig.fakes.shortcuts.sent().is_empty());
}

#[test]
fn letters_go_to_its_line_in_the_layout_shown_with_shift_caps_and_altgr() {
    let q = Quick::opened(None);
    q.b.rig.press("c-0x0010").unwrap();
    q.b.rig.tap("shift").unwrap();
    q.b.rig.tap("c-0x0011").unwrap();
    assert!(!q.on("shift"), "Shift let go as after typing");
    q.b.rig.fakes.source.set_caps(true);
    q.b.rig.tap("c-0x0012").unwrap();
    q.b.rig.fakes.source.set_caps(false);
    q.b.rig.switch(FR);
    q.b.rig.tap("altgr").unwrap();
    q.b.rig.tap("c-0x0012").unwrap();
    q.b.rig.switch("ar-tn");
    q.b.rig.tap("c-0x0010").unwrap();
    let want = [text("q"), text("W"), text("E"), text("€"), text("ض")];
    assert_eq!(q.heard(), want);
    assert!(q.strokes().is_empty());
}

#[test]
fn a_dead_key_gives_the_line_nothing() {
    let q = Quick::opened(None);
    assert_eq!(q.b.rig.switch(FR), STATE);
    assert_eq!(q.b.rig.tap("c-0x001A"), Ok(STATE), "the panel takes it");
    assert!(q.heard().is_empty() && q.strokes().is_empty());
}

#[test]
fn ctrl_alt_or_win_latched_sends_a_letter_to_the_app_as_usual() {
    let q = Quick::opened(None);
    for (latch, code) in [("ctrl", 0x1D), ("alt", 0x38), ("win", 0xE05B)] {
        q.b.rig.tap(latch).unwrap();
        q.b.rig.press("c-0x002E").unwrap();
        let chord = Stroke::Chord(scans(&[code]), Scan::new(0x2E));
        assert_eq!(q.strokes(), [chord], "{latch}");
    }
    assert!(q.heard().is_empty());
}

#[test]
fn keys_it_does_not_take_type_as_usual() {
    let q = Quick::opened(None);
    q.b.rig.tap("tab").unwrap();
    assert_eq!(q.strokes(), [Stroke::Tap(Scan::new(0x0F))]);
    assert!(q.heard().is_empty());
}

#[test]
fn with_quick_fill_shut_space_backspace_delete_and_letters_type_as_usual() {
    let q = Quick::opened(None);
    q.open(false);
    for id in ["space", "bksp", "del", "c-0x0010"] {
        q.b.rig.tap(id).unwrap();
    }
    assert!(q.heard().is_empty());
    let tap = |code| Stroke::Tap(Scan::new(code));
    assert_eq!(
        q.strokes(),
        [tap(0x39), tap(0x0E), tap(DELETE), unicode("q")]
    );
    assert_eq!(q.b.rig.press("c-0x0010"), Ok(HELD), "a letter holds again");
}
