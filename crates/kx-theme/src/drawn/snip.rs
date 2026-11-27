//! What Snip's capture windows draw beyond the keycaps' shared pairs (SnipKey.qml, SnipHint.qml, SnipCount.qml,
//! SnipRecord.qml): text on a see-through tag over any picture, the record dot, a record key hovered and Stop, on a
//! keycap the picked way's ring and the video ways' red light at rest, hovered and pressed, and a step's number on
//! its ink.

use super::{Drawn, worst};
use crate::contrast::{WCAG_AA, WCAG_SHAPES};
use crate::rgba::Rgba;
use crate::view::LookView;

/// The size tag's, the hint's and the countdown's text on their tag.
pub(super) const TAG: &str = "snip text on its tag";
/// The record bar's dot, and a sound switch's icon there while it records.
pub(super) const DOT: &str = "snip record dot";
/// Stop's word and icon on its red key.
pub(super) const STOP: &str = "snip Stop on its key";
/// The ring inside a picked way or a switch that records, on its keycap.
pub(super) const RING: &str = "snip ring on its keycap";
/// A video way's red light on its keycap.
pub(super) const LIGHT: &str = "snip red light on its keycap";
/// A step's number on its ink in the editor (SnipMark.qml), named `<NUMBER>, <ink>`.
pub(super) const NUMBER: &str = "snip step number on its ink";
/// A record bar key's icon on its pill under the pointer (SnipPill.qml), recording or not.
pub(super) const HOVER: &str = "snip record key hovered";

/// The pictures a tag may lie over, at their brightest and darkest.
pub(super) const PICTURE: [Rgba; 2] = [Rgba::WHITE, Rgba::BLACK];

/// The tag's backgrounds: see-through over any picture, or opaque in high contrast, where its text is `WindowText`.
pub(super) fn tags(v: &LookView) -> Vec<Rgba> {
    let tag = v.common.snip_tag;
    if v.contrast {
        vec![Rgba { a: 0xFF, ..tag }]
    } else {
        PICTURE.map(|w| tag.on(w)).to_vec()
    }
}

/// Mark `ink` on a side keycap (SnipKey.qml): at rest and hovered, then pressed under the press shade, where it takes
/// the press text when the look has one, as a legend does.
fn on_way(v: &LookView, name: &str, ink: Rgba, out: &mut Vec<Drawn>) {
    let (p, cap) = (&v.palette, v.palette.key_act);
    out.push(worst(name, ink, 1.0, &[cap, p.hover.on(cap)], WCAG_SHAPES));
    let pressed = p.press_ink.unwrap_or(ink);
    let down = format!("{name}, press");
    out.push(worst(&down, pressed, 1.0, &[p.press.on(cap)], WCAG_SHAPES));
}

/// Snip's own pairs; high contrast draws the dot in Highlight, as the dictation bar's, and Stop in its system colours.
/// A record bar key under the pointer takes `sel_ink`, as the panels' flat buttons do, since the red dot falls short of
/// 3:1 on `pop_hover` in some dark looks and high contrast fills it with Highlight.
pub(super) fn snip(v: &LookView, out: &mut Vec<Drawn>) {
    let (p, c) = (&v.palette, &v.common);
    out.push(worst(TAG, c.snip_tag_ink, 1.0, &tags(v), WCAG_AA));
    let dot = if v.contrast { p.ring } else { c.rec_dot };
    out.push(worst(DOT, dot, 1.0, &[p.pop_bg], WCAG_SHAPES));
    let hot = [p.pop_hover.on(p.pop_bg)];
    out.push(worst(HOVER, v.sel_ink, 1.0, &hot, WCAG_SHAPES));
    out.push(worst(STOP, p.rec_ink, 1.0, &[p.rec], WCAG_AA));
    on_way(v, RING, p.ring_act, out);
    on_way(v, LIGHT, p.rec_light, out);
    for (id, ink, number) in c.inks() {
        out.push(worst(
            &format!("{NUMBER}, {id}"),
            number,
            1.0,
            &[ink],
            WCAG_AA,
        ));
    }
}
