//! The point before the keyboard (ADR-0029 decision 4): the newest point of Windows' pointer history outside the
//! keyboard, else the entry point pushed out past the keyboard's nearest edge. Pure; the guard reads the history.

use kx_module_api::ports::{Point, Rect};

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

/// The newest point of `history` (newest first, as Windows gives it) outside `keyboard` and on the virtual screen
/// `desk`.
pub fn last_outside(
    history: impl IntoIterator<Item = (i32, i32)>,
    keyboard: Rect,
    desk: Rect,
) -> Option<Point> {
    history
        .into_iter()
        .map(|(x, y)| from_history(x, y))
        .find(|&p| contains(desk, p) && !contains(keyboard, p))
}

/// `entry` pushed `edge_px` (one pixel at least) past `keyboard`'s nearest edge; past the opposite edge when that
/// leaves the virtual screen `desk`, then past the other two, then as far as the screen allows. None only when the
/// keyboard covers the whole screen.
#[must_use]
pub fn pushed_out(entry: Point, keyboard: Rect, desk: Rect, edge_px: u32) -> Option<Point> {
    if desk.w <= 0 || desk.h <= 0 {
        return None;
    }
    let push = Push {
        entry: [entry.x, entry.y].map(i64::from),
        kb: edges(keyboard).map(i64::from),
        desk: edges(desk).map(i64::from),
        by: i64::from(edge_px.max(1)),
    };
    let sides = push.order();
    let past = |squeeze| sides.iter().find_map(|&side| push.past(side, squeeze));
    past(false).or_else(|| past(true))
}

/// Where the user pointed before the keyboard: the newest `history` point outside it, else `entry` pushed out.
#[must_use]
pub fn target(
    history: &[(i32, i32)],
    entry: Point,
    keyboard: Rect,
    desk: Rect,
    edge_px: u32,
) -> Option<Point> {
    last_outside(history.iter().copied(), keyboard, desk)
        .or_else(|| pushed_out(entry, keyboard, desk, edge_px))
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
    /// Every side.
    const ALL: [Self; 4] = [Self::Left, Self::Top, Self::Right, Self::Bottom];

    /// The side across from this one.
    const fn opposite(self) -> Self {
        match self {
            Self::Left => Self::Right,
            Self::Right => Self::Left,
            Self::Top => Self::Bottom,
            Self::Bottom => Self::Top,
        }
    }

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

/// One push: the entry point, the keyboard's and the screen's edges (left, top, right, bottom) and the distance.
struct Push {
    entry: [i64; 2],
    kb: [i64; 4],
    desk: [i64; 4],
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

    /// The sides to try: the nearest, its opposite, then the other two, nearest first.
    fn order(&self) -> [Side; 4] {
        let nearer = |a: Side, b: Side| if self.gap(b) < self.gap(a) { b } else { a };
        let first = Side::ALL.into_iter().reduce(nearer).unwrap_or(Side::Left);
        let cross = Side::ALL.into_iter().filter(|s| s.axis() != first.axis());
        let second = cross.reduce(nearer).unwrap_or(Side::Top);
        [first, first.opposite(), second, second.opposite()]
    }

    /// The point `by` past `side`, the entry's other coordinate kept on the screen; with `squeeze` it may come nearer
    /// to stay on the screen. None when it would leave the screen or touch the keyboard.
    fn past(&self, side: Side, squeeze: bool) -> Option<Point> {
        let (axis, other) = (side.axis(), 1 - side.axis());
        let (out, step) = self.out(side);
        let span = |i: usize| (self.desk[i], self.desk[i + 2] - 1);
        let (room, keep) = (span(axis), span(other));
        let far = out + step * (self.by - 1);
        let at = if squeeze {
            far.clamp(room.0, room.1)
        } else {
            far
        };
        if !(room.0..=room.1).contains(&at) || (at - out) * step < 0 {
            return None;
        }
        let mut p = [0; 2];
        p[axis] = at;
        p[other] = self.entry[other].clamp(keep.0, keep.1);
        let [x, y] = p.map(|v| i32::try_from(v).ok());
        Some(Point { x: x?, y: y? })
    }
}

#[cfg(test)]
mod tests;
