//! Where the strip goes: the spike's cases (G19 placed them within 1 px), the rectangles it keeps clear of, and
//! properties for any box another app may send.

use super::*;
use proptest::prelude::*;

/// A full HD screen's area.
const AREA: Rect = Rect {
    x: 0,
    y: 0,
    w: 1920,
    h: 1080,
};
/// The spike's pill: 80 by 30, 8 px from the text.
const STRIP: Strip = Strip {
    w: 80,
    h: 30,
    gap: 8,
};

/// A rectangle from its edges, as the spike's tests wrote them.
fn r(left: i32, top: i32, right: i32, bottom: i32) -> Rect {
    Rect {
        x: left,
        y: top,
        w: right - left,
        h: bottom - top,
    }
}

/// Where the strip goes for one line box and nothing to keep clear of.
fn on_one(line: Rect) -> Point {
    anchor(line, line, STRIP, AREA, &[])
}

/// True when the strip at `at` covers `other`.
fn covers(at: Point, other: Rect) -> bool {
    let placed = Rect {
        x: at.x,
        y: at.y,
        w: STRIP.w,
        h: STRIP.h,
    };
    overlaps(placed, other)
}

#[test]
fn the_strip_sits_under_the_last_line_centred_on_it() {
    let (first, last) = (r(100, 100, 500, 120), r(100, 120, 300, 140));
    let at = anchor(first, last, STRIP, AREA, &[]);
    assert_eq!(at, Point { x: 160, y: 148 });
}

#[test]
fn the_strip_stays_on_the_area_when_the_box_leaves_it() {
    assert_eq!(on_one(r(1900, 100, 2000, 120)), Point { x: 1840, y: 128 });
    assert_eq!(on_one(r(-50, -20, 10, 0)), Point { x: 0, y: 8 });
}

#[test]
fn the_strip_goes_above_the_first_line_when_below_has_no_room() {
    let (first, last) = (r(100, 1030, 500, 1050), r(100, 1050, 200, 1070));
    let at = anchor(first, last, STRIP, AREA, &[]);
    assert_eq!(at, Point { x: 260, y: 992 });
}

#[test]
fn huge_boxes_from_another_app_never_overflow() {
    let huge = r(i32::MAX - 1, i32::MAX - 1, i32::MAX, i32::MAX);
    assert_eq!(on_one(huge), Point { x: 1840, y: 1050 });
    let low = r(i32::MIN, i32::MIN, i32::MIN + 1, i32::MIN + 1);
    assert_eq!(on_one(low), Point { x: 0, y: 0 });
}

#[test]
fn the_strip_goes_above_when_it_would_cover_the_keyboard() {
    let keyboard = r(0, 700, 1920, 1080);
    let (first, last) = (r(100, 640, 500, 660), r(100, 660, 300, 680));
    let at = anchor(first, last, STRIP, AREA, &[keyboard]);
    assert_eq!(at, Point { x: 260, y: 602 });
    assert!(!covers(at, keyboard));
}

#[test]
fn with_no_room_above_it_stays_below_over_the_keyboard_never_over_the_text() {
    let keyboard = r(0, 60, 1920, 400);
    let line = r(100, 20, 300, 40);
    let at = anchor(line, line, STRIP, AREA, &[keyboard]);
    assert_eq!(at, Point { x: 160, y: 48 }, "over the keyboard's edge");
    assert!(!covers(at, line));
}

#[test]
fn a_panel_beside_the_spot_leaves_it_below() {
    let panel = r(1500, 0, 1920, 1080);
    let line = r(100, 100, 300, 120);
    let at = anchor(line, line, STRIP, AREA, &[panel]);
    assert_eq!(at, Point { x: 160, y: 128 });
}

#[test]
fn with_both_spots_covered_it_keeps_the_spikes_rule() {
    let line = r(100, 500, 300, 520);
    let all = [r(0, 0, 1920, 1080)];
    assert_eq!(
        anchor(line, line, STRIP, AREA, &all),
        Point { x: 160, y: 528 }
    );
    let low = r(100, 1060, 300, 1075);
    assert_eq!(
        anchor(low, low, STRIP, AREA, &all),
        Point { x: 160, y: 1022 }
    );
}

#[test]
fn an_empty_rectangle_covers_nothing() {
    let line = r(100, 100, 300, 120);
    let empty = [r(170, 130, 170, 150), r(170, 140, 200, 140)];
    assert_eq!(
        anchor(line, line, STRIP, AREA, &empty),
        Point { x: 160, y: 128 }
    );
}

#[test]
fn an_area_smaller_than_the_strip_puts_it_at_the_areas_corner() {
    let tiny = r(10, 20, 30, 40);
    let line = r(100, 100, 300, 120);
    assert_eq!(anchor(line, line, STRIP, tiny, &[]), Point { x: 10, y: 20 });
}

/// Any rectangle, as another app may send it.
fn any_rect() -> impl Strategy<Value = Rect> {
    (any::<i32>(), any::<i32>(), any::<i32>(), any::<i32>()).prop_map(|(x, y, w, h)| Rect {
        x,
        y,
        w,
        h,
    })
}

/// A rectangle near a screen, with a size that may be zero.
fn near_rect() -> impl Strategy<Value = Rect> {
    (-3000..6000, -3000..4000, 0..2000, 0..200).prop_map(|(x, y, w, h)| Rect { x, y, w, h })
}

proptest! {
    #[test]
    fn any_boxes_give_a_spot_without_overflow_at_or_past_the_areas_corner(
        first in any_rect(),
        last in any_rect(),
        (w, h, gap) in (any::<i32>(), any::<i32>(), any::<i32>()),
        area in any_rect(),
        clear in prop::collection::vec(any_rect(), 0..3),
    ) {
        let at = anchor(first, last, Strip { w, h, gap }, area, &clear);
        prop_assert!(at.x >= area.x && at.y >= area.y);
    }

    #[test]
    fn a_strip_that_fits_the_area_stays_wholly_on_it(
        first in near_rect(),
        last in near_rect(),
        (w, h, gap) in (1..400, 1..80, 0..40),
        clear in prop::collection::vec(near_rect(), 0..3),
    ) {
        let at = anchor(first, last, Strip { w, h, gap }, AREA, &clear);
        prop_assert!(at.x >= AREA.x && at.x + w <= AREA.x + AREA.w);
        prop_assert!(at.y >= AREA.y && at.y + h <= AREA.y + AREA.h);
    }

    #[test]
    fn with_nothing_to_clear_and_room_below_it_sits_gap_under_the_last_line(
        first in near_rect(),
        (x, y, lw, lh) in (0..1900, 0..900, 0..400, 1..40),
    ) {
        let last = Rect { x, y, w: lw, h: lh };
        let at = anchor(first, last, STRIP, AREA, &[]);
        prop_assert_eq!(at.y, y + lh + STRIP.gap);
        let centred = x + lw / 2 - STRIP.w / 2;
        // The bounds are constants in order, so `clamp` cannot panic here.
        prop_assert_eq!(at.x, centred.clamp(AREA.x, AREA.x + AREA.w - STRIP.w));
    }

    #[test]
    fn a_keyboard_under_the_line_sends_the_strip_above_when_there_is_room(
        (x, lw, lh) in (0..1900, 0..400, 1..40),
        top in 300..1000,
        near in 0..(STRIP.gap + STRIP.h),
    ) {
        let keyboard = Rect { x: 0, y: top, w: AREA.w, h: AREA.h - top };
        let line = Rect { x, y: top - lh - near, w: lw, h: lh };
        prop_assume!(line.y - STRIP.gap - STRIP.h >= AREA.y);
        let at = anchor(line, line, STRIP, AREA, &[keyboard]);
        prop_assert!(!covers(at, keyboard));
        prop_assert_eq!(at.y, line.y - STRIP.gap - STRIP.h);
    }

    #[test]
    fn with_no_room_above_it_stays_under_the_last_line_even_over_the_keyboard(
        (x, lw, lh) in (0..1900, 0..400, 1..40),
        y in 0..(STRIP.gap + STRIP.h),
        near in 0..(STRIP.gap + STRIP.h),
    ) {
        let line = Rect { x, y, w: lw, h: lh };
        let keyboard = Rect { x: 0, y: y + lh + near, w: AREA.w, h: AREA.h / 2 };
        let at = anchor(line, line, STRIP, AREA, &[keyboard]);
        prop_assert_eq!(at.y, y + lh + STRIP.gap);
        prop_assert!(!covers(at, line));
    }
}
