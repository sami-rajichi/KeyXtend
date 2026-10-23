use super::*;
use crate::geom::board;
use crate::latch::Latches;
use crate::panel::LangList;
use crate::settings::{KeyboardSettings, Modifiers};
use crate::spec::{BoardSpec, Latch, Nav};
use kx_module_api::ports::{
    AssistChanged, KeyChars, LangLabel, LayoutId, LayoutTable, Scan, ScrollChanged, ShortcutId,
};
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
    state_of(latches, caps, open, AssistChanged::OFF, ScrollChanged::OFF)
}

/// The owner's pages, every shortcut known, with the layer hidden.
fn owners_layer() -> Layer {
    let pages = settings().shortcut_pages;
    let known: HashSet<ShortcutId> = pages.iter().flatten().cloned().collect();
    Layer::new(spec().layer_grid(), &pages, &known)
}

/// What the state is made from, but the latches, Caps Lock and the open list.
struct Others<'a> {
    assist: AssistChanged,
    scroll: ScrollChanged,
    layer: &'a Layer,
    power: PowerChanged,
}

/// The state with the mouse module in state `assist` and the scroll module in state `scroll`.
fn state_of(
    latches: &Latches,
    caps: bool,
    open: Option<&LangList>,
    assist: AssistChanged,
    scroll: ScrollChanged,
) -> StateView {
    let layer = owners_layer();
    let power = PowerChanged::OFF;
    let others = Others {
        assist,
        scroll,
        layer: &layer,
        power,
    };
    state_with(latches, caps, open, &others)
}

/// The state with `others`.
fn state_with(
    latches: &Latches,
    caps: bool,
    open: Option<&LangList>,
    others: &Others<'_>,
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
        rows: 1,
        panel_keys: &panel_keys,
        at: None,
        assist: others.assist,
        scroll: others.scroll,
        layer: others.layer,
        power: others.power,
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
    let mouse =
        |id: &str, k: &KeyState| ["rclick", "grab"].contains(&id) && k.tip == Some(MOUSE_OFF_TIP);
    let power = |id: &str, k: &KeyState| id == "power" && k.tip == Some(POWER_OFF_TIP);
    assert!(
        others
            .into_iter()
            .all(|(id, k)| k.tip.is_none() || mouse(id, k) || power(id, k))
    );
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
mod board;
mod layer;
mod pad;
mod power;
