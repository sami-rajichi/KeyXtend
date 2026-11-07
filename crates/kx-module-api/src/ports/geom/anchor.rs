//! Where the selection strip goes (P10, from the spike's pill): under the last line or above the first, clear of our
//! own windows when there is room, and always on the work area. Boxes come from other apps, so the maths saturates.

use super::{Point, Rect};

/// The strip's size and its gap from the text, in physical pixels.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Strip {
    /// Width.
    pub w: i32,
    /// Height.
    pub h: i32,
    /// Gap between the text and the strip.
    pub gap: i32,
}

/// The strip's top-left: `gap` under the `last` line box, centred on it; above the `first` when there is no room below
/// or it would cover one of `clear` (the keyboard, an open panel); always on `area`.
#[must_use]
pub fn anchor(first: Rect, last: Rect, strip: Strip, area: Rect, clear: &[Rect]) -> Point {
    let below_y = bottom(last).saturating_add(strip.gap);
    let room = below_y.saturating_add(strip.h) <= bottom(area);
    let below = onto(left_of(last, strip), below_y, strip, area);
    let above_y = first.y.saturating_sub(strip.gap).saturating_sub(strip.h);
    let above = onto(left_of(first, strip), above_y, strip, area);
    let free = |at: Point| {
        let placed = Rect {
            x: at.x,
            y: at.y,
            w: strip.w,
            h: strip.h,
        };
        !clear.iter().any(|&c| overlaps(placed, c))
    };
    if room && free(below) {
        below
    } else if free(above) || !room {
        above
    } else {
        below
    }
}

/// The left edge that centres the strip on `line`; the midpoint rounds towards zero and never overflows.
fn left_of(line: Rect, strip: Strip) -> i32 {
    i32::midpoint(line.x, right(line)).saturating_sub(strip.w / 2)
}

/// The point (`x`, `y`) moved onto `area`; min then max, never `clamp`, so an area smaller than the strip cannot panic.
fn onto(x: i32, y: i32, strip: Strip, area: Rect) -> Point {
    Point {
        x: x.min(right(area).saturating_sub(strip.w)).max(area.x),
        y: y.min(bottom(area).saturating_sub(strip.h)).max(area.y),
    }
}

/// The right edge of `r`.
fn right(r: Rect) -> i32 {
    r.x.saturating_add(r.w)
}

/// The bottom edge of `r`.
fn bottom(r: Rect) -> i32 {
    r.y.saturating_add(r.h)
}

/// True when `a` and `b` share a pixel; an empty rectangle shares none.
fn overlaps(a: Rect, b: Rect) -> bool {
    let empty = |r: Rect| r.w <= 0 || r.h <= 0;
    let apart = a.x >= right(b) || b.x >= right(a) || a.y >= bottom(b) || b.y >= bottom(a);
    !empty(a) && !empty(b) && !apart
}

#[cfg(test)]
mod tests;
