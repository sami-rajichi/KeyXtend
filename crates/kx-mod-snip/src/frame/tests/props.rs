//! Properties over any frame in its box, any handle and any set (edges 1-3): the frame stays in its box, never inside
//! out, each moved side keeps at least `min`, the other axis stays, a set where it was picked up changes nothing, and a
//! handle's own spot picks it up.

use super::*;
use proptest::prelude::*;

/// Random cases per property.
const CASES: u32 = 512;

/// Any handle.
fn handle() -> impl Strategy<Value = Handle> {
    proptest::sample::select(Handle::ALL.to_vec())
}

/// A frame inside `WITHIN`.
fn inside() -> impl Strategy<Value = Rect> {
    (0..WITHIN.w, 0..WITHIN.h)
        .prop_flat_map(|(x, y)| (Just(x), Just(y), 1..=WITHIN.w - x, 1..=WITHIN.h - y))
        .prop_map(|(x, y, w, h)| rect(x, y, w, h))
}

/// A point on or around `WITHIN`.
fn point() -> impl Strategy<Value = Point> {
    (-200..1_200_i32, -200..800_i32).prop_map(|(x, y)| at(x, y))
}

/// The ends of one axis of `r`: across or down.
fn axis(r: Rect, across: bool) -> (i32, i32) {
    if across {
        (r.x, r.x + r.w)
    } else {
        (r.y, r.y + r.h)
    }
}

/// Whether `h` moves the frame across or down.
fn moves(h: Handle, across: bool) -> bool {
    use Handle::{Bottom, Left, Right, Top};
    if across {
        !matches!(h, Top | Bottom)
    } else {
        !matches!(h, Left | Right)
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(CASES))]

    #[test]
    fn a_set_frame_stays_in_its_box_with_each_moved_side_at_least_min(
        frame in inside(),
        h in handle(),
        from in point(),
        to in point(),
    ) {
        let set = moved(frame, Grip { handle: h, at: from }, to, WITHIN, MIN);
        let Some(set) = set else {
            return Err(TestCaseError::fail("no frame"));
        };
        for across in [true, false] {
            let ((lo, hi), (low, high)) = (axis(set, across), axis(WITHIN, across));
            prop_assert!(low <= lo && lo < hi && hi <= high, "{set:?} in its box");
            let travel = if across { to.x - from.x } else { to.y - from.y };
            if moves(h, across) && travel != 0 {
                prop_assert!(hi - lo >= i32::from(MIN), "{set:?} at least min");
            } else {
                prop_assert_eq!((lo, hi), axis(frame, across), "an axis it leaves");
            }
        }
    }

    #[test]
    fn a_set_on_the_pick_up_spot_changes_nothing(frame in inside(), h in handle(), from in point()) {
        let set = moved(frame, Grip { handle: h, at: from }, from, WITHIN, MIN);
        prop_assert_eq!(set, Some(frame));
    }

    #[test]
    fn a_handles_own_spot_picks_it_up_on_a_frame_wider_than_its_reach(
        frame in inside(),
        h in handle(),
    ) {
        prop_assume!(frame.w > 2 * REACH && frame.h > 2 * REACH);
        let s = spot(frame, h).expect("a spot");
        prop_assert_eq!(hit(frame, s, REACH), Some(h));
    }

    #[test]
    fn a_click_past_every_handles_reach_picks_none(frame in inside(), p in point()) {
        let near = Handle::ALL.into_iter().filter_map(|h| spot(frame, h)).any(|s| {
            (p.x - s.x).abs() <= REACH && (p.y - s.y).abs() <= REACH
        });
        prop_assert_eq!(hit(frame, p, REACH).is_some(), near);
    }
}
