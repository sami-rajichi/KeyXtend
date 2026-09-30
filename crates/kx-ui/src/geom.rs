//! Window maths the UI does itself: boxes QML sends, the screen a window is on, and the minimise bubble's spot.

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
fn clamp(v: i64) -> i32 {
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
    let main = || screens.iter().find(|s| s.primary).or(screens.first());
    screens.iter().find(holds).or_else(main)
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

#[cfg(test)]
mod tests;
