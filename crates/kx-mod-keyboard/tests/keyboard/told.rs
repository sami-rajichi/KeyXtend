//! What typing tells word prediction (ADR-0033): each piece that reached the app, once and in order; a key that
//! sends nothing, or a send refused whole, tells nothing.

#![allow(
    clippy::unwrap_used,
    reason = "these fixtures run only in tests, where a failed setup should fail the test"
)]

use crate::known::{only, sid};
use crate::typing::Rig;
use kx_mod_keyboard::BoardSpec;
use kx_module_api::Redacted;
use kx_module_api::ports::{InjectError, Piece, Scan};
use kx_platform_fake::Stroke;
use proptest::prelude::*;

/// The US "a" and "c" keys' scan codes.
const A: u16 = 0x1E;
const C: u16 = 0x2E;
/// The scan codes of Backspace (shared with the other test files), the space bar, Enter, Tab and Caps Lock.
pub const BKSP: u16 = 0x0E;
const SPACE: u16 = 0x39;
const ENTER: u16 = 0x1C;
const TAB: u16 = 0x0F;
const CAPS: u16 = 0x3A;
/// The French dead `^`, and `e`.
const HAT: u16 = 0x1A;
const E: u16 = 0x12;

/// Text `t` as a piece.
pub fn text(t: &str) -> Piece {
    Piece::Text(Redacted::new(t.into()))
}

#[test]
fn a_letter_space_backspace_and_enter_tell_text_text_erase_and_advance_once_each() {
    let mut r = Rig::new("en-us", |_| ());
    r.char(A).unwrap();
    for id in ["space", "bksp", "enter"] {
        r.tap(id).unwrap();
    }
    let want = [text("a"), text(" "), Piece::Erase, Piece::Advance];
    assert_eq!(r.typing.take_pieces(), want);
    assert!(r.typing.take_pieces().is_empty(), "each is told once");
}

#[test]
fn the_shared_backspace_is_the_key_a_picked_word_erases_with() {
    let erase = BoardSpec::shipped().unwrap().erase_key();
    assert_eq!(erase, Some(Scan::new(BKSP)));
}

#[test]
fn tab_advances_and_every_other_send_key_breaks() {
    let mut r = Rig::new("en-us", |_| ());
    r.tap("tab").unwrap();
    assert_eq!(r.typing.take_pieces(), [Piece::Advance]);
    for id in ["esc", "del", "up", "left", "down", "right", "menu"] {
        r.tap(id).unwrap();
        assert_eq!(r.typing.take_pieces(), [Piece::Break], "{id}");
    }
}

#[test]
fn a_chord_breaks_even_on_a_key_that_types() {
    let mut r = Rig::new("en-us", |_| ());
    for (latch, id) in [("shift", "space"), ("shift", "enter"), ("ctrl", "bksp")] {
        r.tap(latch).unwrap();
        r.tap(id).unwrap();
        assert_eq!(r.typing.take_pieces(), [Piece::Break], "{latch} + {id}");
    }
    r.tap("ctrl").unwrap();
    r.char(C).unwrap();
    assert_eq!(r.typing.take_pieces(), [Piece::Break], "Ctrl+C");
}

#[test]
fn a_dead_key_and_the_key_after_it_break_as_only_the_app_knows_what_they_made() {
    let mut r = Rig::new("fr-fr", |_| ());
    r.char(HAT).unwrap();
    r.char(E).unwrap();
    assert_eq!(r.typing.take_pieces(), [Piece::Break, Piece::Break]);
    r.char(E).unwrap();
    assert_eq!(r.typing.take_pieces(), [text("e")]);
}

#[test]
fn a_send_key_while_a_dead_key_waits_breaks_as_the_app_composes_with_it() {
    let mut r = Rig::new("fr-fr", |_| ());
    for id in ["space", "bksp", "enter", "tab"] {
        r.char(HAT).unwrap();
        r.tap(id).unwrap();
        assert_eq!(
            r.typing.take_pieces(),
            [Piece::Break, Piece::Break],
            "^ then {id}"
        );
    }
    r.tap("space").unwrap();
    assert_eq!(r.typing.take_pieces(), [text(" ")], "no accent waits");
}

#[test]
fn caps_the_latches_side_keys_and_an_empty_key_tell_nothing() {
    let mut r = Rig::new("en-us", |_| ());
    for id in [
        "caps", "shift", "shift", "ctrl", "ctrl", "alt", "alt", "win", "win",
    ] {
        r.tap(id).unwrap();
    }
    for id in ["mic", "rclick", "shortcuts", "power", "lang"] {
        r.tap(id).unwrap();
    }
    assert!(r.typing.take_pieces().is_empty());
    let mut ar = Rig::new("ar-tn", |_| ());
    ar.tap("shift").unwrap();
    ar.char(0x1F).unwrap();
    assert!(
        ar.typing.take_pieces().is_empty(),
        "س types nothing with Shift"
    );
}

#[test]
fn a_send_refused_whole_tells_nothing_and_one_taken_in_part_tells_its_piece() {
    let mut r = Rig::new("en-us", |_| ());
    r.injector
        .set_error(Some(InjectError::Refused { sent: 0, total: 2 }));
    assert!(r.char(A).is_err());
    assert!(r.tap("bksp").is_err());
    assert!(r.typing.take_pieces().is_empty());
    r.injector
        .set_error(Some(InjectError::Refused { sent: 1, total: 2 }));
    assert!(r.char(A).is_err());
    assert_eq!(r.typing.take_pieces(), [text("a")]);
}

/// The keys a random run clicks: letters, the send keys, Caps Lock and the latches without a lone press.
const RUN: [&str; 13] = [
    "c-0x001E", "c-0x0010", "c-0x002E", "space", "bksp", "enter", "tab", "esc", "left", "caps",
    "shift", "ctrl", "alt",
];

/// The piece a stroke tells, or none for Caps Lock: its text, Backspace an erase, the space bar its space, Enter and
/// Tab an advance, and any other key or chord a break.
fn piece_of(stroke: &Stroke) -> Option<Piece> {
    let tap = |code: u16| *stroke == Stroke::Tap(Scan::new(code));
    Some(match stroke {
        _ if tap(CAPS) => return None,
        Stroke::Text(t, _) => Piece::Text(t.clone()),
        _ if tap(BKSP) => Piece::Erase,
        _ if tap(SPACE) => text(" "),
        _ if tap(ENTER) || tap(TAB) => Piece::Advance,
        _ => Piece::Break,
    })
}

proptest! {
    #[test]
    fn every_stroke_that_reached_the_app_tells_its_piece_once_in_order(
        run in proptest::collection::vec(0..RUN.len(), 1..40),
    ) {
        let mut r = Rig::new("en-us", |_| ());
        for &i in &run {
            r.tap(RUN[i]).map_err(|e| TestCaseError::fail(e.to_string()))?;
        }
        let want: Vec<Piece> = r.sent().iter().filter_map(piece_of).collect();
        prop_assert_eq!(r.typing.take_pieces(), want);
    }
}

#[test]
fn a_shortcut_on_the_layer_breaks() {
    let mut r = Rig::new("en-us", |_| ());
    r.shortcuts.set_known(only(&["copy"]));
    r.shortcut(&sid("copy")).unwrap();
    assert_eq!(r.typing.take_pieces(), [Piece::Break]);
}
