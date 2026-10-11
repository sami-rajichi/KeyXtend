//! A held key through the face (spec §4.6): a click types once on let-go, a hold types at the delay and then once a
//! beat with the latches kept down, and the keys that act once: a shortcut, a dead key, an open list and repeat off.

#![allow(
    clippy::unwrap_used,
    reason = "these fixtures run only in tests, where a failed setup should fail the test"
)]

use crate::host::GOLDEN;
use crate::rig::{Rig, STATE};
use kx_mod_keyboard::KeyboardSettings;
use kx_module_api::Redacted;
use kx_module_api::ports::{FaceReply, LayoutTable, Scan, TextRoute};
use kx_platform_fake::{LayoutFile, Stroke};

/// The character key at US "a".
pub const A: &str = "c-0x001E";

/// The default hold before the first repeat, and the time between repeats, in ms.
pub fn times() -> (u64, u64) {
    let s = KeyboardSettings::from_defaults().unwrap();
    (s.repeat_delay_ms, s.repeat_every_ms)
}

/// `t` sent as Unicode text.
pub fn text(t: &str) -> Stroke {
    Stroke::Text(Redacted::new(t.into()), TextRoute::Unicode)
}

/// Holds key `id` through the delay and `beats` more beats, without letting go.
pub fn hold(r: &Rig, id: &str, beats: u64) {
    let (delay, every) = times();
    r.press(id).unwrap();
    r.tick_in(delay);
    for _ in 0..beats {
        r.tick_in(every);
    }
}

#[test]
fn a_short_click_types_once_on_let_go() {
    let r = Rig::new(&GOLDEN, None);
    let (delay, _) = times();
    assert_eq!(r.press(A), Ok(FaceReply::default()));
    assert_eq!(r.tick_in(delay - 1), FaceReply::default());
    assert!(r.fakes.injector.strokes().is_empty(), "nothing while held");
    assert_eq!(r.let_go(A), Ok(STATE));
    r.tick_in(delay * 2);
    assert_eq!(r.fakes.injector.strokes(), [text("a")]);
}

#[test]
fn a_hold_types_at_the_delay_then_once_a_beat_and_its_let_go_adds_none() {
    let r = Rig::new(&GOLDEN, None);
    let (delay, every) = times();
    r.press(A).unwrap();
    assert_eq!(r.tick_in(delay), STATE);
    assert_eq!(r.tick_in(every / 2), FaceReply::default(), "between beats");
    r.tick_in(every / 2);
    r.tick_in(every);
    r.let_go(A).unwrap();
    r.tick_in(every * 3);
    assert_eq!(
        r.fakes.injector.strokes(),
        [text("a"), text("a"), text("a")]
    );
}

#[test]
fn with_shift_latched_each_repeat_is_capital_and_shift_lets_go_after_the_burst() {
    let r = Rig::new(&GOLDEN, None);
    r.tap("shift").unwrap();
    hold(&r, A, 2);
    assert_eq!(r.state()["keys"]["shift"]["on"], true, "down for the burst");
    assert_eq!(r.let_go(A), Ok(STATE));
    assert_eq!(r.state()["keys"]["shift"]["on"], false, "let go after it");
    assert_eq!(
        r.fakes.injector.strokes(),
        [text("A"), text("A"), text("A")]
    );
}

#[test]
fn with_ctrl_alt_or_win_latched_a_held_key_acts_once() {
    let z = "c-0x002C";
    for (latch, code) in [("ctrl", 0x1D), ("alt", 0x38), ("win", 0xE05B)] {
        let r = Rig::new(&GOLDEN, None);
        r.tap(latch).unwrap();
        hold(&r, z, 5);
        assert!(
            r.fakes.injector.strokes().is_empty(),
            "{latch}: not while held"
        );
        r.let_go(z).unwrap();
        let once = Stroke::Chord(vec![Scan::new(code)], Scan::new(0x2C));
        assert_eq!(r.fakes.injector.strokes(), [once], "{latch}");
        assert_eq!(r.state()["keys"][latch]["on"], false, "{latch}");
    }
}

#[test]
fn a_held_dead_key_types_once() {
    let r = Rig::new(&["fr-fr"], None);
    let hat = "c-0x001A";
    hold(&r, hat, 3);
    assert!(r.fakes.injector.strokes().is_empty());
    r.let_go(hat).unwrap();
    assert_eq!(r.fakes.injector.strokes(), [Stroke::Tap(Scan::new(0x1A))]);
}

#[test]
fn a_key_held_after_a_dead_key_takes_the_accent_once_then_repeats_its_letter() {
    let r = Rig::new(&["fr-fr"], None);
    r.tap("c-0x001A").unwrap();
    hold(&r, "c-0x0012", 2);
    let hat = Stroke::Tap(Scan::new(0x1A));
    let e = Stroke::Tap(Scan::new(0x12));
    let want = [hat, e, text("e"), text("e")];
    assert_eq!(r.fakes.injector.strokes(), want);
}

#[test]
fn while_the_list_is_open_a_held_key_acts_once_and_its_keys_go_to_the_list() {
    let r = Rig::new(&GOLDEN, Some(("lang_key", "list".into())));
    r.tap("lang").unwrap();
    let hi = r.state()["list"]["hi"].clone();
    hold(&r, "down", 3);
    r.let_go("down").unwrap();
    assert!(r.fakes.injector.strokes().is_empty(), "never to the app");
    assert_eq!(r.state()["list"]["hi"], hi, "nor into the list");
    r.panel("down", 5).unwrap();
    assert_ne!(
        r.state()["list"]["hi"],
        hi,
        "the UI sends it there on let-go"
    );
    hold(&r, A, 3);
    r.let_go(A).unwrap();
    assert_eq!(r.fakes.injector.strokes(), [text("a")], "a character once");
}

#[test]
fn with_repeat_off_a_hold_is_a_click() {
    let r = Rig::new(&GOLDEN, Some(("repeat", false.into())));
    hold(&r, A, 5);
    assert!(r.fakes.injector.strokes().is_empty());
    r.let_go(A).unwrap();
    assert_eq!(r.fakes.injector.strokes(), [text("a")]);
}

#[test]
fn a_key_that_never_repeats_types_once_on_let_go() {
    for (id, code) in [("esc", 0x01), ("tab", 0x0F), ("menu", 0xE05D)] {
        let r = Rig::new(&GOLDEN, None);
        hold(&r, id, 3);
        r.let_go(id).unwrap();
        assert_eq!(
            r.fakes.injector.strokes(),
            [Stroke::Tap(Scan::new(code))],
            "{id}"
        );
    }
}

#[test]
fn french_and_arabic_repeat_their_own_characters_altgr_ones_too() {
    let r = Rig::new(&["fr-fr"], None);
    r.tap("altgr").unwrap();
    hold(&r, "c-0x0004", 1);
    r.let_go("c-0x0004").unwrap();
    assert_eq!(r.fakes.injector.strokes(), [text("#"), text("#")]);
    assert_eq!(r.state()["keys"]["altgr"]["on"], false, "after the burst");
    let r = Rig::new(&["ar-tn"], None);
    let table = LayoutTable::from(&LayoutFile::golden("ar-tn").unwrap());
    let dad = &table.keys[&Scan::new(0x10)];
    hold(&r, "c-0x0010", 1);
    r.let_go("c-0x0010").unwrap();
    r.tap("altgr").unwrap();
    hold(&r, "c-0x0010", 1);
    r.let_go("c-0x0010").unwrap();
    let (base, altgr) = (text(&dad.base), text(&dad.altgr));
    let want = [base.clone(), base, altgr.clone(), altgr];
    assert_eq!(r.fakes.injector.strokes(), want);
}
