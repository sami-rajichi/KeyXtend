//! Where the Voice panel goes by the dictation bar (owner, P11): above the bar, else below it, off the keyboard.

use super::{Edges, Fitted, Size, fit_in, inside, scaled, share};
use kx_module_api::ports::Rect;

/// Where a `size` Voice panel goes by `bar`: whole `gap[0]` above it, else below it, centred on it and off the keyboard
/// `kb` where they share columns, else shrunk to the larger room but never below `least`, covering only then. It stays
/// `gap[1]` inside `work`.
#[must_use]
pub fn by_bar(
    size: Size,
    bar: Rect,
    kb: Option<Rect>,
    work: Rect,
    gap: [i32; 2],
    least: f64,
) -> Fitted {
    let inner = inside(work, gap[1]);
    let (edge, space) = (inside(bar, 0), i64::from(gap[0]));
    let (w, h) = (i64::from(size.w).max(0), i64::from(size.h).max(1));
    let left = column(&edge, w, &inner);
    let kb = kb
        .map(|k| inside(k, 0))
        .filter(|k| left < k.right && k.left < left + w);
    let ceiling = (kb.as_ref().filter(|k| k.top < edge.top))
        .map_or(inner.top, |k| inner.top.max(k.bottom + space));
    let floor = (kb.as_ref().filter(|k| k.bottom > edge.bottom))
        .map_or(inner.bottom, |k| inner.bottom.min(k.top - space));
    let rooms = [edge.top - space - ceiling, floor - edge.bottom - space];
    let fit = share(rooms, h, least);
    let (w, h) = (scaled(w, fit), scaled(h, fit));
    let above = rooms[0] >= h || (rooms[1] < h && rooms[0] >= rooms[1]);
    let top = if above {
        edge.top - space - h
    } else {
        edge.bottom + space
    };
    Fitted {
        x: fit_in(column(&edge, w, &inner), inner.left, inner.right - w),
        y: fit_in(top, inner.top, inner.bottom - h),
        fit,
    }
}

/// The left edge of a `w`-wide panel centred on `bar`, kept inside `inner`.
fn column(bar: &Edges, w: i64, inner: &Edges) -> i64 {
    let x = bar.left + (bar.right - bar.left - w) / 2;
    x.min(inner.right - w).max(inner.left)
}
