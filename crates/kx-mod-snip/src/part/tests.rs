//! The picking rules: two corners in any order, the same spot twice, the front window under a point, the screen holding
//! a point or most of a part, and edge 2, a part across two screens of different scaling kept to the pixel.

#![allow(
    clippy::unwrap_used,
    reason = "these helpers run only in tests, where a failed setup should fail the test"
)]

use super::*;
use kx_module_api::ports::BGRX_BYTES;
use kx_platform_fake::{painted, pixel_at};

mod props;

/// The main screen, at 100 %.
const MAIN: Rect = Rect {
    x: 0,
    y: 0,
    w: 40,
    h: 30,
};
/// A smaller screen left of the main one and higher, at negative coordinates and 150 %.
const SIDE: Rect = Rect {
    x: -24,
    y: -10,
    w: 24,
    h: 20,
};
/// A window behind `FRONT`, on the main screen.
const BACK: Rect = Rect {
    x: 2,
    y: 2,
    w: 20,
    h: 16,
};
/// The front window, over part of `BACK`.
const FRONT: Rect = Rect {
    x: 10,
    y: 8,
    w: 20,
    h: 12,
};

/// Both screens as frozen, with the windows front first.
fn frozen() -> Frozen {
    Frozen {
        screens: vec![painted(MAIN, 96).unwrap(), painted(SIDE, 144).unwrap()],
        windows: vec![FRONT, BACK],
        pointer: at(1, 1),
    }
}

const fn at(x: i32, y: i32) -> Point {
    Point { x, y }
}

/// `rect` taken as a part.
fn taken(rect: Rect) -> Option<Pick> {
    Part::new(rect).map(Pick::Part)
}

/// A click at `to` while `way` is picked, with `corner` waiting.
fn click(way: BarKey, corner: Option<Point>, to: Point) -> Option<Pick> {
    read(way, corner, to, &frozen())
}

/// The pixel at `to` holds, cut from `screens`: its own on a screen, black between them.
fn pixel(to: Point, screens: &[FrozenScreen]) -> [u8; BGRX_BYTES] {
    let on = screens.iter().any(|s| holds(s.rect(), to));
    if on { pixel_at(to) } else { [0; BGRX_BYTES] }
}

/// `rect` holds the pixel at `to`; small test numbers never overflow.
fn holds(rect: Rect, to: Point) -> bool {
    (rect.x..rect.x + rect.w).contains(&to.x) && (rect.y..rect.y + rect.h).contains(&to.y)
}

/// `part` cut from `screens` holds every pixel as frozen there, black where no screen is.
fn exact(part: Part, screens: &[FrozenScreen]) -> bool {
    let Some(shot) = part.cut(screens) else {
        return false;
    };
    let r = part.rect();
    let points = (r.y..r.y + r.h).flat_map(|y| (r.x..r.x + r.w).map(move |x| at(x, y)));
    let want: Vec<u8> = points.flat_map(|p| pixel(p, screens)).collect();
    shot.bgrx().expose().as_ref() == want.as_slice()
}

#[test]
fn a_region_waits_for_its_first_corner() {
    for way in [BarKey::Region, BarKey::VideoRegion] {
        let corner = Some(Pick::Corner(at(3, 4)));
        assert_eq!(click(way, None, at(3, 4)), corner, "{way:?}");
    }
}

#[test]
fn two_corners_in_any_order_take_the_same_part() {
    let want = taken(Rect {
        x: 3,
        y: 4,
        w: 10,
        h: 6,
    });
    let (a, b, c, d) = (at(3, 4), at(12, 9), at(12, 4), at(3, 9));
    for (one, two) in [(a, b), (b, a), (c, d), (d, c)] {
        assert_eq!(
            click(BarKey::Region, Some(one), two),
            want,
            "{one:?}, {two:?}"
        );
    }
}

#[test]
fn both_corner_pixels_are_inside_the_part() {
    let want = taken(Rect {
        x: 3,
        y: 4,
        w: 2,
        h: 1,
    });
    assert_eq!(click(BarKey::Region, Some(at(4, 4)), at(3, 4)), want);
}

#[test]
fn a_video_region_stops_at_its_first_corners_screen_while_a_still_crosses() {
    let (one, two) = (at(3, 4), at(-10, -5));
    let still = Rect {
        x: -10,
        y: -5,
        w: 14,
        h: 10,
    };
    assert_eq!(click(BarKey::Region, Some(one), two), taken(still));
    let video = Rect {
        x: 0,
        y: 0,
        w: 4,
        h: 4,
    };
    assert_eq!(
        click(BarKey::VideoRegion, Some(one), two),
        taken(video),
        "even, on MAIN"
    );
}

#[test]
fn a_video_region_is_even_and_at_least_two_by_two_on_its_screen() {
    let video = |one, two| click(BarKey::VideoRegion, Some(one), two);
    let part = |x, y, w, h| taken(Rect { x, y, w, h });
    assert_eq!(
        video(at(3, 4), at(5, 7)),
        part(3, 4, 2, 4),
        "odd sides lose a pixel"
    );
    assert_eq!(
        video(at(3, 4), at(4, 4)),
        part(3, 4, 2, 2),
        "a thin one grows"
    );
    let (right, bottom) = (MAIN.w - 1, MAIN.h - 1);
    let edge = video(at(right, bottom), at(right, bottom + 5));
    assert_eq!(
        edge,
        part(right - 1, bottom - 1, 2, 2),
        "it grows away from the edge"
    );
}

#[test]
fn edge_2_the_same_spot_twice_drops_the_corner() {
    for way in [BarKey::Region, BarKey::VideoRegion] {
        let twice = click(way, Some(at(3, 4)), at(3, 4));
        assert_eq!(twice, Some(Pick::Unset), "{way:?}");
    }
}

#[test]
fn the_front_window_under_the_click_is_taken() {
    // A corner left by another way is not read.
    let on = |to| click(BarKey::Window, Some(at(1, 1)), to);
    assert_eq!(on(at(15, 10)), taken(FRONT), "both hold it; the front wins");
    assert_eq!(on(at(4, 4)), taken(BACK), "only the back one holds it");
    assert_eq!(on(at(10, 8)), taken(FRONT), "a frame holds its first pixel");
    assert_eq!(on(at(30, 8)), None, "and not the one past its last");
    assert_eq!(on(at(35, 25)), None, "a point on no window");
}

#[test]
fn a_window_takes_only_what_showed_on_the_screens() {
    let off = Rect {
        x: 30,
        y: 20,
        w: 20,
        h: 20,
    };
    let across = Rect {
        x: -10,
        y: 0,
        w: 20,
        h: 10,
    };
    let frozen = Frozen {
        windows: vec![off, across],
        ..frozen()
    };
    let shown = Rect {
        x: 30,
        y: 20,
        w: 10,
        h: 10,
    };
    assert_eq!(
        window(at(35, 25), &frozen),
        Part::new(shown),
        "past the edge"
    );
    assert_eq!(window(at(-5, 5), &frozen), Part::new(across), "on both");
    let hidden = Frozen {
        windows: vec![Rect { x: 50, ..off }],
        ..frozen
    };
    assert_eq!(window(at(55, 25), &hidden), None, "on no screen");
}

#[test]
fn full_screen_takes_the_screen_holding_the_click() {
    for way in [BarKey::Full, BarKey::VideoFull] {
        assert_eq!(click(way, None, at(5, 5)), taken(MAIN), "{way:?}");
        assert_eq!(click(way, None, at(-5, -5)), taken(SIDE), "{way:?}");
        assert_eq!(click(way, None, at(-5, 15)), None, "{way:?} between");
    }
}

#[test]
fn a_point_between_screens_is_on_no_screen() {
    let screens = frozen().screens;
    let on = |to| screen(to, &screens).map(FrozenScreen::rect);
    assert_eq!(
        on(at(-5, 15)),
        None,
        "under the side screen, left of the main"
    );
    assert_eq!(on(at(40, 0)), None, "a screen ends before its right edge");
    assert_eq!(on(at(39, 29)), Some(MAIN));
    assert_eq!(on(at(-24, -10)), Some(SIDE));
}

#[test]
fn settings_and_close_take_nothing() {
    for way in [BarKey::Settings, BarKey::Close] {
        assert_eq!(click(way, None, at(5, 5)), None, "{way:?}");
    }
}

#[test]
fn a_part_across_two_screens_belongs_to_the_one_holding_most() {
    let screens = frozen().screens;
    let home_of = |x, w| home(Part::new(Rect { x, y: 0, w, h: 6 }).unwrap(), &screens);
    let side = Some(Home {
        rect: SIDE,
        dpi: 144,
    });
    let main = Some(Home {
        rect: MAIN,
        dpi: 96,
    });
    assert_eq!(
        home_of(-10, 16),
        side,
        "10 columns on the side, 6 on the main"
    );
    assert_eq!(home_of(-3, 20), main, "3 on the side, 17 on the main");
    assert_eq!(home_of(-5, 10), main, "a tie goes to the first screen");
    assert_eq!(home_of(100, 4), None, "on no screen");
}

#[test]
fn edge_2_a_part_across_screens_of_different_scaling_keeps_exact_pixels() {
    let frozen = frozen();
    let read = read(BarKey::Region, Some(at(-20, -8)), at(20, 20), &frozen);
    let Some(Pick::Part(part)) = read else {
        panic!("no part: {read:?}");
    };
    let shot = part.cut(&frozen.screens).unwrap();
    assert_eq!((shot.width(), shot.height()), (41, 29), "physical pixels");
    assert!(exact(part, &frozen.screens));
}

#[test]
fn rectangles_meet_only_when_they_share_a_pixel() {
    assert!(meets(MAIN, FRONT));
    let beside = Rect { x: MAIN.w, ..MAIN };
    assert!(!meets(MAIN, beside), "touching edges share no pixel");
}
