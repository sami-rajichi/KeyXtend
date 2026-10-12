//! A held key through the face (spec §4.6): a press types at once, a hold adds one at the delay and then one a beat
//! with the latches kept down, a let-go types nothing, and the keys that act once and so start no beat: a shortcut,
//! a dead key, an open list and repeat off.

#![allow(
    clippy::unwrap_used,
    reason = "these fixtures run only in tests, where a failed setup should fail the test"
)]

use crate::host::GOLDEN;
use crate::rig::{HELD, HOLDING, Rig, STATE};
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
fn a_press_types_once_at_once_and_its_let_go_types_nothing() {
    let r = Rig::new(&GOLDEN, None);
    let (delay, _) = times();
    assert_eq!(r.press(A), Ok(HELD), "it holds, so the beat starts");
    assert_eq!(r.fakes.injector.strokes(), [text("a")], "at the press");
    assert_eq!(
        r.tick_in(delay - 1),
        HOLDING,
        "no repeat yet, the beat goes on"
    );
    assert_eq!(r.let_go(A), Ok(STATE), "the burst ends");
    r.tick_in(delay * 2);
    assert_eq!(r.fakes.injector.strokes(), [text("a")]);
}

#[test]
fn a_hold_adds_one_at_the_delay_then_one_a_beat_and_its_let_go_adds_none() {
    let r = Rig::new(&GOLDEN, None);
    let (delay, every) = times();
    let typed = || r.fakes.injector.strokes().len();
    r.press(A).unwrap();
    assert_eq!(r.tick_in(delay), HOLDING, "it holds on and redraws nothing");
    assert_eq!(typed(), 2, "one at the delay");
    assert_eq!(r.tick_in(every / 2), HOLDING, "between beats");
    assert_eq!(typed(), 2);
    assert_eq!(r.tick_in(every / 2), HOLDING);
    assert_eq!(r.tick_in(every), HOLDING);
    assert_eq!(typed(), 4, "one a beat");
    assert_eq!(r.let_go(A), Ok(STATE), "the beat ends");
    r.tick_in(every * 3);
    assert_eq!(r.fakes.injector.strokes(), vec![text("a"); 4]);
}

#[test]
fn a_let_go_of_another_key_leaves_the_hold() {
    let r = Rig::new(&GOLDEN, None);
    let (delay, every) = times();
    r.press(A).unwrap();
    assert_eq!(
        r.let_go("bksp"),
        Ok(FaceReply::default()),
        "not the held key"
    );
    assert_eq!(r.tick_in(delay), HOLDING, "A still repeats");
    assert_eq!(r.let_go(A), Ok(STATE), "its own let-go ends it");
    r.tick_in(every * 2);
    assert_eq!(r.fakes.injector.strokes(), vec![text("a"); 2]);
}

#[test]
fn a_double_click_types_twice_one_per_press() {
    let r = Rig::new(&GOLDEN, None);
    for _ in 0..2 {
        r.press(A).unwrap();
        r.let_go(A).unwrap();
    }
    assert_eq!(r.fakes.injector.strokes(), [text("a"), text("a")]);
}

#[test]
fn with_shift_latched_a_click_types_a_capital_and_shift_lets_go_at_the_let_go() {
    let r = Rig::new(&GOLDEN, None);
    r.tap("shift").unwrap();
    r.press(A).unwrap();
    assert_eq!(r.fakes.injector.strokes(), [text("A")]);
    assert_eq!(r.state()["keys"]["shift"]["on"], true, "lit while held");
    assert_eq!(r.let_go(A), Ok(STATE));
    assert_eq!(r.state()["keys"]["shift"]["on"], false);
}

#[test]
fn with_shift_latched_each_repeat_is_capital_and_shift_lets_go_after_the_burst() {
    let r = Rig::new(&GOLDEN, None);
    r.tap("shift").unwrap();
    hold(&r, A, 1);
    assert_eq!(
        r.tick_in(times().1),
        HOLDING,
        "Shift stays lit, so nothing redraws"
    );
    assert_eq!(r.state()["keys"]["shift"]["on"], true, "down for the burst");
    assert_eq!(r.let_go(A), Ok(STATE));
    assert_eq!(r.state()["keys"]["shift"]["on"], false, "let go after it");
    assert_eq!(r.fakes.injector.strokes(), vec![text("A"); 4]);
}

#[test]
fn with_ctrl_alt_or_win_latched_a_held_key_acts_once_at_the_press() {
    let z = "c-0x002C";
    for (latch, code) in [("ctrl", 0x1D), ("alt", 0x38), ("win", 0xE05B)] {
        let r = Rig::new(&GOLDEN, None);
        r.tap(latch).unwrap();
        assert_eq!(r.press(z), Ok(STATE), "{latch}: no beat starts");
        let once = [Stroke::Chord(vec![Scan::new(code)], Scan::new(0x2C))];
        assert_eq!(r.fakes.injector.strokes(), once, "{latch}");
        assert_eq!(r.state()["keys"][latch]["on"], false, "{latch}");
        r.tick_in(times().0 * 2);
        r.let_go(z).unwrap();
        assert_eq!(r.fakes.injector.strokes(), once, "{latch}: only once");
    }
}

#[test]
fn altgr_acting_as_right_alt_acts_once_at_the_press() {
    // US English has no AltGr characters, so the latch sends Right Alt, which makes a shortcut.
    let r = Rig::new(&GOLDEN, None);
    r.tap("altgr").unwrap();
    r.press(A).unwrap();
    r.tick_in(times().0 * 2);
    r.let_go(A).unwrap();
    let once = Stroke::Chord(vec![Scan::new(0xE038)], Scan::new(0x1E));
    assert_eq!(r.fakes.injector.strokes(), [once]);
    assert_eq!(r.state()["keys"]["altgr"]["on"], false);
}

#[test]
fn a_held_dead_key_types_once_at_the_press() {
    let r = Rig::new(&["fr-fr"], None);
    let hat = "c-0x001A";
    assert!(!r.press(hat).unwrap().holding, "no beat starts");
    assert_eq!(r.fakes.injector.strokes(), [Stroke::Tap(Scan::new(0x1A))]);
    r.tick_in(times().0 * 2);
    r.let_go(hat).unwrap();
    assert_eq!(r.fakes.injector.strokes(), [Stroke::Tap(Scan::new(0x1A))]);
}

#[test]
fn a_key_held_after_a_dead_key_takes_the_accent_at_the_press_then_repeats_its_letter() {
    let r = Rig::new(&["fr-fr"], None);
    r.tap("c-0x001A").unwrap();
    hold(&r, "c-0x0012", 1);
    let hat = Stroke::Tap(Scan::new(0x1A));
    let e = Stroke::Tap(Scan::new(0x12));
    let want = [hat, e, text("e"), text("e")];
    assert_eq!(r.fakes.injector.strokes(), want);
}

#[test]
fn while_the_list_is_open_its_keys_wait_for_the_list_and_a_character_types_once() {
    let r = Rig::new(&GOLDEN, Some(("lang_key", "list".into())));
    r.tap("lang").unwrap();
    let hi = r.state()["list"]["hi"].clone();
    assert_eq!(r.press("down"), Ok(FaceReply::default()));
    r.tick_in(times().0 * 2);
    r.let_go("down").unwrap();
    assert!(r.fakes.injector.strokes().is_empty(), "never to the app");
    assert_eq!(r.state()["list"]["hi"], hi, "nor into the list");
    r.panel("down", 5).unwrap();
    assert_ne!(r.state()["list"]["hi"], hi, "the UI sends it there");
    hold(&r, A, 3);
    r.let_go(A).unwrap();
    assert_eq!(r.fakes.injector.strokes(), [text("a")], "a character once");
}

#[test]
fn with_repeat_off_a_press_types_once() {
    let r = Rig::new(&GOLDEN, Some(("repeat", false.into())));
    assert_eq!(r.press(A), Ok(STATE), "no beat starts");
    assert_eq!(r.fakes.injector.strokes(), [text("a")]);
    r.tick_in(times().0 * 2);
    r.let_go(A).unwrap();
    assert_eq!(r.fakes.injector.strokes(), [text("a")]);
}

#[test]
fn a_key_that_never_repeats_types_once_at_the_press() {
    for (id, code) in [("esc", 0x01), ("tab", 0x0F), ("menu", 0xE05D)] {
        let r = Rig::new(&GOLDEN, None);
        assert_eq!(r.press(id), Ok(STATE), "{id}: no beat starts");
        let once = [Stroke::Tap(Scan::new(code))];
        assert_eq!(r.fakes.injector.strokes(), once, "{id}");
        r.tick_in(times().0 * 2);
        r.let_go(id).unwrap();
        assert_eq!(r.fakes.injector.strokes(), once, "{id}");
    }
}

#[test]
fn french_and_arabic_repeat_their_own_characters_altgr_ones_too() {
    let r = Rig::new(&["fr-fr"], None);
    r.tap("altgr").unwrap();
    hold(&r, "c-0x0004", 1);
    r.let_go("c-0x0004").unwrap();
    assert_eq!(r.fakes.injector.strokes(), vec![text("#"); 3]);
    assert_eq!(r.state()["keys"]["altgr"]["on"], false, "after the burst");
    let r = Rig::new(&["ar-tn"], None);
    let table = LayoutTable::from(&LayoutFile::golden("ar-tn").unwrap());
    let dad = &table.keys[&Scan::new(0x10)];
    hold(&r, "c-0x0010", 0);
    r.let_go("c-0x0010").unwrap();
    r.tap("altgr").unwrap();
    hold(&r, "c-0x0010", 0);
    r.let_go("c-0x0010").unwrap();
    let (base, altgr) = (text(&dad.base), text(&dad.altgr));
    let want = [base.clone(), base, altgr.clone(), altgr];
    assert_eq!(r.fakes.injector.strokes(), want);
}
