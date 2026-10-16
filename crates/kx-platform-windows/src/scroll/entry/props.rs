//! Properties over random screens with dead zones between them: a missed history point is the nearest edge's push
//! that lands on a work area, never on the keyboard; history words round trip.

use super::*;
use crate::screen::rect_of;
use proptest::collection::vec;
use proptest::prelude::*;

/// Random desks per property.
const CASES: u32 = 512;
/// Screen corners lie within this many pixels of the origin, either way.
const SPAN: i32 = 20_000;
/// Most screens on one desk.
const MAX_SCREENS: usize = 3;
/// The keyboard reaches past its screen by up to this share of it, as 1 / `REACH`.
const REACH: i32 = 8;
/// The largest `edge_px` tried, the setting's top.
const MAX_EDGE: u32 = 64;

/// A screen anywhere near the origin.
fn screen() -> impl Strategy<Value = Screen> {
    (-SPAN..SPAN, -SPAN..SPAN, 1..SPAN, 1..SPAN).prop_map(|(x, y, w, h)| Screen {
        rect: Rect { x, y, w, h },
        dpi: crate::screen::DEFAULT_DPI,
        primary: false,
    })
}

/// Screens, a keyboard around the first one, and an entry point inside the keyboard when `inside` and it has room.
fn desk() -> impl Strategy<Value = (Vec<Screen>, Rect, Point)> {
    vec(screen(), 1..=MAX_SCREENS)
        .prop_flat_map(|screens| {
            let Rect { x, y, w, h } = screens[0].rect;
            let (xs, ys) = (
                x - w / REACH..x + w + w / REACH,
                y - h / REACH..y + h + h / REACH,
            );
            let corners = (xs.clone(), ys.clone(), xs.clone(), ys.clone());
            (Just(screens), corners, (xs, ys), any::<bool>())
        })
        .prop_map(|(screens, (x0, y0, x1, y1), (ex, ey), inside)| {
            let kb = rect_of([x0.min(x1), y0.min(y1), x0.max(x1), y0.max(y1)]);
            let entry = if inside && kb.w > 0 && kb.h > 0 {
                Point {
                    x: kb.x + ex.rem_euclid(kb.w),
                    y: kb.y + ey.rem_euclid(kb.h),
                }
            } else {
                Point { x: ex, y: ey }
            };
            (screens, kb, entry)
        })
}

/// The push worked out apart from the code: each edge's point and gap, nearest first (left, top, right, bottom on a
/// tie), and the first on a work area off the keyboard.
fn want(entry: Point, kb: Rect, screens: &[Screen], edge: u32) -> Option<Point> {
    let d = i32::try_from(edge.max(1)).unwrap();
    let (right, bottom) = (kb.x + kb.w - 1, kb.y + kb.h - 1);
    let pt = |x, y| Point { x, y };
    let mut tries = [
        (entry.x - kb.x, pt(kb.x - d, entry.y)),
        (entry.y - kb.y, pt(entry.x, kb.y - d)),
        (right - entry.x, pt(right + d, entry.y)),
        (bottom - entry.y, pt(entry.x, bottom + d)),
    ];
    tries.sort_by_key(|&(gap, _)| gap);
    let on = |p: Point| screens.iter().any(|s| contains(s.rect, p)) && !contains(kb, p);
    tries.into_iter().map(|(_, p)| p).find(|&p| on(p))
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(CASES))]

    #[test]
    fn a_missed_history_point_is_the_nearest_push_on_a_work_area(
        (screens, kb, entry) in desk(),
        edge in 0..=MAX_EDGE,
    ) {
        let got = pushed_out(entry, kb, &screens, edge);
        prop_assert_eq!(got, want(entry, kb, &screens, edge));
        if let Some(at) = got {
            prop_assert!(screens.iter().any(|s| contains(s.rect, at)), "on no screen");
            prop_assert!(!contains(kb, at), "on the keyboard");
        }
    }

    #[test]
    fn history_words_round_trip(x in any::<i16>(), y in any::<i16>()) {
        let at = Point { x: i32::from(x), y: i32::from(y) };
        let (wx, wy) = to_history(at);
        prop_assert!((0..=i32::from(u16::MAX)).contains(&wx));
        prop_assert!((0..=i32::from(u16::MAX)).contains(&wy));
        prop_assert_eq!(from_history(wx, wy), at);
    }
}
