//! Controller tests on the product board; nothing here sends keys. Typing's own tests moved to kx-mod-keyboard (P3).

use super::*;
use crate::config::ToolButton;
use kx_mod_keyboard::Latch;

/// A layout handle no system has, so nothing is read for it.
const FAKE: isize = 0x7777_7777;

/// A keyboard on the fake layout.
fn kb() -> Kb {
    let cfg = crate::config::load().expect("spike.toml loads");
    let themes = crate::theme::load().expect("themes.toml loads");
    let mut k = Kb::new(cfg, themes, SystemLook::default()).expect("the board loads");
    k.hkl = FAKE;
    k
}

#[test]
fn it_starts_small_and_steps_through_sizes_and_presets() {
    let mut k = kb();
    let presets = k.limits.presets().to_vec();
    assert!((k.size() - presets[0]).abs() < f64::EPSILON, "starts at S");
    assert!(k.step(true));
    assert!(k.preset(2));
    assert!((k.size() - presets[2]).abs() < f64::EPSILON);
    assert!(!k.preset(9), "no fourth preset");
    assert!(!k.preset(2), "already L");
}

#[test]
fn a_resize_follows_the_pointer_and_ends_at_the_size_it_reached() {
    let mut k = kb();
    let start = k.size();
    let w = k.board().plate.w;
    k.resize_begin((100.0, 100.0));
    assert!(k.resizing());
    assert!(k.resize_at((100.0 + w * 0.25, 100.0)));
    assert!(k.size() > start);
    let reached = k.size();
    k.resize_end();
    assert!((k.size() - reached).abs() < f64::EPSILON && !k.resizing());
    assert!(!k.resize_at((0.0, 0.0)), "nothing after the end");
}

#[test]
fn a_chip_that_does_not_exist_sends_nothing() {
    let mut k = kb();
    assert!(k.chip(usize::MAX).is_err());
}

#[test]
fn side_keys_toggle_or_show_their_note_and_modifiers_latch() {
    let mut k = kb();
    assert_eq!(k.tap("grab", false), Ok(Tapped::Done));
    assert!(k.toggled.contains("grab"));
    k.tap("grab", false).expect("toggle");
    assert!(!k.toggled.contains("grab"), "a second click turns it off");
    let note = k.tap("power", false).expect("note");
    assert!(matches!(note, Tapped::Note(n) if !n.is_empty()));
    k.tap("shift", false).expect("latch");
    assert!(k.typing.latches().is_on(Latch::Shift));
    assert_eq!(k.tap("snip", false), Ok(Tapped::Tool(ToolButton::Snip)));
    assert!(k.tap("nope", false).is_err(), "no such key");
}

#[test]
fn the_mode_switch_flips_what_is_shown_and_themes_must_exist() {
    let mut k = kb();
    k.set_mode(ModeChoice::Auto);
    k.flip_mode();
    assert!(
        k.look().expect("look").dark,
        "Windows is light, so the switch goes dark"
    );
    k.flip_mode();
    assert!(!k.look().expect("look").dark);
    assert!(k.set_theme(2) && !k.set_theme(9));
}

#[test]
fn the_mic_key_shows_the_recording_until_it_stops() {
    let mut k = kb();
    k.set_rec(true);
    assert!(k.state().keys["mic"].rec);
    k.set_rec(false);
    assert!(!k.state().keys["mic"].rec);
}

#[test]
fn the_names_turn_the_way_the_key_stepped_and_only_once() {
    let mut k = kb();
    k.asked = -1;
    k.switched(FAKE);
    assert_eq!(k.state().lang.turn, -1);
    k.switched(FAKE);
    assert_eq!(
        k.state().lang.turn,
        0,
        "a change made another way does not turn"
    );
}

#[test]
fn the_settings_key_opens_the_arabic_panel() {
    let mut k = kb();
    assert_eq!(k.tap("settings", false), Ok(Tapped::Panel));
}
