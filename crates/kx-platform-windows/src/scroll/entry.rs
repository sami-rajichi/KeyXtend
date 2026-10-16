//! The point before the keyboard (ADR-0029 decision 4): the newest point of Windows' pointer history outside the
//! keyboard, else the entry point pushed out past the keyboard's nearest edge. Pure; the guard reads the history.

use kx_module_api::ports::{Point, Rect, Screen};

use super::low_word;
use crate::screen::{contains, edges};

/// A history point as `GetMouseMovePointsEx` gives display points: 16-bit words, so above `i16::MAX` is negative.
#[must_use]
pub fn from_history(x: i32, y: i32) -> Point {
    let word = |v| i32::from(low_word(v).cast_signed());
    Point {
        x: word(x),
        y: word(y),
    }
}

/// Point `p` as `GetMouseMovePointsEx` looks it up: each coordinate cut to its low 16 bits.
#[must_use]
pub fn to_history(p: Point) -> (i32, i32) {
    let word = |v| i32::from(low_word(v));
    (word(p.x), word(p.y))
}

/// The newest point of `history` (newest first, as Windows gives it) outside `keyboard` and on a work area of
/// `screens`, so the taskbar and the dead zones between screens are passed over.
pub fn last_outside(
    history: impl IntoIterator<Item = (i32, i32)>,
    keyboard: Rect,
    screens: &[Screen],
) -> Option<Point> {
    history
        .into_iter()
        .map(|(x, y)| from_history(x, y))
        .find(|&p| on_screen(screens, p) && !contains(keyboard, p))
}

/// `entry` pushed `edge_px` (one pixel at least) past an edge of `keyboard`, trying the nearest edge first; the first
/// push that lands on a work area of `screens` wins, and none when no push does.
#[must_use]
pub fn pushed_out(entry: Point, keyboard: Rect, screens: &[Screen], edge_px: u32) -> Option<Point> {
    let push = Push {
        entry: [entry.x, entry.y].map(i64::from),
        kb: edges(keyboard).map(i64::from),
        by: i64::from(edge_px.max(1)),
    };
    let mut sides = Side::ALL;
    sides.sort_by_key(|&side| push.gap(side));
    sides
        .into_iter()
        .filter_map(|side| push.past(side))
        .find(|&p| on_screen(screens, p) && !contains(keyboard, p))
}

/// Where the user pointed before the keyboard: the newest `history` point outside it, else `entry` pushed out.
#[must_use]
pub fn target(
    history: &[(i32, i32)],
    entry: Point,
    keyboard: Rect,
    screens: &[Screen],
    edge_px: u32,
) -> Option<Point> {
    last_outside(history.iter().copied(), keyboard, screens)
        .or_else(|| pushed_out(entry, keyboard, screens, edge_px))
}

/// True when `p` lies on the work area of one of `screens`.
fn on_screen(screens: &[Screen], p: Point) -> bool {
    screens.iter().any(|s| contains(s.rect, p))
}

/// A side of the keyboard.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum Side {
    Left,
    Top,
    Right,
    Bottom,
}

impl Side {
    /// Every side, in the order ties go.
    const ALL: [Self; 4] = [Self::Left, Self::Top, Self::Right, Self::Bottom];

    /// The axis this side crosses: 0 for x, 1 for y, as an index into a point and into a box's near edges.
    const fn axis(self) -> usize {
        match self {
            Self::Left | Self::Right => 0,
            Self::Top | Self::Bottom => 1,
        }
    }

    /// True for the left and top sides, which face the axis' start.
    const fn near(self) -> bool {
        matches!(self, Self::Left | Self::Top)
    }
}

/// One push: the entry point, the keyboard's edges (left, top, right, bottom) and the distance, all wide enough that
/// no sum overflows.
struct Push {
    entry: [i64; 2],
    kb: [i64; 4],
    by: i64,
}

impl Push {
    /// The first pixel outside the keyboard past `side`, and the outward step there.
    const fn out(&self, side: Side) -> (i64, i64) {
        let axis = side.axis();
        if side.near() {
            (self.kb[axis] - 1, -1)
        } else {
            (self.kb[axis + 2], 1)
        }
    }

    /// How far the entry point lies inside `side`; below 0 when it is already past it.
    const fn gap(&self, side: Side) -> i64 {
        let (out, step) = self.out(side);
        (self.entry[side.axis()] - out) * -step - 1
    }

    /// The point `by` past `side`, with the entry's other coordinate; none past the ends of the pixel range.
    fn past(&self, side: Side) -> Option<Point> {
        let (out, step) = self.out(side);
        let mut p = self.entry;
        p[side.axis()] = out + step * (self.by - 1);
        let [x, y] = p.map(|v| i32::try_from(v).ok());
        Some(Point { x: x?, y: y? })
    }
}

#[cfg(test)]
mod props;
#[cfg(test)]
mod tests;
