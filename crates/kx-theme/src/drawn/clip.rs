//! What the clipboard panel draws beyond the shared panel pairs (ClipRow.qml, ClipFilters.qml, ClipButton.qml): a
//! ticked row's text and marks, the tick number, the kind icon on its tile, the chips and the coloured buttons.

use super::{Drawn, worst};
use crate::contrast::{WCAG_AA, WCAG_SHAPES};
use crate::rgba::Rgba;
use crate::view::LookView;

/// A ticked row's text.
pub(super) const TICKED_TEXT: &str = "clip text on a ticked row";
/// A ticked row's marks.
pub(super) const TICKED_MARK: &str = "clip mark on a ticked row";
/// The tick number on its badge.
pub(super) const TICK_NUMBER: &str = "clip tick number";
/// The kind icon on its tile.
pub(super) const KIND_ICON: &str = "clip kind icon";
/// The kind icon on its tile in a row under the pointer or ticked.
pub(super) const LIT_KIND_ICON: &str = "clip kind icon on a lit row";
/// The chip on media that goes tonight.
pub(super) const EXPIRES_CHIP: &str = "clip expires chip";
/// A filter chip under the pointer.
pub(super) const HOVER_CHIP: &str = "clip chip under the pointer";
/// The Delete button in the danger keys' colours.
pub(super) const DELETE_BUTTON: &str = "clip delete button";
/// The Insert button in Enter's colours.
pub(super) const INSERT_BUTTON: &str = "clip insert button";

/// One pair: its name, the ink, the backgrounds it sits on and the least contrast it needs.
type Pair = (&'static str, Rgba, Vec<Rgba>, f32);

/// The panel's own pairs; its plain text, hovered rows and ghost buttons are `plate::panels`' pairs.
pub(super) fn clip(v: &LookView, out: &mut Vec<Drawn>) {
    for (name, ink, bgs, min) in rows(v).into_iter().chain(controls(v)) {
        out.push(worst(name, ink, 1.0, &bgs, min));
    }
}

/// A row's pairs: a ticked row's text and marks, the tick number and the kind icon on its tile, lit or not.
fn rows(v: &LookView) -> [Pair; 5] {
    let p = &v.palette;
    let bg = p.pop_bg;
    let (hot, ticked) = (p.pop_hover.on(bg), p.pop_sel.on(bg));
    let both = p.pop_hover.on(ticked);
    let tile = |row| p.pop_hover.on(row);
    // High contrast fills a ticked row and every tile with Highlight, so their marks take sel_ink there.
    let (mark, kind) = if v.contrast {
        (v.sel_ink, v.sel_ink)
    } else {
        (p.ring, p.pop_muted)
    };
    [
        (TICKED_TEXT, v.sel_ink, vec![ticked, both], WCAG_AA),
        (TICKED_MARK, mark, vec![ticked], WCAG_SHAPES),
        (TICK_NUMBER, p.badge_ink, vec![p.badge_bg], WCAG_AA),
        (KIND_ICON, kind, vec![tile(bg)], WCAG_SHAPES),
        (
            LIT_KIND_ICON,
            v.sel_ink,
            vec![tile(hot), tile(ticked), tile(both)],
            WCAG_SHAPES,
        ),
    ]
}

/// The chips' and coloured buttons' pairs, under the pointer too.
fn controls(v: &LookView) -> [Pair; 4] {
    let p = &v.palette;
    [
        (EXPIRES_CHIP, p.danger_ink, vec![p.danger], WCAG_AA),
        (HOVER_CHIP, p.pop_ink, vec![p.hover.on(p.pop_bg)], WCAG_AA),
        (
            DELETE_BUTTON,
            p.danger_ink,
            vec![p.danger, p.hover.on(p.danger)],
            WCAG_AA,
        ),
        (
            INSERT_BUTTON,
            p.enter_ink,
            vec![p.enter, p.hover.on(p.enter)],
            WCAG_AA,
        ),
    ]
}
