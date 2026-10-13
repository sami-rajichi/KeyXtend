//! Where each legend sits on its cap at size 1, as KeyText.qml lays it out, and the cap colours under it (CapFace.qml).

use crate::look::{Cap, Gradient};
use crate::rgba::Rgba;
use crate::view::LookView;

/// A stretch of the cap, top and bottom, in px at size 1 from the cap's top edge.
pub(super) type Span = [f32; 2];

/// What an ink draws on a cap, which sets where it sits.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Mark {
    /// A character key's main legend, at its Latin and Arabic sizes.
    Main,
    /// The corner legends and the space bar's text.
    Second,
    /// A named key's label, or the language key's current name.
    Label,
    /// A named key's icon.
    Icon,
    /// Cycle's column of dots.
    Dots,
    /// Cycle's globe.
    Globe,
    /// List's chevron.
    Chevron,
    /// Carousel's side names.
    Side,
}

/// The cap's height at size 1: the keyboard's row, `row_px`, less the gap between keys.
pub(super) fn height(v: &LookView, row_px: f32) -> f32 {
    row_px - v.look.gap_px
}

/// How far a sculpted cap lifts its legends.
fn lift(v: &LookView) -> f32 {
    match &v.look.cap {
        Cap::Skirt { legend_lift_px, .. } => *legend_lift_px,
        _ => 0.0,
    }
}

/// The face the theme draws, as insets from the cap (top, right, bottom, left): a sculpted lit top, else the cap.
fn face(v: &LookView) -> [f32; 4] {
    match &v.look.cap {
        Cap::Skirt { top_inset_px, .. } => *top_inset_px,
        _ => [0.0; 4],
    }
}

/// The capitals of a text `px` high whose line box starts at `top`: baseline up to cap height.
fn text(v: &LookView, top: f32, px: f32) -> Span {
    let [ascent, _, cap] = v.look.font_metrics;
    let base = top + ascent * px;
    [base - cap * px, base]
}

/// A text `px` high whose line box is centred on `mid`.
fn centred(v: &LookView, mid: f32, px: f32) -> Span {
    let [ascent, descent, _] = v.look.font_metrics;
    text(v, mid - (ascent + descent) * px / 2.0, px)
}

/// A text `px` high whose line box ends at `bottom`.
fn above(v: &LookView, bottom: f32, px: f32) -> Span {
    let [ascent, descent, _] = v.look.font_metrics;
    text(v, bottom - (ascent + descent) * px, px)
}

/// A square of side `px` centred on `mid`.
fn square(mid: f32, px: f32) -> Span {
    [mid - px / 2.0, mid + px / 2.0]
}

/// The tallest column of Cycle's dots that fits a cap `h` high with room for the larger lit dot (LangCycle.qml
/// `perColumn`).
fn column(v: &LookView, h: f32) -> f32 {
    let c = &v.shape.lang.cycle;
    let room = h - (c.dot_on_px - c.dot_px);
    let dots = ((room + c.dot_gap_px) / (c.dot_px + c.dot_gap_px))
        .floor()
        .max(1.0);
    dots * c.dot_px + (dots - 1.0) * c.dot_gap_px
}

/// Where `mark` sits on a cap `h` high: one span for each place or size it may take.
pub(super) fn spans(v: &LookView, h: f32, mark: Mark) -> Vec<Span> {
    let (look, lg, lang) = (&v.look, &v.shape.legend, &v.shape.lang);
    let mid = h / 2.0 - lift(v);
    match mark {
        Mark::Main => [1.0, look.arabic_scale]
            .map(|k| centred(v, mid, look.legend_px * k))
            .to_vec(),
        Mark::Second => {
            // The corners sit in from the face on the theme's Latin baseline, whatever font they draw in; as for every
            // text, the check reads their capitals, so accents and bracket tips above them go unchecked.
            let corner = |i: usize| look.legend_px * lg.corner_scale[i];
            let [top, _, bottom, _] = face(v);
            let [from_top, from_bottom, _] = lg.corner_px;
            let mut all = vec![
                text(v, top + from_top, corner(0)),
                above(v, h - bottom - from_bottom, corner(1)),
            ];
            // The space bar's text is centred on the cap itself, unlifted.
            all.extend(lg.space_px.map(|px| centred(v, h / 2.0, px)));
            all
        }
        Mark::Label => {
            let names = lang.font_px.map(|px| centred(v, mid, px));
            [vec![centred(v, mid, look.mod_legend_px)], names.to_vec()].concat()
        }
        Mark::Icon => lg.icon_px.map(|px| square(mid, px)).to_vec(),
        Mark::Dots => vec![square(mid, column(v, h))],
        Mark::Globe => vec![square(mid, lang.cycle.icon_px)],
        Mark::Chevron => vec![square(mid, lang.list.icon_px)],
        Mark::Side => vec![centred(v, mid, lang.side[0])],
    }
}

/// Gradient `g` on `fill` as Qt draws it: its stops top to bottom, an upward one flipped.
fn stops(g: &Gradient, fill: Rgba) -> [(f32, Rgba); 3] {
    let light = Rgba::WHITE.over(fill, g.lighten);
    let dark = Rgba::BLACK.over(fill, g.darken);
    if g.upward {
        [(0.0, dark), (1.0 - g.mid_stop, fill), (1.0, light)]
    } else {
        [(0.0, light), (g.mid_stop, fill), (1.0, dark)]
    }
}

/// The colour at `t`, 0 at the top and 1 at the bottom, between `stops`, each channel blended in a straight line.
fn shade(stops: &[(f32, Rgba); 3], t: f32) -> Rgba {
    let pair = stops
        .windows(2)
        .find(|w| t <= w[1].0)
        .unwrap_or(&stops[1..]);
    let ((start, from), (end, to)) = (pair[0], pair[1]);
    let share = if end > start {
        (t - start) / (end - start)
    } else {
        0.0
    };
    to.over(from, share.clamp(0.0, 1.0))
}

/// The colours a cap `h` high and of `fill` shows over `span`, at rest or pressed: both ends and any stop between,
/// plus the skirt when the span runs off a sculpted top.
pub(super) fn under(
    v: &LookView,
    h: f32,
    fill: Rgba,
    skirt: Rgba,
    span: Span,
    pressed: bool,
) -> Vec<Rgba> {
    let (grad, band) = match &v.look.cap {
        Cap::Flat {} => return vec![fill],
        Cap::Convex {
            rest,
            pressed: down,
        } => (if pressed { down } else { rest }, [0.0, h]),
        Cap::Skirt {
            top,
            top_inset_px: up,
            pressed_inset_px: down,
            ..
        } => {
            let inset = if pressed { down } else { up };
            (top, [inset[0], h - inset[2]])
        }
    };
    let at = |y: f32| ((y - band[0]) / (band[1] - band[0])).clamp(0.0, 1.0);
    let (first, last) = (at(span[0]), at(span[1]));
    let stops = stops(grad, fill);
    let inner = stops.iter().map(|s| s.0);
    let inner = inner.filter(|t| (first..=last).contains(t));
    let mut out: Vec<Rgba> = [first, last]
        .into_iter()
        .chain(inner)
        .map(|t| shade(&stops, t))
        .collect();
    let off = span[0] < band[0] || span[1] > band[1];
    if off && matches!(v.look.cap, Cap::Skirt { .. }) {
        out.push(skirt);
    }
    out
}

#[cfg(test)]
mod tests;
