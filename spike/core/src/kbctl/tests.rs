//! Controller tests on a made-up layout; nothing here sends keys.

use super::*;
use crate::config::ToolButton;
use crate::facecfg::Action;
use crate::latch::Latch;
use crate::legend::testkeys::{dead, letter};

/// A layout handle no system has, so the cache is the only source.
const FAKE: isize = 0x7777_7777;

/// A keyboard on the fake layout: q and e, with € on AltGr+E when `altgr`, and a dead ^ on the W key.
fn kb(altgr: bool) -> Kb {
    let cfg = crate::config::load().expect("spike.toml loads");
    let themes = crate::theme::load().expect("themes.toml loads");
    let mut k = Kb::new(cfg, themes, SystemLook::default());
    let euro = if altgr { "€" } else { "" };
    k.hkl = FAKE;
    k.chars.insert(
        FAKE,
        vec![
            (0x10, letter("q", "Q")),
            (0x11, dead("^")),
            (
                0x12,
                KeyChars {
                    altgr: euro.into(),
                    ..letter("e", "E")
                },
            ),
        ],
    );
    k
}

#[test]
fn it_starts_small_and_steps_through_sizes_and_presets() {
    let mut k = kb(false);
    assert!(
        (k.size() - k.cfg().size.presets[0]).abs() < f32::EPSILON,
        "starts at S"
    );
    assert!(k.step(true));
    assert!(k.preset(2));
    assert!((k.size() - k.cfg().size.presets[2]).abs() < f32::EPSILON);
    assert!(!k.preset(9), "no fourth preset");
    assert!(!k.preset(2), "already L");
}

#[test]
fn a_resize_follows_the_pointer_and_ends_at_the_size_it_reached() {
    let mut k = kb(false);
    let start = k.size();
    let w = k.board().plate.w;
    k.resize_begin((100.0, 100.0));
    assert!(k.resizing());
    assert!(k.resize_at((100.0 + w * 0.25, 100.0)));
    assert!(k.size() > start);
    let reached = k.size();
    k.resize_end();
    assert!((k.size() - reached).abs() < f32::EPSILON && !k.resizing());
    assert!(!k.resize_at((0.0, 0.0)), "nothing after the end");
}

#[test]
fn named_keys_send_latch_or_toggle_as_configured() {
    let mut k = kb(false);
    assert_eq!(k.plan("esc", false, false), Plan::Chord(0x01));
    assert_eq!(
        k.plan("rshift", false, false),
        Plan::Latch(Latch::Shift, 0x36)
    );
    assert_eq!(k.plan("caps", false, false), Plan::Caps(0x3A));
    assert_eq!(k.plan("lang", false, true), Plan::Lang(true));
    assert_eq!(k.plan("mic", false, false), Plan::Side(Action::Mic));
    assert_eq!(k.plan("nope", false, false), Plan::Nothing);
}

#[test]
fn a_character_key_types_its_text_or_a_shortcut() {
    let mut k = kb(true);
    assert_eq!(k.plan("c-1-0", false, false), Plan::Text("q".into()));
    assert_eq!(
        k.plan("c-1-0", true, false),
        Plan::Text("Q".into()),
        "Caps Lock"
    );
    k.latches.toggle(Latch::AltGr, 0xE038);
    assert_eq!(k.plan("c-1-2", false, false), Plan::Text("€".into()));
    k.latches.take();
    k.latches.toggle(Latch::Ctrl, 0x1D);
    assert_eq!(
        k.plan("c-1-0", false, false),
        Plan::Chord(0x10),
        "Ctrl+Q is a shortcut"
    );
}

#[test]
fn a_dead_key_and_the_key_after_it_go_as_key_presses_so_the_app_adds_the_accent() {
    let mut k = kb(false);
    let hat = k.plan("c-1-1", false, false);
    assert_eq!(hat, Plan::Dead(0x11, vec![]));
    k.mark(&hat);
    let e = k.plan("c-1-2", false, false);
    assert_eq!(e, Plan::Press(0x12, vec![]), "the layout makes ê");
    k.mark(&e);
    assert_eq!(k.plan("c-1-2", false, false), Plan::Text("e".into()));
}

#[test]
fn a_latch_between_a_dead_key_and_its_letter_keeps_the_accent_waiting() {
    let mut k = kb(false);
    k.mark(&Plan::Dead(0x11, vec![]));
    k.mark(&Plan::Latch(Latch::Shift, 0x2A));
    assert_eq!(
        k.plan("c-1-2", false, false),
        Plan::Press(0x12, vec![]),
        "Ê"
    );
}

#[test]
fn an_arrow_keeps_the_accent_waiting_and_space_ends_it() {
    // Windows' own layout says which keys type something, so this uses the real one.
    let mut k = kb(false);
    k.hkl = layout::foreground_layout().0 as isize;
    k.mark(&Plan::Dead(0x11, vec![]));
    k.mark(&Plan::Chord(0xE04B));
    assert!(k.dead, "Left types nothing");
    k.mark(&Plan::Chord(0x39));
    assert!(!k.dead, "Space types the accent");
}

#[test]
fn a_shift_the_dead_key_ignores_is_released_but_not_held() {
    let mut k = kb(false);
    k.latches.toggle(Latch::Shift, 0x2A);
    assert_eq!(
        k.plan("c-1-1", false, false),
        Plan::Dead(0x11, vec![Latch::Shift])
    );
}

#[test]
fn a_latch_the_legend_ignores_is_not_held_around_a_dead_key_or_its_letter() {
    let mut k = kb(true);
    k.latches.toggle(Latch::AltGr, 0xE038);
    let hat = k.plan("c-1-1", false, false);
    assert_eq!(
        hat,
        Plan::Dead(0x11, vec![Latch::AltGr]),
        "^ has no AltGr character"
    );
    k.mark(&hat);
    k.latches.toggle(Latch::Shift, 0x2A);
    let q = k.plan("c-1-0", false, false);
    assert_eq!(
        q,
        Plan::Press(0x10, vec![Latch::AltGr]),
        "Shift is kept for Q"
    );
}

#[test]
fn a_chip_that_does_not_exist_sends_nothing() {
    let mut k = kb(false);
    assert!(k.chip(usize::MAX).is_err());
}

#[test]
fn altgr_without_altgr_characters_is_right_alt() {
    let mut k = kb(false);
    k.latches.toggle(Latch::AltGr, 0xE038);
    assert_eq!(k.plan("c-1-0", false, false), Plan::Chord(0x10));
}

#[test]
fn side_keys_toggle_or_show_their_note_and_modifiers_latch() {
    let mut k = kb(false);
    assert_eq!(k.run("grab", Plan::Side(Action::Toggle)), Ok(Tapped::Done));
    assert!(k.toggled.contains("grab"));
    k.run("grab", Plan::Side(Action::Toggle)).expect("toggle");
    assert!(!k.toggled.contains("grab"), "a second click turns it off");
    let note = k.run("power", Plan::Side(Action::Note)).expect("note");
    assert!(matches!(note, Tapped::Note(n) if !n.is_empty()));
    k.run("shift", Plan::Latch(Latch::Shift, 0x2A))
        .expect("latch");
    assert!(k.latches.is_on(Latch::Shift));
    let snip = k.run("snip", Plan::Side(Action::Snip));
    assert_eq!(snip, Ok(Tapped::Tool(ToolButton::Snip)));
}

#[test]
fn the_mode_switch_flips_what_is_shown_and_themes_must_exist() {
    let mut k = kb(false);
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
