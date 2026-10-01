//! What the UI draws, as the face's JSON: the board (boxes, names, dimmed keys and the size), which changes with
//! the size, and the state (legends, lights, the language key, the list and the place), which changes as keys are used.

mod paint;

pub use paint::{DIM, Paint, paint};

use crate::geom::{Area, KeyBox, board};
use crate::langkey::{LangView, lang_view, lone};
use crate::latch::Latches;
use crate::legend::{Legend, LegendMods, legend, mark_names, shown, typed};
use crate::panel::{LangList, ListView, list_view};
use crate::settings::{Corner, KeyboardSettings, LangKey};
use crate::sizer::Limits;
use crate::spec::{BoardSpec, KeyKind, Nav, Role};
use kx_module_api::ports::{LangLabel, LayoutTable, Point};
use serde::Serialize;
use std::collections::BTreeMap;

/// Starts a named key's name, a translation key such as `key.esc`.
pub const NAME_PREFIX: &str = "key.";
/// Starts a named key's text label, a translation key such as `label.esc`, shown when its look has no icon.
pub const LABEL_PREFIX: &str = "label.";
/// Starts a named key's icon, an `icons.toml` key such as `keys.esc`.
pub const ICON_PREFIX: &str = "keys.";
/// The tip of every dimmed part: "Coming in a later version".
pub const LATER_TIP: &str = "tip.later";
/// The language key's tip while it has nothing to step to: "Only one keyboard language is installed".
pub const LONE_TIP: &str = "tip.one-lang";
/// The D-pad's name.
pub const DPAD_NAME: &str = "key.dpad";
/// The Cycle design's globe, by its `icons.toml` key.
pub const CYCLE_ICON: &str = "lang.cycle";
/// The List design's chevron, by its `icons.toml` key.
pub const LIST_ICON: &str = "lang.list";

/// One key as the board draws it; it changes only with the size or the language key's design.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct KeyView {
    /// Id, which keys its state and its clicks.
    pub id: String,
    /// Kind, which sets its colours.
    pub kind: KeyKind,
    /// Main row from 0 at the top, or side-block row.
    pub row: usize,
    /// In the side block.
    pub side: bool,
    /// The cell in logical pixels; the cap is the cell less half the theme's `gap_px` (times the size) on each side.
    #[serde(flatten)]
    pub cell: Area,
    /// Its icon by `icons.toml` key; the UI shows `label` when the look has none.
    pub icon: Option<String>,
    /// Its text label, by translation key.
    pub label: Option<String>,
    /// Its name for screen readers and tips, by translation key; a character key's comes with the state.
    pub name: Option<String>,
    /// It latches or locks, so screen readers hear it as checkable.
    pub checkable: bool,
    /// It comes in a later version: drawn dimmed and does nothing.
    pub dim: bool,
    /// Its tip by translation key, for a dimmed key.
    pub tip: Option<&'static str>,
}

/// The D-pad's round face.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct DpadView {
    /// Its box in logical pixels, inset like a key's cell.
    #[serde(flatten)]
    pub area: Area,
    /// Scrolling comes in a later version.
    pub dim: bool,
    /// Its name, by translation key.
    pub name: &'static str,
    /// Its tip while dimmed, by translation key.
    pub tip: Option<&'static str>,
}

/// The keyboard at one size.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct BoardView {
    /// The size factor.
    pub size: f64,
    /// The preset the size is, if any.
    pub preset: Option<usize>,
    /// The preset sizes, one S, M, L button each.
    pub presets: Vec<f64>,
    /// How often the UI sends `relabel`, in ms.
    pub relabel_ms: u64,
    /// The language key's design.
    pub lang_key: LangKey,
    /// The bottom corner the minimise bubble goes to.
    pub bubble_corner: Corner,
    /// The plate, which is the window.
    pub plate: Area,
    /// The top bar.
    pub bar: Area,
    /// The D-pad.
    pub dpad: DpadView,
    /// Every key: main rows left to right, then the side block.
    pub keys: Vec<KeyView>,
}

/// One key's changing part.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize)]
pub struct KeyState {
    /// Legends; empty for named keys.
    #[serde(flatten)]
    pub legend: Legend,
    /// What a character key types now, for screen readers, marks on a dotted circle; empty for named keys.
    pub name: String,
    /// The names of the marks it types, by translation key, which readers hear instead of `name`; often none.
    pub marks: Vec<String>,
    /// Latched, or Caps Lock while it is on.
    pub on: bool,
    /// Its colours now, by palette token.
    pub paint: Paint,
    /// What it says after its name now, by translation key, shown in its tip; the language key's `LONE_TIP`.
    pub tip: Option<&'static str>,
}

/// Everything that changes as keys are used.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize)]
pub struct StateView {
    /// Each key's state, by id.
    pub keys: BTreeMap<String, KeyState>,
    /// The language key and the space bar.
    pub lang: LangView,
    /// The language list while it is open.
    pub list: Option<ListView>,
    /// Where the window goes, in physical pixels, once the UI has said where it can be.
    pub at: Option<Point>,
}

/// What the state is made from.
pub struct StateInput<'a> {
    /// Every key, at any size.
    pub keys: &'a [KeyBox],
    /// The latched modifiers.
    pub latches: &'a Latches,
    /// Caps Lock is on.
    pub caps: bool,
    /// The layout shown.
    pub table: &'a LayoutTable,
    /// Every layout, in the order the key steps.
    pub list: &'a [LangLabel],
    /// Which way the language names turned.
    pub turn: i8,
    /// The language list, while open.
    pub open: Option<&'a LangList>,
    /// The board keys that drive an open list, with what each does.
    pub panel_keys: &'a [(String, Nav)],
    /// Where the window goes.
    pub at: Option<Point>,
}

/// The board of `spec` at `size`, with the rules and design of `settings`.
#[must_use]
pub fn board_view(spec: &BoardSpec, settings: &KeyboardSettings, size: f64) -> BoardView {
    let b = board(spec, 0.0, size);
    let dim = spec.dpad.later;
    let dpad = DpadView {
        area: b.dpad,
        dim,
        name: DPAD_NAME,
        tip: dim.then_some(LATER_TIP),
    };
    BoardView {
        size,
        preset: Limits::of(settings).preset_of(size),
        presets: settings.presets.clone(),
        relabel_ms: settings.relabel_ms,
        lang_key: settings.lang_key,
        bubble_corner: settings.bubble_corner,
        plate: b.plate,
        bar: b.bar,
        dpad,
        keys: b
            .keys
            .iter()
            .map(|k| key_view(k, settings.lang_key))
            .collect(),
    }
}

/// Key `k` as the board draws it, with the language key in design `style`.
fn key_view(k: &KeyBox, style: LangKey) -> KeyView {
    let text = |prefix: &str| Some(format!("{prefix}{}", k.id));
    let (icon, label, name) = match k.kind {
        KeyKind::Char => (None, None, None),
        KeyKind::Space => (None, None, text(NAME_PREFIX)),
        KeyKind::Lang => (lang_icon(style), None, text(NAME_PREFIX)),
        _ => (text(ICON_PREFIX), text(LABEL_PREFIX), text(NAME_PREFIX)),
    };
    let dim = k.role == Role::Later;
    KeyView {
        id: k.id.clone(),
        kind: k.kind,
        row: k.row,
        side: k.kind == KeyKind::Act,
        cell: k.cell,
        icon,
        label,
        name,
        checkable: matches!(k.role, Role::Latch(_) | Role::Caps),
        dim,
        tip: dim.then_some(LATER_TIP),
    }
}

/// The language key's icon in design `style`: the Carousel shows names only.
fn lang_icon(style: LangKey) -> Option<String> {
    match style {
        LangKey::Carousel => None,
        LangKey::Cycle => Some(CYCLE_ICON.to_owned()),
        LangKey::List => Some(LIST_ICON.to_owned()),
    }
}

/// Every key's state, the language key, the list and the place.
#[must_use]
pub fn state_view(i: &StateInput<'_>) -> StateView {
    let mods = LegendMods::of(i.latches, i.caps, &i.table.keys);
    let rtl = i.table.label.rtl;
    let keys = i
        .keys
        .iter()
        .map(|k| (k.id.clone(), key_state(i, k, mods, rtl)));
    let cur = &i.table.label;
    StateView {
        keys: keys.collect(),
        lang: lang_view(i.list, cur, i.turn),
        list: i.open.map(|l| list_view(l, i.list, cur.id, i.panel_keys)),
        at: i.at,
    }
}

/// Key `k`'s legends, light, paint and tip with the legend modifiers `mods` on a layout written right to left when
/// `rtl`; the language key says so when one layout, or none, is installed.
fn key_state(i: &StateInput<'_>, k: &KeyBox, mods: LegendMods, rtl: bool) -> KeyState {
    if k.role == Role::Later {
        return KeyState {
            paint: DIM,
            ..KeyState::default()
        };
    }
    let chars = k.sc.filter(|_| k.role == Role::Char);
    if let Some(c) = chars.and_then(|sc| i.table.keys.get(&sc)) {
        let text = typed(c, mods);
        return KeyState {
            legend: legend(c, mods, k.row == 0, rtl),
            name: shown(text),
            marks: mark_names(text),
            on: false,
            paint: paint(k.kind, false),
            tip: None,
        };
    }
    let on = match k.role {
        Role::Latch(l) => i.latches.is_on(l),
        Role::Caps => i.caps,
        _ => false,
    };
    let still = k.role == Role::Lang && lone(i.list.len());
    KeyState {
        on,
        paint: paint(k.kind, on),
        tip: still.then_some(LONE_TIP),
        ..KeyState::default()
    }
}

#[cfg(test)]
mod tests;
