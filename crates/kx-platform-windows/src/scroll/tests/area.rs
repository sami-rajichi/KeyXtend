//! The box UI Automation gives, checked against the desk: kept up to the desk's own size past it, else refused.

use super::super::area;
use kx_module_api::ports::Rect;
use proptest::prelude::*;

/// Two screens side by side: x -1920 to 2559, y -360 to 1439, so it is 4480 wide and 1800 high.
const WHOLE: Rect = Rect {
    x: -1920,
    y: -360,
    w: 4480,
    h: 1800,
};
/// The furthest box `area` keeps on `WHOLE`: one whole size past each edge.
const FURTHEST: [i32; 4] = [-1920 - 4480, -360 - 1800, 2560 + 4480, 1440 + 1800];

#[test]
fn a_box_on_the_desk_or_up_to_its_size_past_it_is_kept() {
    let kept = area([100, 200, 900, 700], WHOLE);
    let want = Rect {
        x: 100,
        y: 200,
        w: 800,
        h: 500,
    };
    assert_eq!(kept, Some(want));
    let [l, t, r, b] = FURTHEST;
    let far = Rect {
        x: l,
        y: t,
        w: r - l,
        h: b - t,
    };
    assert_eq!(area(FURTHEST, WHOLE), Some(far));
}

#[test]
fn an_empty_inside_out_or_huge_box_is_refused() {
    assert_eq!(area([5, 5, 5, 50], WHOLE), None, "no width");
    assert_eq!(area([5, 5, 50, 5], WHOLE), None, "no height");
    assert_eq!(area([50, 5, 5, 50], WHOLE), None, "inside out");
    assert_eq!(area([5, 50, 50, 5], WHOLE), None, "upside down");
    for side in 0..4 {
        let mut past = FURTHEST;
        past[side] += if side < 2 { -1 } else { 1 };
        assert_eq!(area(past, WHOLE), None, "one pixel too far, side {side}");
    }
    let (lo, hi) = (i32::MIN, i32::MAX);
    assert_eq!(area([lo, lo, hi, hi], WHOLE), None);
    assert_eq!(area([hi - 1, 0, hi, 10], WHOLE), None);
    assert_eq!(area([0, 0, 10, 10], Rect::default()), None, "no desk");
}

proptest! {
    #[test]
    fn a_kept_box_is_never_empty_nor_further_than_the_desks_size(edges: [i32; 4]) {
        if let Some(r) = area(edges, WHOLE) {
            let [l, t, r_, b] = crate::screen::edges(r).map(i64::from);
            prop_assert!(r.w > 0 && r.h > 0);
            prop_assert_eq!([l, t, r_, b], edges.map(i64::from));
            let far = FURTHEST.map(i64::from);
            prop_assert!(l >= far[0] && t >= far[1] && r_ <= far[2] && b <= far[3]);
        }
    }
}
