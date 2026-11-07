//! A key with Ctrl, Alt or Win latched goes to the app as a shortcut while a panel is aimed at (ADR-0040 decision 1), as
//! a letter does, and lets the latch go; Shift stays the panel's.

use super::{LETTER, Quick};
use crate::assist::scans;
use crate::clip::{self, Clip};
use crate::fill::{Heard, text};
use kx_module_api::ports::{PanelKey, Scan};
use kx_platform_fake::Stroke;

/// Each latch that makes a shortcut, with its scan code.
const LATCHES: [(&str, u16); 3] = [("ctrl", 0x1D), ("alt", 0x38), ("win", 0xE05B)];
/// Every key with a `nav` on the board, with its scan code.
const NAVS: [(&str, u16); 9] = [
    ("up", 0xE048),
    ("down", 0xE050),
    ("left", 0xE04B),
    ("right", 0xE04D),
    ("enter", 0x1C),
    ("esc", 0x01),
    ("space", 0x39),
    ("bksp", 0x0E),
    ("del", 0xE053),
];
/// The board's Delete key, with its scan code.
const DELETE: (&str, u16) = ("del", 0xE053);
/// The board's W key.
const W: &str = "c-0x0011";

/// The chord of `latch` on `key`.
fn chord(latch: u16, key: u16) -> Stroke {
    Stroke::Chord(scans(&[latch]), Scan::new(key))
}

#[test]
fn ctrl_delete_reaches_the_app_and_the_next_letter_goes_to_the_panel_with_no_latch_left() {
    let q = Quick::opened(None);
    q.b.rig.tap("ctrl").unwrap();
    q.b.rig.tap(DELETE.0).unwrap();
    assert_eq!(q.strokes(), [chord(0x1D, DELETE.1)]);
    assert!(q.heard().is_empty(), "no FillKey goes out");
    assert!(!q.on("ctrl"), "Ctrl let go");
    q.b.rig.tap(W).unwrap();
    assert_eq!(q.heard(), [text("w")]);
    assert!(q.strokes().is_empty(), "the panel took the letter");
}

#[test]
fn every_nav_key_with_each_latch_reaches_the_app_and_never_the_panel() {
    let q = Quick::opened(None);
    for (latch, held) in LATCHES {
        for (id, code) in NAVS {
            q.b.rig.tap(latch).unwrap();
            q.b.rig.tap(id).unwrap();
            assert_eq!(q.strokes(), [chord(held, code)], "{latch} {id}");
            assert!(!q.on(latch), "{latch} let go after {id}");
        }
    }
    assert!(q.heard().is_empty(), "no FillKey goes out");
}

#[test]
fn a_press_with_a_latch_is_the_apps_shortcut_too() {
    let q = Quick::opened(None);
    q.b.rig.tap("ctrl").unwrap();
    q.b.rig.press("bksp").unwrap();
    assert_eq!(q.strokes(), [chord(0x1D, 0x0E)]);
    assert!(q.heard().is_empty());
}

#[test]
fn the_clipboard_panel_leaves_a_chord_to_the_app_as_well() {
    let c = Clip::quiet(None);
    c.open(true);
    for (latch, held) in LATCHES {
        c.b.rig.tap(latch).unwrap();
        c.b.rig.tap("left").unwrap();
        assert_eq!(c.strokes(), [chord(held, 0xE04B)], "{latch}");
    }
    assert!(c.heard().is_empty(), "no ClipKey goes out");
    c.b.rig.tap(LETTER).unwrap();
    assert_eq!(c.heard(), [clip::text("q")]);
}

#[test]
fn shift_leaves_a_nav_key_to_the_panel_and_stays_latched() {
    let q = Quick::opened(None);
    q.b.rig.tap("shift").unwrap();
    q.b.rig.tap(DELETE.0).unwrap();
    assert_eq!(q.heard(), [Heard::Key(PanelKey::Delete)]);
    assert!(q.strokes().is_empty());
    assert!(q.on("shift"), "as it always was");
}

#[test]
fn space_on_a_panel_lets_shift_go_as_a_typed_letter_does() {
    let q = Quick::opened(None);
    q.b.rig.tap("shift").unwrap();
    q.b.rig.tap("space").unwrap();
    assert_eq!(q.heard(), [Heard::Key(PanelKey::Tick)]);
    assert!(!q.on("shift"), "the next letter is small");
    q.b.rig.tap(W).unwrap();
    assert_eq!(q.heard(), [text("w")]);
}
