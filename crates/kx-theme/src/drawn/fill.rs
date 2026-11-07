//! What Quick-fill draws (`QuickFill*.qml`): its texts and icons on the panel, its kind tiles, ghost and shown fills,
//! chips under the pointer, and Save and Delete entry in Enter's and the danger keys' colours. Each reads as text does.

use super::{Drawn, worst};
use crate::contrast::WCAG_AA;
use crate::rgba::Rgba;
use crate::view::LookView;

/// Names, values, typed text, a backup password as it is typed, the header and the questions, on the panel.
pub(super) const INK: &str = "fill text";
/// Hints, a place's state, a found box's line, the folder shown, captions, the mask, an empty line's hint, the
/// invitations and the waiting line, with their icons.
pub(super) const MUTED: &str = "fill muted text and icons";
/// A kind's, a label's or a place's icon on its tile: in `pop_ink`, as `pop_muted` there falls below 4.5:1 in Dolch
/// light, and in Highlight text in high contrast, where the tile is Highlight.
pub(super) const TILE_ICON: &str = "fill icon on its tile";
/// A ghost button, a kind chip and the note band, under the pointer too.
pub(super) const GHOST: &str = "fill text on a ghost fill";
/// The shown filter chip, and the hidden and memory checks switches while on, under the pointer too.
pub(super) const SHOWN: &str = "fill text on the shown fill";
/// A filter chip or a flat button under the pointer.
pub(super) const CHIP: &str = "fill text under the pointer";
/// Save, Done and a foreign box's Yes in Enter's colours.
pub(super) const SAVE: &str = "fill save button";
/// A line a click aims the letters at, under the pointer: its text, hint, caption and icon, as `ClipRow`'s hovered row.
pub(super) const LINE: &str = "fill text on a hovered line";
/// Delete entry, and the Yes of Delete entry, Remove place and Replace, in the danger keys' colours.
pub(super) const DANGER: &str = "fill delete button";

/// One pair: its name, the ink and the backgrounds it sits on.
type Pair = (&'static str, Rgba, Vec<Rgba>);

/// Quick-fill's pairs, each at 4.5:1, icons too.
pub(super) fn fill(v: &LookView, out: &mut Vec<Drawn>) {
    for (name, ink, bgs) in pairs(v) {
        out.push(worst(name, ink, 1.0, &bgs, WCAG_AA));
    }
}

/// Each pair on the colours QML draws it on.
fn pairs(v: &LookView) -> [Pair; 9] {
    let p = &v.palette;
    let bg = p.pop_bg;
    let (ghost, shown) = (p.pop_hover.on(bg), p.pop_sel.on(bg));
    [
        (INK, p.pop_ink, vec![bg]),
        (MUTED, p.pop_muted, vec![bg]),
        (TILE_ICON, v.sel_ink, vec![ghost]),
        (GHOST, v.sel_ink, vec![ghost, p.hover.on(ghost)]),
        (SHOWN, v.sel_ink, vec![shown, p.hover.on(shown)]),
        (CHIP, p.pop_ink, vec![p.hover.on(bg)]),
        (SAVE, p.enter_ink, vec![p.enter, p.hover.on(p.enter)]),
        (DANGER, p.danger_ink, vec![p.danger, p.hover.on(p.danger)]),
        (LINE, v.sel_ink, vec![ghost]),
    ]
}
