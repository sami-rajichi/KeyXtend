//! The shortcuts layer as the UI draws it (spec §4.2): its keys in the side cells while it shows, with each shortcut's
//! steps for its tip, the Shortcuts key lit while it shows and dimmed with a tip while it has nothing to show, and its
//! page for the Next key's dots.

use super::{BoardView, DIM, KeyState, KeyView, NAME_PREFIX, paint};
use crate::geom::side_area;
use crate::layer::{Layer, LayerCell, LayerKey};
use crate::spec::{BoardSpec, KeyKind, NEXT_ID, Role};
use kx_module_api::ports::{KeyName, ShortcutId, ShortcutInfo, Step};
use serde::Serialize;

/// Starts a shortcut's name and label, a translation key such as `shortcut.copy`.
pub const SHORTCUT_NAME_PREFIX: &str = "shortcut.";
/// Starts a shortcut's icon, an `icons.toml` key such as `shortcuts.copy`.
pub const SHORTCUT_ICON_PREFIX: &str = "shortcuts.";
/// The Next key's icon, by its `icons.toml` key.
pub const NEXT_ICON: &str = "layer.next";
/// The Shortcuts key's tip while the layer has nothing to show: "No shortcuts are chosen".
pub const NO_SHORTCUTS_TIP: &str = "tip.no-shortcuts";

/// The layer's changing part: whether it shows, and its page for the Next key's dots.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize)]
pub struct LayerState {
    /// The layer shows.
    pub shown: bool,
    /// The page it is on, from 0, whose dot is lit.
    pub page: usize,
    /// How many pages it has: one dot each.
    pub pages: usize,
}

/// `view` with `layer`'s keys while it shows: they take every side cell but the Shortcuts key's, the pad hides, and
/// each shortcut names its steps from `known`; a hidden layer changes nothing.
pub fn with_layer(view: &mut BoardView, spec: &BoardSpec, layer: &Layer, known: &[ShortcutInfo]) {
    if !layer.shown() {
        return;
    }
    let key = spec.side.iter().find(|s| s.role == Role::Layer);
    let key = key.map(|s| s.id.as_str());
    view.keys.retain(|k| !k.side || Some(k.id.as_str()) == key);
    let size = view.size;
    let cells = layer.cells();
    let drawn = cells.iter().map(|c| cell_view(spec, size, c, known));
    view.keys.extend(drawn);
    view.layer = true;
}

/// Layer key `c` as the board draws it at `size`: a shortcut by its icon alone, with its steps, or Next.
fn cell_view(spec: &BoardSpec, size: f64, c: &LayerCell, known: &[ShortcutInfo]) -> KeyView {
    let base = KeyView {
        id: c.key.id(),
        kind: KeyKind::Act,
        row: usize::from(c.cell.row.saturating_sub(1)),
        side: true,
        cell: side_area(spec, c.cell, size),
        icon: None,
        label: None,
        name: None,
        icon_only: false,
        dots: false,
        checkable: false,
        dim: false,
        tip: None,
        repeat: false,
        steps: Vec::new(),
    };
    match &c.key {
        LayerKey::Shortcut(id) => {
            let text = Some(format!("{SHORTCUT_NAME_PREFIX}{}", id.as_str()));
            KeyView {
                icon: Some(format!("{SHORTCUT_ICON_PREFIX}{}", id.as_str())),
                label: text.clone(),
                name: text,
                icon_only: true,
                steps: steps(id, known),
                ..base
            }
        }
        LayerKey::Next => KeyView {
            icon: Some(NEXT_ICON.to_owned()),
            name: Some(format!("{NAME_PREFIX}{NEXT_ID}")),
            dots: true,
            ..base
        },
    }
}

/// The steps the platform sends for `id`, each as its key names with the tapped key last; none for an id it lacks.
fn steps(id: &ShortcutId, known: &[ShortcutInfo]) -> Vec<Vec<KeyName>> {
    let names = |s: &Step| s.held.iter().chain([&s.key]).cloned().collect();
    let info = known.iter().find(|k| k.id == *id);
    info.map_or_else(Vec::new, |i| i.steps.iter().map(names).collect())
}

/// The Shortcuts key of `kind`: lit while `layer` shows, and dimmed with a tip, doing nothing, while it has nothing
/// to show.
pub(super) fn layer_key(layer: &Layer, kind: KeyKind) -> KeyState {
    if !layer.any() {
        return KeyState {
            dim: true,
            paint: DIM,
            tip: Some(NO_SHORTCUTS_TIP),
            ..KeyState::default()
        };
    }
    let on = layer.shown();
    KeyState {
        on,
        paint: paint(kind, on),
        ..KeyState::default()
    }
}

/// The state of each key `layer` draws now, by id: plain side keys.
pub(super) fn cell_states(layer: &Layer) -> impl Iterator<Item = (String, KeyState)> {
    let state = |c: LayerCell| {
        let paint = paint(KeyKind::Act, false);
        let key = KeyState {
            paint,
            ..KeyState::default()
        };
        (c.key.id(), key)
    };
    layer.cells().into_iter().map(state)
}

/// Whether `layer` shows, and its page.
pub(super) fn layer_state(layer: &Layer) -> LayerState {
    LayerState {
        shown: layer.shown(),
        page: layer.page(),
        pages: layer.page_count(),
    }
}
