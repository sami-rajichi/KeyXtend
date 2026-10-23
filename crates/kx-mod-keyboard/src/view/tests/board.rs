//! The board as the UI draws it: cells, names and icons, dimmed, checkable and repeating keys, and its rules.

use super::*;

#[test]
fn every_key_is_a_cell_the_ui_insets_by_the_themes_gap() {
    let v = board_view(&spec(), &settings(), 1.25);
    let b = board(&spec(), 0.0, 1.25);
    assert_eq!(v.keys.len(), b.keys.len());
    for (view, cell) in v.keys.iter().zip(&b.keys) {
        assert_eq!((view.id.as_str(), view.cell), (cell.id.as_str(), cell.cell));
    }
    assert_eq!((v.plate, v.bar, v.dpad.area), (b.plate, b.bar, b.dpad));
    assert!(v.keys.iter().any(|k| k.id == "c-0x0056"), "the ISO key");
}

#[test]
fn named_keys_carry_translation_and_icon_keys_and_character_keys_none() {
    let v = board_view(&spec(), &settings(), 1.0);
    let esc = key(&v, "esc");
    assert_eq!(esc.name.as_deref(), Some("key.esc"));
    assert_eq!(esc.label.as_deref(), Some("label.esc"));
    assert_eq!(esc.icon.as_deref(), Some("keys.esc"));
    let q = key(&v, "c-0x0010");
    assert!(q.name.is_none() && q.label.is_none() && q.icon.is_none());
    let space = key(&v, "space");
    assert_eq!(space.name.as_deref(), Some("key.space"));
    assert!(
        space.label.is_none() && space.icon.is_none(),
        "it shows the language"
    );
}

#[test]
fn later_side_keys_are_dimmed_and_say_they_come_later() {
    let v = board_view(&spec(), &settings(), 1.0);
    let mic = key(&v, "mic");
    assert!(mic.dim && mic.side);
    assert_eq!(mic.tip, Some(LATER_TIP));
    let esc = key(&v, "esc");
    assert!(!esc.dim && esc.tip.is_none());
}

#[test]
fn only_modifiers_caps_lock_the_mouse_assist_keys_and_the_shortcuts_key_are_checkable() {
    let v = board_view(&spec(), &settings(), 1.0);
    let checkable: Vec<&str> = v
        .keys
        .iter()
        .filter(|k| k.checkable)
        .map(|k| k.id.as_str())
        .collect();
    let want = ["caps", "shift", "rshift", "ctrl", "win", "alt", "altgr"];
    let side = ["rclick", "grab", "shortcuts"];
    assert_eq!(checkable, [want.as_slice(), &side].concat());
}

#[test]
fn the_hidden_layer_leaves_the_board_as_it_is_and_no_key_names_steps() {
    let v = board_view(&spec(), &settings(), 1.0);
    assert!(!v.layer);
    assert!(v.keys.iter().all(|k| !k.icon_only && k.steps.is_empty()));
    for id in ["shortcuts", "power"] {
        let k = key(&v, id);
        assert!(
            !k.dim && k.tip.is_none() && !k.repeat,
            "{id}: its state dims it"
        );
    }
}

#[test]
fn the_language_keys_icon_follows_its_design() {
    let mut s = settings();
    let icon = |s: &KeyboardSettings| key(&board_view(&spec(), s, 1.0), "lang").icon.clone();
    assert_eq!(icon(&s), None, "the Carousel shows names");
    s.lang_key = LangKey::Cycle;
    assert_eq!(icon(&s).as_deref(), Some(CYCLE_ICON));
    s.lang_key = LangKey::List;
    assert_eq!(icon(&s).as_deref(), Some(LIST_ICON));
}

#[test]
fn the_board_carries_the_size_rules_the_relabel_time_and_the_bubble_corner() {
    let s = settings();
    let v = board_view(&spec(), &s, 1.25);
    assert_eq!((v.size, v.preset), (1.25, Some(1)));
    assert_eq!(v.presets, s.presets);
    assert_eq!((v.relabel_ms, v.lang_key), (s.relabel_ms, s.lang_key));
    assert_eq!(v.bubble_corner, s.bubble_corner);
}

#[test]
fn the_board_marks_the_keys_that_repeat_and_carries_the_hold_times() {
    let s = settings();
    let v = board_view(&spec(), &s, 1.0);
    let marked: HashSet<String> = v
        .keys
        .iter()
        .filter(|k| k.repeat)
        .map(|k| k.id.clone())
        .collect();
    assert_eq!(marked, spec().repeating());
    let times = (v.repeat_delay_ms, v.repeat_every_ms);
    assert_eq!(times, (s.repeat_delay_ms, s.repeat_every_ms));
    let off = KeyboardSettings { repeat: false, ..s };
    let v = board_view(&spec(), &off, 1.0);
    assert!(
        v.keys.iter().all(|k| !k.repeat),
        "repeat off: every key clicks"
    );
}
