//! What the clipboard panel draws beyond the shared panel pairs (ClipRow.qml, ClipFilters.qml, ClipButton.qml): a
//! ticked row's text and marks, the tick number, the kind icon on its tile, the chips and the coloured buttons.

use super::{Drawn, worst};
use crate::contrast::{WCAG_AA, WCAG_SHAPES};
use crate::rgba::Rgba;
use crate::view::LookView;

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
        (
            "clip text on a ticked row",
            v.sel_ink,
            vec![ticked, both],
            WCAG_AA,
        ),
        ("clip mark on a ticked row", mark, vec![ticked], WCAG_SHAPES),
        ("clip tick number", p.badge_ink, vec![p.badge_bg], WCAG_AA),
        ("clip kind icon", kind, vec![tile(bg)], WCAG_SHAPES),
        (
            "clip kind icon on a lit row",
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
        ("clip expires chip", p.danger_ink, vec![p.danger], WCAG_AA),
        (
            "clip chip under the pointer",
            p.pop_ink,
            vec![p.hover.on(p.pop_bg)],
            WCAG_AA,
        ),
        (
            "clip delete button",
            p.danger_ink,
            vec![p.danger, p.hover.on(p.danger)],
            WCAG_AA,
        ),
        (
            "clip insert button",
            p.enter_ink,
            vec![p.enter, p.hover.on(p.enter)],
            WCAG_AA,
        ),
    ]
}
