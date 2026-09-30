//! Window maths the UI does itself: boxes QML sends, the screen a window is on, and where the bubble and the pop-ups go.

use kx_module_api::ports::{Corner, Point, Rect, Screen};
use serde::Deserialize;

/// A box QML sends, in its own units from the window's top-left corner.
#[derive(Copy, Clone, Debug, Default, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Area {
    /// Left edge.
    pub x: f64,
    /// Top edge.
    pub y: f64,
    /// Width.
    pub w: f64,
    /// Height.
    pub h: f64,
}

/// `v` as a pixel: it saturates at the ends of the range, and NaN is 0.
fn px(v: f64) -> i32 {
    #[allow(
        clippy::cast_possible_truncation,
        reason = "`as` saturates at the ends of the range and gives 0 for NaN"
    )]
    let p = v as i32;
    p
}

/// `v` rounded to a whole pixel; it saturates at the ends of the range, and NaN is 0.
#[must_use]
#[cfg_attr(
    not(feature = "qt"),
    allow(dead_code, reason = "only the Qt bridges round QML's numbers")
)]
pub fn whole(v: f64) -> i32 {
    px(v.round())
}

/// `v` as a pixel, stopping at the ends of the range.
pub(crate) fn clamp(v: i64) -> i32 {
    i32::try_from(v).unwrap_or(if v < 0 { i32::MIN } else { i32::MAX })
}

/// `a` in physical pixels at scale `dpr`, rounded outwards so the whole box is covered.
#[must_use]
pub fn outward(a: &Area, dpr: f64) -> Rect {
    let (left, top) = (px((a.x * dpr).floor()), px((a.y * dpr).floor()));
    let right = px(((a.x + a.w) * dpr).ceil());
    let bottom = px(((a.y + a.h) * dpr).ceil());
    Rect {
        x: left,
        y: top,
        w: clamp(i64::from(right) - i64::from(left)).max(0),
        h: clamp(i64::from(bottom) - i64::from(top)).max(0),
    }
}

/// The screen whose work area holds `at`, else the primary, else the first.
#[must_use]
pub fn screen_at(at: Point, screens: &[Screen]) -> Option<&Screen> {
    let holds = |s: &&Screen| {
        let (x, y) = (i64::from(at.x), i64::from(at.y));
        let (left, top) = (i64::from(s.rect.x), i64::from(s.rect.y));
        (left..left + i64::from(s.rect.w)).contains(&x)
            && (top..top + i64::from(s.rect.h)).contains(&y)
    };
    screens.iter().find(holds).or_else(|| main_screen(screens))
}

/// The main screen: the primary one, else the first.
#[must_use]
pub fn main_screen(screens: &[Screen]) -> Option<&Screen> {
    screens.iter().find(|s| s.primary).or(screens.first())
}

/// The middle of `r`, never past the ends of the range.
#[must_use]
pub fn centre(r: Rect) -> Point {
    let mid = |at: i32, len: i32| clamp(i64::from(at) + i64::from(len) / 2);
    Point {
        x: mid(r.x, r.w),
        y: mid(r.y, r.h),
    }
}

/// Where a `d`-wide bubble goes: `inset` inside the bottom `corner` of `work`, never off it.
#[must_use]
pub fn bubble(work: Rect, corner: Corner, d: i32, inset: i32) -> Point {
    let (left, top) = (i64::from(work.x), i64::from(work.y));
    let (right, bottom) = (left + i64::from(work.w), top + i64::from(work.h));
    let (d, inset) = (i64::from(d), i64::from(inset));
    let x = match corner {
        Corner::Left => left + inset,
        Corner::Right => right - d - inset,
    };
    Point {
        x: clamp(x.min(right - d).max(left)),
        y: clamp((bottom - d - inset).max(top)),
    }
}

/// A window's size in physical pixels.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Size {
    /// Width.
    pub w: i32,
    /// Height.
    pub h: i32,
}

/// `v` kept from `lo` to `hi`; `lo` wins when there is no room between them.
fn fit(v: i64, lo: i64, hi: i64) -> i32 {
    clamp(v.min(hi).max(lo))
}

/// The four edges of a rectangle, wide enough that sums never overflow.
struct Edges {
    left: i64,
    top: i64,
    right: i64,
    bottom: i64,
}

/// `work`'s edges moved `edge` inside it.
fn inside(work: Rect, edge: i32) -> Edges {
    let (x, y, e) = (i64::from(work.x), i64::from(work.y), i64::from(edge));
    Edges {
        left: x + e,
        top: y + e,
        right: x + i64::from(work.w) - e,
        bottom: y + i64::from(work.h) - e,
    }
}

/// Where a `size` pop-up goes: `gap[0]` above `over`, else below it, else at the top, with its left edge at `left`,
/// and at least `gap[1]` inside `work` (mock-up `placePop`).
#[must_use]
pub fn beside(size: Size, left: i32, over: Rect, work: Rect, gap: [i32; 2]) -> Point {
    let e = inside(work, gap[1]);
    let (w, h, space) = (i64::from(size.w), i64::from(size.h), i64::from(gap[0]));
    let above = i64::from(over.y) - space - h;
    let below = i64::from(over.y) + i64::from(over.h) + space;
    let top = if above >= e.top {
        above
    } else if below + h <= e.bottom {
        below
    } else {
        e.top
    };
    Point {
        x: fit(i64::from(left), e.left, e.right - w),
        y: fit(top, e.top, e.bottom - h),
    }
}

/// Where a `size` pop-up goes with no keyboard to sit beside: the bottom centre of `work`, `edge` inside it.
#[must_use]
pub fn home(size: Size, work: Rect, edge: i32) -> Point {
    let e = inside(work, edge);
    let (w, h) = (i64::from(size.w), i64::from(size.h));
    let x = i64::from(work.x) + (i64::from(work.w) - w) / 2;
    Point {
        x: fit(x, e.left, e.right - w),
        y: fit(e.bottom - h, e.top, e.bottom - h),
    }
}

/// Where a `size` menu goes: its bottom-left corner at `at`, moved at least `edge` inside `work`.
#[must_use]
pub fn menu(size: Size, at: Point, work: Rect, edge: i32) -> Point {
    let e = inside(work, edge);
    let (w, h) = (i64::from(size.w), i64::from(size.h));
    Point {
        x: fit(i64::from(at.x), e.left, e.right - w),
        y: fit(i64::from(at.y) - h, e.top, e.bottom - h),
    }
}

/// The screen whose work area is nearest `at`, so a keyboard partly off every work area still has one; the first
/// wins a tie.
#[must_use]
pub fn screen_near(at: Point, screens: &[Screen]) -> Option<&Screen> {
    let off = |s: &&Screen| {
        let (x, y) = (i64::from(at.x), i64::from(at.y));
        let e = inside(s.rect, 0);
        let dx = (e.left - x).max(x - (e.right - 1)).max(0);
        let dy = (e.top - y).max(y - (e.bottom - 1)).max(0);
        dx.max(dy)
    };
    screens.iter().min_by_key(off)
}

#[cfg(test)]
mod tests;
