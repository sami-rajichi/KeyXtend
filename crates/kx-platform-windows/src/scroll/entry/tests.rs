//! The point before the keyboard, on two screens with dead zones between them: the main one, and one left of it and
//! higher.

use super::*;

/// The main screen's work area at 125 %, in physical pixels: 2560 × 1440 with a 40-pixel taskbar at the bottom.
const MAIN: Rect = Rect {
    x: 0,
    y: 0,
    w: 2560,
    h: 1400,
};
/// The left screen's work area at 100 %, 360 pixels higher: y runs from -360 to 719.
const LEFT: Rect = Rect {
    x: -1920,
    y: -360,
    w: 1920,
    h: 1080,
};
/// The keyboard near the bottom of the main screen: x 400 to 1999, y 1000 to 1359.
const KB: Rect = Rect {
    x: 400,
    y: 1000,
    w: 1600,
    h: 360,
};
/// The `[scroll]` default `edge_px`.
const EDGE: i32 = 8;

/// Both screens.
fn screens() -> [Screen; 2] {
    let main = Screen {
        rect: MAIN,
        dpi: 120,
        primary: true,
    };
    let left = Screen {
        rect: LEFT,
        dpi: 96,
        primary: false,
    };
    [main, left]
}

/// The push on both screens with the default edge.
fn push(x: i32, y: i32, kb: Rect) -> Option<Point> {
    pushed_out(Point { x, y }, kb, &screens(), EDGE.unsigned_abs())
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
    let got = last_outside(history, KB, &screens());
    assert_eq!(got, Some(Point { x: -1820, y: -320 }));
}

#[test]
fn a_history_inside_the_keyboard_or_off_every_work_area_misses() {
    let all = screens();
    assert_eq!(last_outside([], KB, &all), None);
    assert_eq!(last_outside([(800, 1100), (400, 1000)], KB, &all), None);
    let off = [(3000, 100), (-100, 1000), (100, -100), (900, 1420)];
    assert_eq!(
        last_outside(off, KB, &all),
        None,
        "past, dead zones, taskbar"
    );
    let after = [(900, 1420), (900, 500)];
    assert_eq!(
        last_outside(after, KB, &all),
        p(900, 500),
        "older than the taskbar"
    );
}

#[test]
fn a_miss_pushes_the_entry_point_edge_px_past_the_nearest_edge() {
    assert_eq!(push(900, 1002, KB), p(900, 1000 - EDGE), "top");
    assert_eq!(push(403, 1200, KB), p(400 - EDGE, 1200), "left");
    assert_eq!(push(1998, 1200, KB), p(1999 + EDGE, 1200), "right");
    assert_eq!(push(900, 1357, KB), p(900, 1359 + EDGE), "bottom");
    assert_eq!(
        push(9000, 1002, KB),
        p(1999 + EDGE, 1002),
        "an entry past the right"
    );
}

#[test]
fn a_push_off_every_work_area_takes_the_next_nearest_edge() {
    let low = Rect {
        y: 1100,
        h: 300,
        ..KB
    };
    assert_eq!(
        push(900, 1398, low),
        p(900, 1100 - EDGE),
        "not onto the taskbar"
    );
}

#[test]
fn a_push_into_a_dead_zone_between_screens_takes_the_next_nearest_edge() {
    let kb = Rect {
        x: 0,
        y: 1000,
        w: 1600,
        h: 400,
    };
    assert_eq!(
        push(5, 1300, kb),
        p(5, 1000 - EDGE),
        "(-8, 1300) is on no screen"
    );
}

#[test]
fn a_miss_with_no_screen_past_any_edge_finds_none() {
    assert_eq!(
        push(5, 1000, MAIN),
        None,
        "dead zones, the taskbar and past the right"
    );
    assert_eq!(
        pushed_out(Point { x: 900, y: 1002 }, KB, &[], 8),
        None,
        "no screen"
    );
}

#[test]
fn an_edge_of_0_still_leaves_the_keyboard() {
    let got = pushed_out(Point { x: 900, y: 1002 }, KB, &screens(), 0);
    assert_eq!(got, p(900, 999));
}

#[test]
fn boxes_at_the_ends_of_the_range_never_panic() {
    let far = Rect {
        x: i32::MAX - 4,
        y: i32::MAX - 4,
        w: 100,
        h: 100,
    };
    let near = Rect {
        x: i32::MIN,
        y: i32::MIN,
        w: 3,
        h: 3,
    };
    let rim = Rect {
        x: i32::MAX,
        y: i32::MAX,
        w: 5,
        h: 5,
    };
    let at = |x, y| Point { x, y };
    let screens = [far, near, rim].map(|rect| Screen {
        rect,
        ..screens()[0]
    });
    let (top, bottom) = (at(i32::MAX, i32::MAX), at(i32::MIN, i32::MIN));
    for (entry, kb) in [(top, far), (bottom, near), (top, rim)] {
        for edge in [0, EDGE.unsigned_abs(), u32::MAX] {
            let got = pushed_out(entry, kb, &screens, edge);
            assert!(got.is_none_or(|p| !contains(kb, p)), "{edge}");
        }
    }
    let got = pushed_out(at(i32::MAX - 2, 0), near, &screens, EDGE.unsigned_abs());
    assert!(got.is_none_or(|p| screens.iter().any(|s| contains(s.rect, p))));
}

#[test]
fn the_target_takes_the_history_before_the_push() {
    let (entry, all) = (Point { x: 900, y: 1002 }, screens());
    let edge = EDGE.unsigned_abs();
    assert_eq!(target(&[(500, 500)], entry, KB, &all, edge), p(500, 500));
    assert_eq!(target(&[], entry, KB, &all, edge), p(900, 1000 - EDGE));
    let inside = [(800, 1100)];
    assert_eq!(target(&inside, entry, KB, &all, edge), p(900, 1000 - EDGE));
}

#[test]
fn the_guard_keeps_the_newest_history_point_outside_else_the_push() {
    let all = screens();
    let entry = Point { x: 800, y: 1010 };
    let kept = Kept::new(&[(800, 1010), (800, 990), (900, 500)], entry, KB, &all);
    assert_eq!(kept.outside, p(800, 990));
    assert_eq!(kept.target(&all, EDGE.unsigned_abs()), p(800, 990));
    let missed = Kept::new(&[(800, 1010)], entry, KB, &all);
    assert_eq!(missed.outside, None);
    assert_eq!(
        missed.target(&all, EDGE.unsigned_abs()),
        push(800, 1010, KB)
    );
    let whole = target(&[(800, 1010)], entry, KB, &all, EDGE.unsigned_abs());
    assert_eq!(
        missed.target(&all, EDGE.unsigned_abs()),
        whole,
        "as target does"
    );
}

#[test]
fn a_kept_point_on_a_screen_gone_since_falls_back_to_the_push() {
    let [main, _] = screens();
    let entry = Point { x: 800, y: 1010 };
    let kept = Kept::new(&[(800, 1010), (63716, 65216)], entry, KB, &screens());
    assert_eq!(kept.outside, p(-1820, -320), "on the left screen");
    let push = pushed_out(entry, KB, &[main], EDGE.unsigned_abs());
    assert_eq!(kept.target(&[main], EDGE.unsigned_abs()), push);
}

/// A point on our banner, just above the keyboard.
fn on_banner(p: Point) -> bool {
    p == Point { x: 700, y: 990 }
}

#[test]
fn an_entry_passes_over_our_windows_to_the_app_before_them() {
    let (all, entry) = (screens(), Point { x: 800, y: 1010 });
    let last = Kept::new(&[(800, 1010), (900, 500)], entry, KB, &all);
    let history = [(800, 1010), (700, 990), (600, 400)];
    let via = Kept::past(&history, entry, KB, &all, on_banner);
    assert_eq!(via.outside, p(600, 400), "the app before the banner");
    assert_eq!(via.after(Some(last)), via);
}

#[test]
fn an_entry_from_our_windows_alone_keeps_the_last_and_one_with_nothing_outside_pushes_out() {
    let (all, entry) = (screens(), Point { x: 800, y: 1010 });
    let last = Kept::new(&[(800, 1010), (900, 500)], entry, KB, &all);
    let banner = Kept::past(&[(800, 1010), (700, 990)], entry, KB, &all, on_banner);
    assert_eq!(banner.after(Some(last)), last, "back from our banner");
    assert_eq!(banner.after(None).outside, None, "a first entry pushes out");
    let missed = Kept::past(&[(800, 1010)], entry, KB, &all, |_| true);
    assert_eq!(missed.after(Some(last)), missed, "nothing outside");
}

#[test]
fn a_kept_entry_hides_every_point_in_debug() {
    let kept = Kept::new(&[(812, 913)], Point { x: 814, y: 1015 }, KB, &screens());
    let shown = format!("{kept:?}");
    for number in ["812", "913", "814", "1015", "1600"] {
        assert!(!shown.contains(number), "{shown}");
    }
    assert!(shown.contains("Kept"), "{shown}");
}
