//! Where the strip goes, from boxes other apps told (P10 design "Where it sits"): the selection's screen must be known
//! and its boxes upright, and the strip QML measured must fit that screen's work area. The maths saturates.

use kx_module_api::ports::{Point, Rect, Strip, StripSpot, anchor};

/// The least side a box counts as, so a caret's zero-wide box still touches its screen.
const LEAST_PX: i32 = 1;

/// True when `spot` can place a strip: its screen is known, no box is turned inside out, and the last line, which
/// picked the screen, touches its work area.
pub(super) fn sane(spot: &StripSpot) -> bool {
    let screen = spot.screen;
    let known = screen.dpi > 0 && screen.rect.w > 0 && screen.rect.h > 0;
    let upright = |r: Rect| r.w >= 0 && r.h >= 0;
    known && upright(spot.first) && upright(spot.last) && touches(spot.last, screen.rect)
}

/// The top-left of a `w` by `h` strip by `spot`, `gap` from its line and clear of `clear` when there is room; `None`
/// when it does not fit the work area.
pub(super) fn at(spot: &StripSpot, w: i32, h: i32, gap: i32, clear: &[Rect]) -> Option<Point> {
    let area = spot.screen.rect;
    let fits = (LEAST_PX..=area.w).contains(&w) && (LEAST_PX..=area.h).contains(&h);
    fits.then(|| anchor(spot.first, spot.last, Strip { w, h, gap }, area, clear))
}

/// True when `r`, at least `LEAST_PX` each way, shares a pixel with `area`; widened, so no edge overflows.
fn touches(r: Rect, area: Rect) -> bool {
    let span = |at: i32, len: i32| (i64::from(at), i64::from(at) + i64::from(len));
    let (left, right) = span(r.x, r.w.max(LEAST_PX));
    let (top, bottom) = span(r.y, r.h.max(LEAST_PX));
    let (a_left, a_right) = span(area.x, area.w);
    let (a_top, a_bottom) = span(area.y, area.h);
    left < a_right && right > a_left && top < a_bottom && bottom > a_top
}
