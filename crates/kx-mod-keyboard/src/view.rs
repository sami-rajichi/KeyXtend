//! What the UI draws, as the face's JSON: the board (boxes, names, dimmed keys, the size and the shortcuts layer's
//! keys), which changes with the size and the layer, and the state (legends, lights, the language key, the list, the
//! place, the pad and the layer's page), which changes as keys are used.

mod assist;
mod board;
mod layer;
mod pad;
mod paint;
mod power;

pub use assist::{GRAB_TIP, MOUSE_OFF_TIP, RCLICK_TIP};
pub use board::board_view;
pub use layer::{
    LayerState, NEXT_ICON, NO_SHORTCUTS_TIP, SHORTCUT_ICON_PREFIX, SHORTCUT_NAME_PREFIX, with_layer,
};
pub use pad::{PadState, SCROLL_OFF_TIP, pad_state};
pub use paint::{DIM, Paint, paint};
pub use power::POWER_OFF_TIP;

use crate::geom::{Area, KeyBox};
use crate::langkey::{LangView, lang_view, lone};
use crate::latch::Latches;
use crate::layer::Layer;
use crate::legend::{Legend, LegendMods, is_empty, legend, mark_names, shown, typed};
use crate::panel::{LangList, ListView, list_view};
use crate::settings::{Corner, LangKey};
use crate::spec::{KeyKind, Nav, Role};
use kx_module_api::ports::{
    AssistChanged, KeyName, LangLabel, LayoutTable, Point, PowerChanged, ScrollChanged,
};
use serde::Serialize;
use std::collections::BTreeMap;
use std::fmt;

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
/// The name of a character key that types nothing with the latched Shift or `AltGr`: "Empty key".
pub const EMPTY_NAME: &str = "key.empty";
/// The Cycle design's globe, by its `icons.toml` key.
pub const CYCLE_ICON: &str = "lang.cycle";
/// The List design's chevron, by its `icons.toml` key.
pub const LIST_ICON: &str = "lang.list";

/// One key as the board draws it; it changes only with the size or the settings, such as the language key's design.
#[derive(Clone, Debug, PartialEq, Serialize)]
#[allow(
    clippy::struct_excessive_bools,
    reason = "Each flag is one fact about the key that QML reads flat from the board JSON."
)]
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
    /// It shows its icon alone, never the label under it; the label stands in only when the look has no such icon.
    pub icon_only: bool,
    /// It shows the shortcuts layer's page dots under its icon: the Next key.
    pub dots: bool,
    /// It latches, locks or turns a mode on, so screen readers hear it as checkable.
    pub checkable: bool,
    /// It comes in a later version: drawn dimmed and does nothing.
    pub dim: bool,
    /// Its tip by translation key, for a dimmed key.
    pub tip: Option<&'static str>,
    /// It repeats while held (spec §4.6), so the UI sends its press and let-go instead of a click.
    pub repeat: bool,
    /// A shortcut's steps for its tip, in order: each step's key names, the tapped key last; none for other keys.
    pub steps: Vec<Vec<KeyName>>,
}

/// The D-pad's round face; whether it works, and how it scrolls, come with the state.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct DpadView {
    /// Its box in logical pixels, inset like a key's cell.
    #[serde(flatten)]
    pub area: Area,
    /// Its name, by translation key.
    pub name: &'static str,
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
    /// How long the UI's hold timer waits before its first `tick`, in ms.
    pub repeat_delay_ms: u64,
    /// The hold timer's pace after that, in ms.
    pub repeat_every_ms: u64,
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
    /// The shortcuts layer shows: its keys stand in the side block but the Shortcuts key's cell, and the pad hides.
    pub layer: bool,
    /// Every key: main rows left to right, then the side block, or the Shortcuts key and the layer's keys.
    pub keys: Vec<KeyView>,
}

/// One key's changing part.
#[derive(Clone, Default, PartialEq, Eq, Serialize)]
pub struct KeyState {
    /// Legends; empty for named keys.
    #[serde(flatten)]
    pub legend: Legend,
    /// What a character key types now, for screen readers, marks on a dotted circle; empty for named keys.
    pub name: String,
    /// The names of the marks it types, by translation key, which readers hear instead of `name`; often none.
    pub marks: Vec<String>,
    /// `EMPTY_NAME` while a character key types nothing, so it shows no main legend; readers hear it instead of `name`.
    pub blank: Option<&'static str>,
    /// Latched, or Caps Lock while it is on.
    pub on: bool,
    /// It does nothing now: a later key, a mouse-assist or Power key while its module is not running, or the
    /// Shortcuts key with nothing to show.
    pub dim: bool,
    /// Its colours now, by palette token.
    pub paint: Paint,
    /// What it says after its name now, by translation key, shown in its tip: the language key's `LONE_TIP`, or a
    /// side key's.
    pub tip: Option<&'static str>,
}

/// Everything that changes as keys are used.
#[derive(Clone, Default, PartialEq, Eq, Serialize)]
pub struct StateView {
    /// Each key's state, by id, the layer's keys too while it shows.
    pub keys: BTreeMap<String, KeyState>,
    /// The language key and the space bar.
    pub lang: LangView,
    /// The language list while it is open.
    pub list: Option<ListView>,
    /// Where the window goes, in physical pixels, once the UI has said where it can be.
    pub at: Option<Point>,
    /// The scroll pad.
    pub dpad: PadState,
    /// The shortcuts layer and its page.
    pub layer: LayerState,
}

/// Shows no field: its legends and lights tell the latches and Caps Lock, as `Latches` hides them.
impl fmt::Debug for KeyState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("KeyState").finish_non_exhaustive()
    }
}

/// Shows the language key, the list and the place, never the keys' states.
impl fmt::Debug for StateView {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("StateView")
            .field("lang", &self.lang)
            .field("list", &self.list)
            .field("at", &self.at)
            .finish_non_exhaustive()
    }
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
    /// Rows per page of the open list, as the UI shows it.
    pub rows: usize,
    /// The board keys that drive an open list, with what each does.
    pub panel_keys: &'a [(String, Nav)],
    /// Where the window goes.
    pub at: Option<Point>,
    /// The mouse module's state, as the bus last told it.
    pub assist: AssistChanged,
    /// The scroll module's state, as the bus last told it.
    pub scroll: ScrollChanged,
    /// The shortcuts layer.
    pub layer: &'a Layer,
    /// The power module's state, as the bus last told it.
    pub power: PowerChanged,
}

/// Every key's state, the language key, the list, the place, the pad and the layer.
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
        keys: keys.chain(layer::cell_states(i.layer)).collect(),
        lang: lang_view(i.list, cur, i.turn),
        list: i
            .open
            .map(|l| list_view(l, i.list, cur.id, i.panel_keys, i.rows)),
        at: i.at,
        dpad: pad_state(i.scroll),
        layer: layer::layer_state(i.layer),
    }
}

/// Key `k`'s legends, light, paint and tip with the legend modifiers `mods` on a layout written right to left when
/// `rtl`; the language key says so when one layout, or none, is installed.
fn key_state(i: &StateInput<'_>, k: &KeyBox, mods: LegendMods, rtl: bool) -> KeyState {
    match k.role {
        Role::Later => {
            return KeyState {
                dim: true,
                paint: DIM,
                ..KeyState::default()
            };
        }
        Role::Assist(a) => return assist::assist_state(i.assist, a, k.kind),
        Role::Layer => return layer::layer_key(i.layer, k.kind),
        Role::Power => return power::power_key(i.power, k.kind),
        _ => {}
    }
    let chars = k.sc.filter(|_| k.role == Role::Char);
    if let Some(c) = chars.and_then(|sc| i.table.keys.get(&sc)) {
        let text = typed(c, mods);
        return KeyState {
            legend: legend(c, mods, k.row == 0, rtl),
            name: shown(text),
            marks: mark_names(text),
            blank: is_empty(c, mods).then_some(EMPTY_NAME),
            on: false,
            dim: false,
            paint: paint(k.kind, false),
            tip: None,
        };
    }
    let on = match k.role {
        Role::Latch(r) => i.latches.is_on(r.latch),
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
