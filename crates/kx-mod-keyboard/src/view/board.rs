//! The board as the UI draws it at one size: each key's cell, names and icon, which keys are dimmed or repeat, and
//! the rules and times the UI reads with it.

use super::{
    BoardView, CYCLE_ICON, DPAD_NAME, DpadView, ICON_PREFIX, KeyView, LABEL_PREFIX, LATER_TIP,
    LIST_ICON, NAME_PREFIX,
};
use crate::geom::{KeyBox, board};
use crate::settings::{KeyboardSettings, LangKey};
use crate::sizer::Limits;
use crate::spec::{BoardSpec, KeyKind, Role};
use std::collections::HashSet;

/// The board of `spec` at `size`, with the rules and design of `settings`; with repeat off, no key repeats.
#[must_use]
pub fn board_view(spec: &BoardSpec, settings: &KeyboardSettings, size: f64) -> BoardView {
    let b = board(spec, 0.0, size);
    let dpad = DpadView {
        area: b.dpad,
        name: DPAD_NAME,
    };
    let repeating = if settings.repeat {
        spec.repeating()
    } else {
        HashSet::new()
    };
    let key = |k: &KeyBox| key_view(k, settings.lang_key, repeating.contains(&k.id));
    BoardView {
        size,
        preset: Limits::of(settings).preset_of(size),
        presets: settings.presets.clone(),
        relabel_ms: settings.relabel_ms,
        repeat_delay_ms: settings.repeat_delay_ms,
        repeat_every_ms: settings.repeat_every_ms,
        lang_key: settings.lang_key,
        bubble_corner: settings.bubble_corner,
        plate: b.plate,
        bar: b.bar,
        dpad,
        keys: b.keys.iter().map(key).collect(),
    }
}

/// Key `k` as the board draws it, with the language key in design `style`; it repeats while held when `repeat`.
fn key_view(k: &KeyBox, style: LangKey, repeat: bool) -> KeyView {
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
        checkable: matches!(k.role, Role::Latch(_) | Role::Caps | Role::Assist(_)),
        dim,
        tip: dim.then_some(LATER_TIP),
        repeat,
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
