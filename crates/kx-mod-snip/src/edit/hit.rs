//! What a click reaches: the top drawing that shows and lies within the slop of it, by its shape and the theme's sizes
//! (`SnipFit`), and how near two clicks are one spot. The slop comes in window pixels and is scaled to the picture's.

use crate::frame;
use kx_module_api::caret;
use kx_module_api::ports::{
    Drawing, Mark, Point, Rect, Scale, Shape, SnipFit, TextSize, Thickness,
};

/// A point in reals, for distances.
#[derive(Copy, Clone)]
struct At {
    x: f64,
    y: f64,
}

impl At {
    fn of(p: Point) -> Self {
        Self {
            x: f64::from(p.x),
            y: f64::from(p.y),
        }
    }

    fn to(self, other: Self) -> f64 {
        (self.x - other.x).hypot(self.y - other.y)
    }
}

/// A box in reals: its left, top, right and bottom.
#[derive(Copy, Clone)]
struct Ext {
    x0: f64,
    y0: f64,
    x1: f64,
    y1: f64,
}

impl Ext {
    /// The box from `a` to `b`.
    fn span(a: At, b: At) -> Self {
        Self {
            x0: a.x.min(b.x),
            y0: a.y.min(b.y),
            x1: a.x.max(b.x),
            y1: a.y.max(b.y),
        }
    }

    /// The box round `points`; `None` for none.
    fn round(points: &[Point]) -> Option<Self> {
        let boxes = points.iter().map(|p| Self::span(At::of(*p), At::of(*p)));
        boxes.reduce(Self::join)
    }

    /// The box holding both.
    fn join(self, other: Self) -> Self {
        Self {
            x0: self.x0.min(other.x0),
            y0: self.y0.min(other.y0),
            x1: self.x1.max(other.x1),
            y1: self.y1.max(other.y1),
        }
    }

    /// This box grown by `by` on every side.
    fn grow(self, by: f64) -> Self {
        Self {
            x0: self.x0 - by,
            y0: self.y0 - by,
            x1: self.x1 + by,
            y1: self.y1 + by,
        }
    }

    /// This box reaches into the pixels of `crop`.
    fn meets(self, crop: Rect) -> bool {
        let (x0, y0) = (f64::from(crop.x), f64::from(crop.y));
        let (x1, y1) = (x0 + f64::from(crop.w) - 1.0, y0 + f64::from(crop.h) - 1.0);
        self.x0 <= x1 && self.x1 >= x0 && self.y0 <= y1 && self.y1 >= y0
    }

    /// How far `p` is from this box filled; 0 inside.
    fn gap(self, p: At) -> f64 {
        let dx = (self.x0 - p.x).max(p.x - self.x1).max(0.0);
        let dy = (self.y0 - p.y).max(p.y - self.y1).max(0.0);
        dx.hypot(dy)
    }

    /// How far `p` is from this box's edges.
    fn edge_gap(self, p: At) -> f64 {
        let inside = (p.x - self.x0)
            .min(self.x1 - p.x)
            .min(p.y - self.y0)
            .min(self.y1 - p.y);
        if inside > 0.0 { inside } else { self.gap(p) }
    }

    /// About how far `p` is from the circle in this box, along the ray from its centre.
    fn ring_gap(self, p: At) -> f64 {
        let (a, b) = ((self.x1 - self.x0) / 2.0, (self.y1 - self.y0) / 2.0);
        let (dx, dy) = (p.x - (self.x0 + a), p.y - (self.y0 + b));
        let r = (dx / a).hypot(dy / b);
        if r == 0.0 {
            return a.min(b);
        }
        let len = dx.hypot(dy);
        (len - len / r).abs()
    }
}

/// How far a click may miss a frame's handle at `scale`: half `fit`'s handle click area, in the picture's whole pixels.
pub(super) fn reach(fit: &SnipFit, scale: Scale) -> i32 {
    let px =
        f64::from(fit.handle) / frame::SIDES * f64::from(Scale::PER) / f64::from(scale.shown());
    #[allow(
        clippy::cast_possible_truncation,
        reason = "a u16 over a scale of at least 1 stays well inside the range"
    )]
    let reach = px.round() as i32;
    reach
}

/// `fit`'s slop at `scale`, in the picture's pixels.
pub(super) fn slop(fit: &SnipFit, scale: Scale) -> f64 {
    f64::from(fit.slop) * f64::from(Scale::PER) / f64::from(scale.shown())
}

/// Two clicks no farther apart than `slop`, in the picture's pixels, are one spot, as an unsteady hand gives.
pub(super) fn one_spot(a: Point, b: Point, slop: f64) -> bool {
    At::of(a).to(At::of(b)) <= slop
}

/// What a click is measured by: the theme's sizes, the slop in the picture's pixels, and the part that shows.
pub(super) struct Aim<'a> {
    fit: &'a SnipFit,
    slop: f64,
    crop: Rect,
}

impl<'a> Aim<'a> {
    /// A click shown at `scale`, on the part `crop`.
    pub(super) fn new(fit: &'a SnipFit, scale: Scale, crop: Rect) -> Self {
        let slop = slop(fit, scale);
        Self { fit, slop, crop }
    }

    /// The place of the top drawing that shows and that `at` lands on, the last drawn first; `None` on an empty spot.
    pub(super) fn top(&self, drawings: &[Drawing], at: Point) -> Option<usize> {
        let p = At::of(at);
        let shows = |d: &Drawing| self.extent(d).is_some_and(|e| e.meets(self.crop));
        drawings
            .iter()
            .rposition(|d| shows(d) && self.gap(d, p) <= self.slop)
    }

    /// The theme's thickness step `step`, or its last one past its end.
    fn thick(&self, step: u8) -> Thickness {
        let list = &self.fit.thick;
        list.get(usize::from(step))
            .or(list.last())
            .copied()
            .unwrap_or_default()
    }

    /// The theme's text size `step`, or its last one past its end.
    fn text(&self, step: u8) -> TextSize {
        let list = &self.fit.text;
        list.get(usize::from(step))
            .or(list.last())
            .copied()
            .unwrap_or_default()
    }

    /// How far `p` is from `d`'s ink; 0 or less on it.
    fn gap(&self, d: &Drawing, p: At) -> f64 {
        let thick = self.thick(d.size);
        match &d.mark {
            Mark::Pen(path) => path_gap(p, path) - half(thick.pen),
            Mark::Highlight(path) => path_gap(p, path) - half(thick.marker),
            Mark::Shape { shape, from, to } => {
                shape_gap(*shape, p, At::of(*from), At::of(*to), thick)
            }
            Mark::Step { at, .. } => p.to(At::of(*at)) - half(thick.disc),
            Mark::Text { .. } => self.extent(d).map_or(f64::INFINITY, |e| e.gap(p)),
        }
    }

    /// The box `d`'s ink fills; `None` for a path with no point.
    fn extent(&self, d: &Drawing) -> Option<Ext> {
        let thick = self.thick(d.size);
        let span = |from: &Point, to: &Point| Ext::span(At::of(*from), At::of(*to));
        Some(match &d.mark {
            Mark::Pen(path) => Ext::round(path)?.grow(half(thick.pen)),
            Mark::Highlight(path) => Ext::round(path)?.grow(half(thick.marker)),
            Mark::Shape {
                shape: Shape::Arrow,
                from,
                to,
            } => span(from, to).grow(f64::from(thick.head) + half(thick.pen)),
            Mark::Shape {
                shape: Shape::Cover,
                from,
                to,
            } => span(from, to),
            Mark::Shape { from, to, .. } => span(from, to).grow(half(thick.pen)),
            Mark::Step { at, .. } => span(at, at).grow(half(thick.disc)),
            Mark::Text { at, text, rtl } => self.text_box(At::of(*at), text.expose(), *rtl, d.size),
        })
    }

    /// The box a text fills: its letters' widths from its point, leftward when it reads right to left, a line high.
    fn text_box(&self, at: At, text: &str, rtl: bool, step: u8) -> Ext {
        let size = self.text(step);
        let letters = u32::try_from(caret::graphemes(text)).unwrap_or(u32::MAX);
        let width = f64::from(letters) * f64::from(size.letter);
        let x0 = if rtl { at.x - width } else { at.x };
        Ext {
            x0,
            y0: at.y,
            x1: x0 + width,
            y1: at.y + f64::from(size.height),
        }
    }
}

/// Half of `width`.
fn half(width: u16) -> f64 {
    f64::from(width) / 2.0
}

/// How far `p` is from a shape's ink; an arrow's head reaches `head` from its tip, and a cover is filled.
fn shape_gap(shape: Shape, p: At, from: At, to: At, thick: Thickness) -> f64 {
    let pen = half(thick.pen);
    let span = Ext::span(from, to);
    match shape {
        Shape::Line => segment_gap(p, from, to) - pen,
        Shape::Arrow => {
            (segment_gap(p, from, to) - pen).min(p.to(to) - f64::from(thick.head) - pen)
        }
        Shape::Box => span.edge_gap(p) - pen,
        Shape::Circle => span.ring_gap(p) - pen,
        Shape::Cover => span.gap(p),
    }
}

/// How far `p` is from the segment from `a` to `b`.
fn segment_gap(p: At, a: At, b: At) -> f64 {
    let (dx, dy) = (b.x - a.x, b.y - a.y);
    let len = dx.mul_add(dx, dy * dy);
    if len == 0.0 {
        return p.to(a);
    }
    let t = ((p.x - a.x).mul_add(dx, (p.y - a.y) * dy) / len).clamp(0.0, 1.0);
    p.to(At {
        x: t.mul_add(dx, a.x),
        y: t.mul_add(dy, a.y),
    })
}

/// How far `p` is from a path.
fn path_gap(p: At, path: &[Point]) -> f64 {
    let at = path.iter().map(|q| At::of(*q));
    let gaps = at
        .clone()
        .zip(at.skip(1))
        .map(|(a, b)| segment_gap(p, a, b));
    gaps.fold(f64::INFINITY, f64::min)
}
