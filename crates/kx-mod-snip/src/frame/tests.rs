//! The frame's rules (plan 2026-11-26 "The frame", edges 1-3): each handle's place and click area, each handle moving
//! its own sides, a swap past the opposite side, a frame kept in its box and at least `min`, and a set where it was
//! picked up changing nothing.

use super::*;

mod props;

/// The frame the tests move.
const FRAME: Rect = Rect {
    x: 100,
    y: 100,
    w: 200,
    h: 100,
};
/// The box it keeps within.
const WITHIN: Rect = Rect {
    x: 0,
    y: 0,
    w: 1_000,
    h: 600,
};
/// The click area's reach each way of a handle.
const REACH: i32 = 8;
/// The smallest side.
const MIN: u16 = 8;

fn at(x: i32, y: i32) -> Point {
    Point { x, y }
}

fn rect(x: i32, y: i32, w: i32, h: i32) -> Rect {
    Rect { x, y, w, h }
}

/// `handle` picked up on its own spot of `frame`.
fn grip(frame: Rect, handle: Handle) -> Grip {
    let at = spot(frame, handle).expect("a spot");
    Grip { handle, at }
}

/// `frame` with `handle` moved by (`dx`, `dy`) inside `WITHIN`.
fn by(frame: Rect, handle: Handle, dx: i32, dy: i32) -> Option<Rect> {
    let g = grip(frame, handle);
    moved(frame, g, at(g.at.x + dx, g.at.y + dy), WITHIN, MIN)
}

#[test]
fn each_handle_sits_at_its_corner_or_the_middle_of_its_side() {
    let spots = Handle::ALL.map(|h| spot(FRAME, h));
    let want = [
        at(100, 100),
        at(200, 100),
        at(300, 100),
        at(300, 150),
        at(300, 200),
        at(200, 200),
        at(100, 200),
        at(100, 150),
    ];
    assert_eq!(spots, want.map(Some));
}

#[test]
fn a_click_within_reach_of_a_handle_picks_it_up_and_one_past_it_does_not() {
    for h in Handle::ALL {
        let s = spot(FRAME, h).expect("a spot");
        assert_eq!(hit(FRAME, s, REACH), Some(h), "{h:?} on its spot");
        let edge = at(s.x + REACH, s.y - REACH);
        assert_eq!(hit(FRAME, edge, REACH), Some(h), "{h:?} at its reach");
        let past = at(s.x + REACH + 1, s.y - REACH - 1);
        assert_eq!(hit(FRAME, past, REACH), None, "{h:?} past its reach");
    }
}

#[test]
fn a_click_inside_the_frame_or_away_from_it_picks_no_handle() {
    assert_eq!(hit(FRAME, at(150, 130), REACH), None, "inside");
    assert_eq!(hit(FRAME, at(20, 20), REACH), None, "away");
}

#[test]
fn a_click_at_the_far_end_of_the_pixel_range_picks_no_handle_and_never_overflows() {
    let corner = rect(i32::MAX - 10, i32::MAX - 10, 10, 10);
    assert_eq!(hit(corner, at(i32::MIN, i32::MIN), REACH), None);
}

#[test]
fn on_a_frame_smaller_than_the_reach_the_nearest_handle_wins_and_a_corner_on_a_tie() {
    let small = rect(0, 0, 4, 4);
    assert_eq!(hit(small, at(0, 0), REACH), Some(Handle::TopLeft));
    assert_eq!(hit(small, at(2, -1), REACH), Some(Handle::Top));
    assert_eq!(
        hit(small, at(1, 0), REACH),
        Some(Handle::TopLeft),
        "as near the top's middle"
    );
}

#[test]
fn each_handle_moves_its_own_sides_by_the_pointers_travel() {
    let (dx, dy) = (10, 7);
    let want = [
        rect(110, 107, 190, 93),
        rect(100, 107, 200, 93),
        rect(100, 107, 210, 93),
        rect(100, 100, 210, 100),
        rect(100, 100, 210, 107),
        rect(100, 100, 200, 107),
        rect(110, 100, 190, 107),
        rect(110, 100, 190, 100),
    ];
    for (h, want) in Handle::ALL.into_iter().zip(want) {
        assert_eq!(by(FRAME, h, dx, dy), Some(want), "{h:?}");
    }
}

#[test]
fn a_handle_picked_up_off_its_spot_moves_by_the_travel_not_to_the_pointer() {
    let g = Grip {
        handle: Handle::Right,
        at: at(305, 160),
    };
    let set = moved(FRAME, g, at(325, 140), WITHIN, MIN);
    assert_eq!(set, Some(rect(100, 100, 220, 100)));
}

#[test]
fn edge_1_a_handle_set_past_the_opposite_side_swaps_them() {
    assert_eq!(
        by(FRAME, Handle::Left, 250, 0),
        Some(rect(300, 100, 50, 100))
    );
    assert_eq!(
        by(FRAME, Handle::Bottom, 0, -160),
        Some(rect(100, 40, 200, 60))
    );
    let flipped = by(FRAME, Handle::BottomRight, -260, -130);
    assert_eq!(flipped, Some(rect(40, 70, 60, 30)), "both axes");
}

#[test]
fn edge_2_a_frame_stops_at_its_box() {
    assert_eq!(
        by(FRAME, Handle::Right, 2_000, 0),
        Some(rect(100, 100, 900, 100))
    );
    assert_eq!(
        by(FRAME, Handle::TopLeft, -500, -500),
        Some(rect(0, 0, 300, 200))
    );
}

#[test]
fn edge_2_a_frame_keeps_at_least_min_on_each_side_it_moves() {
    assert_eq!(
        by(FRAME, Handle::Left, 197, 0),
        Some(rect(292, 100, 8, 100))
    );
    assert_eq!(
        by(FRAME, Handle::Left, 203, 0),
        Some(rect(300, 100, 8, 100)),
        "past it"
    );
    assert_eq!(
        by(FRAME, Handle::Top, 0, 100),
        Some(rect(100, 192, 200, 8)),
        "onto it"
    );
}

#[test]
fn edge_2_a_frame_at_its_boxs_edge_keeps_min_inside_the_box() {
    let edge = rect(990, 100, 10, 100);
    assert_eq!(by(edge, Handle::Left, 9, 0), Some(rect(992, 100, 8, 100)));
    assert_eq!(by(edge, Handle::Left, 40, 0), Some(rect(992, 100, 8, 100)));
    let narrow = rect(0, 0, 5, 600);
    let g = grip(narrow, Handle::Right);
    let set = moved(narrow, g, at(g.at.x - 3, g.at.y), narrow, MIN);
    assert_eq!(set, Some(narrow), "a box under min gives its whole side");
}

#[test]
fn edge_3_a_set_where_the_handle_was_picked_up_leaves_the_frame_as_it_was() {
    let tiny = rect(50, 50, 3, 3);
    for h in Handle::ALL {
        let g = Grip {
            handle: h,
            at: at(51, 49),
        };
        assert_eq!(moved(tiny, g, g.at, WITHIN, MIN), Some(tiny), "{h:?}");
    }
}

#[test]
fn a_box_turned_inside_out_moves_nothing() {
    let g = grip(FRAME, Handle::Right);
    let bad = rect(0, 0, -1, 600);
    assert_eq!(moved(FRAME, g, at(g.at.x + 5, g.at.y), bad, MIN), None);
}
