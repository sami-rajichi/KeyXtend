//! The frame's rules (plan 2026-11-26 "The frame"), for a region on the frozen screens and a crop on the picture: where
//! each handle sits, which one a click picks up, and where a handle set by a later click leaves the frame. Pure, in the
//! frame's own pixels.

use kx_module_api::ports::{Grip, Handle, Point, Rect};

/// A handle's click area spreads both ways of it, so its reach is its width across over this.
pub const SIDES: f64 = 2.0;

/// Where a handle sits along one axis, which end of it the handle moves.
#[derive(Copy, Clone, PartialEq, Eq)]
enum End {
    /// The left or top edge.
    Near,
    /// The middle; the handle leaves this axis.
    Mid,
    /// The right or bottom edge.
    Far,
}

/// Where `handle` sits across and down.
const fn ends(handle: Handle) -> (End, End) {
    use End::{Far, Mid, Near};
    match handle {
        Handle::TopLeft => (Near, Near),
        Handle::Top => (Mid, Near),
        Handle::TopRight => (Far, Near),
        Handle::Right => (Far, Mid),
        Handle::BottomRight => (Far, Far),
        Handle::Bottom => (Mid, Far),
        Handle::BottomLeft => (Near, Far),
        Handle::Left => (Near, Mid),
    }
}

/// A corner moves both axes, which wins a tie with a side.
fn side(handle: Handle) -> bool {
    let (across, down) = ends(handle);
    across == End::Mid || down == End::Mid
}

/// The point on an axis from `near` to `far` that `end` names.
const fn on(end: End, (near, far): (i64, i64)) -> i64 {
    match end {
        End::Near => near,
        End::Mid => near + (far - near) / 2,
        End::Far => far,
    }
}

/// An axis of `r`, its far edge out, wide enough that no sum overflows: across, or down.
fn span(r: Rect, across: bool) -> (i64, i64) {
    let (at, len) = if across { (r.x, r.w) } else { (r.y, r.h) };
    (i64::from(at), i64::from(at) + i64::from(len))
}

/// Where `handle` sits on `frame`: a corner, or the middle of a side; `None` past the pixel range.
#[must_use]
pub fn spot(frame: Rect, handle: Handle) -> Option<Point> {
    let (across, down) = ends(handle);
    let px = |v: i64| i32::try_from(v).ok();
    Some(Point {
        x: px(on(across, span(frame, true)))?,
        y: px(on(down, span(frame, false)))?,
    })
}

/// The handle whose click area, `reach` pixels each way of it, holds `at`: the nearest when several do, a corner on a
/// tie; `None` off every handle.
#[must_use]
pub fn hit(frame: Rect, at: Point, reach: i32) -> Option<Handle> {
    let reach = i64::from(reach);
    let near = |handle: Handle| {
        let s = spot(frame, handle)?;
        let dx = (i64::from(at.x) - i64::from(s.x)).abs();
        let dy = (i64::from(at.y) - i64::from(s.y)).abs();
        (dx <= reach && dy <= reach).then(|| (dx * dx + dy * dy, side(handle), handle))
    };
    let held = Handle::ALL.into_iter().filter_map(near);
    held.min_by_key(|&(far, side, _)| (far, side))
        .map(|(_, _, handle)| handle)
}

/// `frame` once `grip`'s handle is set at `to`: its sides move by the pointer's travel since the pick-up, swap past the
/// opposite side, stay within `within` and keep at least `min` pixels, or `within`'s side when less. A set on the
/// pick-up spot leaves it as it was; `None` for a `within` turned inside out or past the pixel range.
#[must_use]
pub fn moved(frame: Rect, grip: Grip, to: Point, within: Rect, min: u16) -> Option<Rect> {
    if within.w < 0 || within.h < 0 {
        return None;
    }
    if to == grip.at {
        return Some(frame);
    }
    let (across, down) = ends(grip.handle);
    let min = i64::from(min);
    let travel = |a: i32, b: i32| i64::from(b) - i64::from(a);
    let x = axis(
        span(frame, true),
        across,
        travel(grip.at.x, to.x),
        span(within, true),
        min,
    );
    let y = axis(
        span(frame, false),
        down,
        travel(grip.at.y, to.y),
        span(within, false),
        min,
    );
    let px = |v: i64| i32::try_from(v).ok();
    Some(Rect {
        x: px(x.0)?,
        y: px(y.0)?,
        w: px(x.1 - x.0)?,
        h: px(y.1 - y.0)?,
    })
}

/// One axis from `near` to `far` with the edge `end` names moved by `by`: past the other edge it becomes that side, and
/// the axis stays on `low..=high`, at least `min` long or that whole range when shorter.
fn axis(
    (near, far): (i64, i64),
    end: End,
    by: i64,
    (low, high): (i64, i64),
    min: i64,
) -> (i64, i64) {
    let (fixed, moving) = match end {
        End::Near => (far, near),
        End::Far => (near, far),
        End::Mid => return (near, far),
    };
    if by == 0 {
        return (near, far);
    }
    let to = (moving + by).clamp(low, high);
    // On the fixed edge it keeps the side it came from.
    let after = if to == fixed {
        moving > fixed
    } else {
        to > fixed
    };
    let least = min.min(high - low);
    let (lo, hi) = if after {
        (fixed, to.max(fixed + least))
    } else {
        (to.min(fixed - least), fixed)
    };
    let len = (hi - lo).min(high - low);
    let lo = lo.clamp(low, high - len);
    (lo, lo + len)
}

#[cfg(test)]
mod tests;
