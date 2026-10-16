//! The point before the keyboard, on a desk of two screens: the main one, and one left of it and higher.

use super::*;

/// The virtual screen in physical pixels: a 2560 × 1440 main screen at 125 %, and a 1920 × 1080 one at 100 % to its
/// left, 360 pixels higher.
const DESK: Rect = Rect {
    x: -1920,
    y: -360,
    w: 4480,
    h: 1800,
};
/// The keyboard near the bottom of the main screen: x 400 to 1999, y 1000 to 1399.
const KB: Rect = Rect {
    x: 400,
    y: 1000,
    w: 1600,
    h: 400,
};
/// The `[scroll]` default `edge_px`.
const EDGE: i32 = 8;

/// The push with the default edge.
fn push(x: i32, y: i32, kb: Rect) -> Option<Point> {
    pushed_out(Point { x, y }, kb, DESK, EDGE.unsigned_abs())
}

/// A point found.
#[allow(
    clippy::unnecessary_wraps,
    reason = "It stands for what the calls return."
)]
const fn p(x: i32, y: i32) -> Option<Point> {
    Some(Point { x, y })
}

#[test]
fn history_words_above_i16_max_are_negative() {
    assert_eq!(from_history(65535, 65176), Point { x: -1, y: -360 });
    assert_eq!(
        from_history(-1, -360),
        Point { x: -1, y: -360 },
        "already signed"
    );
    assert_eq!(from_history(2559, 1439), Point { x: 2559, y: 1439 });
    assert_eq!(to_history(Point { x: -1920, y: -360 }), (63616, 65176));
    assert_eq!(to_history(Point { x: 2559, y: 0 }), (2559, 0));
}

#[test]
fn the_newest_history_point_outside_the_keyboard_wins() {
    let left_screen = (63716, 65216);
    let history = [(800, 1100), (800, 1001), left_screen, (500, 500)];
    let got = last_outside(history, KB, DESK);
    assert_eq!(got, Some(Point { x: -1820, y: -320 }));
}

#[test]
fn a_history_inside_the_keyboard_or_off_the_screen_misses() {
    assert_eq!(last_outside([], KB, DESK), None);
    assert_eq!(last_outside([(800, 1100), (400, 1000)], KB, DESK), None);
    assert_eq!(last_outside([(3000, 100), (-100, 1500)], KB, DESK), None);
}

#[test]
fn a_miss_pushes_the_entry_point_edge_px_past_the_nearest_edge() {
    assert_eq!(push(900, 1002, KB), p(900, 1000 - EDGE), "top");
    assert_eq!(push(403, 1200, KB), p(400 - EDGE, 1200), "left");
    assert_eq!(push(1998, 1200, KB), p(1999 + EDGE, 1200), "right");
    assert_eq!(push(900, 1397, KB), p(900, 1399 + EDGE), "bottom");
}

#[test]
fn a_push_off_the_screen_goes_past_the_opposite_edge() {
    let low = Rect {
        y: 1100,
        h: 340,
        ..KB
    };
    assert_eq!(
        push(900, 1438, low),
        p(900, 1100 - EDGE),
        "the screen's bottom"
    );
    let left = Rect {
        x: -1920,
        y: 0,
        w: 1000,
        h: 500,
    };
    assert_eq!(
        push(-1919, 200, left),
        p(-921 + EDGE, 200),
        "the desk's left"
    );
}

#[test]
fn a_keyboard_as_wide_as_the_screen_goes_past_its_top() {
    let wide = Rect {
        x: -1920,
        y: 1100,
        w: 4480,
        h: 340,
    };
    assert_eq!(push(-1918, 1300, wide), p(-1918, 1100 - EDGE));
}

#[test]
fn with_no_room_for_edge_px_the_point_squeezes_onto_the_screen() {
    let most = Rect {
        y: -357,
        h: 1797,
        ..DESK
    };
    assert_eq!(push(0, 0, most), p(0, -360), "three pixels left at the top");
}

#[test]
fn a_keyboard_over_the_whole_screen_leaves_no_point() {
    assert_eq!(push(0, 0, DESK), None);
    let more = Rect {
        x: -2000,
        y: -400,
        w: 5000,
        h: 2000,
    };
    assert_eq!(push(0, 0, more), None);
    let none = Rect { w: 0, ..DESK };
    assert_eq!(pushed_out(Point::default(), KB, none, 8), None, "no screen");
}

#[test]
fn an_entry_off_the_screen_still_lands_on_it() {
    assert_eq!(
        push(9000, 1002, KB),
        p(1999 + EDGE, 1002),
        "past the right edge"
    );
    let wide = Rect {
        x: -1920,
        y: 1100,
        w: 4480,
        h: 340,
    };
    assert_eq!(
        push(9000, 1300, wide),
        p(2559, 1100 - EDGE),
        "x kept on the screen"
    );
}

#[test]
fn an_edge_of_0_still_leaves_the_keyboard() {
    let got = pushed_out(Point { x: 900, y: 1002 }, KB, DESK, 0);
    assert_eq!(got, p(900, 999));
}

#[test]
fn the_target_takes_the_history_before_the_push() {
    let entry = Point { x: 900, y: 1002 };
    let edge = EDGE.unsigned_abs();
    assert_eq!(target(&[(500, 500)], entry, KB, DESK, edge), p(500, 500));
    assert_eq!(target(&[], entry, KB, DESK, edge), p(900, 1000 - EDGE));
    let inside = [(800, 1100)];
    assert_eq!(target(&inside, entry, KB, DESK, edge), p(900, 1000 - EDGE));
}
