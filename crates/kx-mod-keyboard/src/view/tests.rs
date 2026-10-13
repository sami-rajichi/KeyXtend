use super::*;
use crate::geom::board;
use crate::latch::Latches;
use crate::panel::LangList;
use crate::settings::{KeyboardSettings, Modifiers};
use crate::spec::{BoardSpec, Latch, Nav};
use kx_module_api::ports::{AssistChanged, KeyChars, LangLabel, LayoutId, LayoutTable, Scan};
use std::collections::{BTreeMap, HashSet};

fn spec() -> BoardSpec {
    BoardSpec::shipped().unwrap()
}

fn settings() -> KeyboardSettings {
    KeyboardSettings::from_defaults().unwrap()
}

fn key<'a>(v: &'a BoardView, id: &str) -> &'a KeyView {
    v.keys.iter().find(|k| k.id == id).unwrap()
}

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
fn side_keys_and_the_dpad_are_dimmed_and_say_they_come_later() {
    let v = board_view(&spec(), &settings(), 1.0);
    let mic = key(&v, "mic");
    assert!(mic.dim && mic.side);
    assert_eq!(mic.tip, Some(LATER_TIP));
    assert!(v.dpad.dim);
    assert_eq!((v.dpad.name, v.dpad.tip), (DPAD_NAME, Some(LATER_TIP)));
    let esc = key(&v, "esc");
    assert!(!esc.dim && esc.tip.is_none());
}

#[test]
fn only_modifiers_caps_lock_and_the_mouse_assist_keys_are_checkable() {
    let v = board_view(&spec(), &settings(), 1.0);
    let checkable: Vec<&str> = v
        .keys
        .iter()
        .filter(|k| k.checkable)
        .map(|k| k.id.as_str())
        .collect();
    let want = ["caps", "shift", "rshift", "ctrl", "win", "alt", "altgr"];
    assert_eq!(checkable, [want.as_slice(), &["rclick", "grab"]].concat());
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

/// A letter with `altgr` on `AltGr`: Caps Lock swaps its cases.
fn letter(base: &str, shift: &str, altgr: &str) -> KeyChars {
    KeyChars {
        base: base.into(),
        shift: shift.into(),
        altgr: altgr.into(),
        caps: shift.into(),
        caps_shift: base.into(),
        dead: Vec::new(),
    }
}

/// A tiny layout: 1 and !, e with € on `AltGr`, and x on every other key.
fn table() -> LayoutTable {
    let chars = |sc: Scan| match sc.code() {
        0x02 => KeyChars {
            caps: "1".into(),
            caps_shift: "!".into(),
            ..letter("1", "!", "")
        },
        0x12 => letter("e", "E", "€"),
        _ => letter("x", "X", ""),
    };
    let keys: BTreeMap<Scan, KeyChars> = spec()
        .char_scans()
        .into_iter()
        .map(|sc| (sc, chars(sc)))
        .collect();
    let label = LangLabel {
        id: LayoutId::new(1),
        short: "EN".into(),
        name: vec!["English".into()],
        ..LangLabel::default()
    };
    LayoutTable { label, keys }
}

fn latched(l: Latch, code: u16) -> Latches {
    let mut m = Latches::new(Modifiers::Release);
    m.toggle(l, Scan::new(code));
    m
}

fn state(latches: &Latches, caps: bool, open: Option<&LangList>) -> StateView {
    state_of(latches, caps, open, UNHEARD)
}

/// The state with the mouse module in state `assist`.
fn state_of(
    latches: &Latches,
    caps: bool,
    open: Option<&LangList>,
    assist: AssistChanged,
) -> StateView {
    let keys = board(&spec(), 0.0, 1.0).keys;
    let table = table();
    let list = [table.label.clone()];
    let panel_keys = [("esc".to_owned(), Nav::Esc)];
    state_view(&StateInput {
        keys: &keys,
        latches,
        caps,
        table: &table,
        list: &list,
        turn: 0,
        open,
        panel_keys: &panel_keys,
        at: None,
        assist,
    })
}

fn none() -> Latches {
    Latches::new(Modifiers::Release)
}

#[test]
fn a_latched_shift_lights_both_shift_keys_and_shifts_the_legends() {
    let s = state(&latched(Latch::Shift, 0x2A), false, None);
    assert!(s.keys["shift"].on && s.keys["rshift"].on);
    assert_eq!(s.keys["shift"].paint.fill, "lock");
    assert_eq!(s.keys["c-0x0002"].legend.main, "!", "the digit row shifts");
    assert_eq!(s.keys["c-0x0012"].name, "E");
}

#[test]
fn caps_lights_its_key_and_capitals_the_letters() {
    let s = state(&none(), true, None);
    assert!(s.keys["caps"].on);
    assert_eq!(s.keys["c-0x0012"].legend.main, "E");
    assert!(!s.keys["shift"].on);
}

#[test]
fn altgr_counts_when_the_layout_has_altgr_characters() {
    let s = state(&latched(Latch::AltGr, 0xE038), false, None);
    assert_eq!(s.keys["c-0x0012"].legend.main, "€");
    assert_eq!(s.lang.space, ["English"]);
}

#[test]
fn a_key_that_types_nothing_with_altgr_shows_nothing_and_is_named_empty() {
    let s = state(&latched(Latch::AltGr, 0xE038), false, None);
    let x = &s.keys["c-0x0010"];
    assert_eq!((x.legend.main.as_str(), x.name.as_str()), ("", ""));
    assert_eq!(x.blank, Some(EMPTY_NAME));
    assert_eq!(s.keys["c-0x0012"].blank, None, "it types €");
    let plain = state(&none(), false, None);
    assert!(plain.keys.values().all(|k| k.blank.is_none()));
}

#[test]
fn dimmed_keys_paint_dim_and_named_keys_have_no_legend() {
    let s = state(&none(), false, None);
    assert_eq!(s.keys["mic"].paint, DIM);
    assert!(!s.keys["mic"].on);
    assert_eq!(s.keys["esc"].legend, Legend::default());
    assert_eq!(s.keys["esc"].paint.fill, "danger");
}

#[test]
fn with_one_layout_the_language_key_says_so_and_no_other_key_does() {
    let s = state(&none(), false, None);
    assert_eq!(s.lang.count, 1, "the case this test is for");
    assert_eq!(s.keys["lang"].tip, Some(LONE_TIP));
    let others = s.keys.iter().filter(|(id, _)| *id != "lang");
    let mouse = |k: &KeyState| k.tip == Some(MOUSE_OFF_TIP);
    assert!(others.into_iter().all(|(_, k)| k.tip.is_none() || mouse(k)));
}

#[test]
fn the_list_shows_only_while_it_is_open() {
    assert!(state(&none(), false, None).list.is_none());
    let open = LangList::open(0);
    let list = state(&none(), false, Some(&open)).list.unwrap();
    assert_eq!((list.rows.len(), list.hi, list.keys.len()), (1, 0, 1));
}

#[test]
fn key_and_state_debug_hide_legends_lights_and_caps() {
    let lit = KeyState {
        legend: Legend {
            main: "Q".into(),
            ..Legend::default()
        },
        name: "Q".into(),
        on: true,
        ..KeyState::default()
    };
    assert_eq!(format!("{lit:?}"), "KeyState { .. }");
    let state = StateView {
        keys: BTreeMap::from([("caps".to_owned(), lit)]),
        ..StateView::default()
    };
    let shown = format!("{state:?}");
    for leak in ["caps", "Q", "true", "keys"] {
        assert!(!shown.contains(leak), "{shown}");
    }
    assert!(
        shown.starts_with("StateView { lang: ") && shown.ends_with(", .. }"),
        "{shown}"
    );
}

mod assist;
