use super::*;
use kx_module_api::ports::BASE_DPI;
use proptest::prelude::*;

/// The size-1 board, about 671 by 196 logical pixels.
const PLATE: (f64, f64) = (670.78, 196.0);
/// A 150 % screen.
const HIGH_DPI: u32 = 144;
/// How far from the screens the tests try places, in pixels.
const FAR: i32 = 8_000;

fn rect(x: i32, y: i32, w: i32, h: i32) -> Rect {
    Rect { x, y, w, h }
}

fn at(x: i32, y: i32) -> Point {
    Point { x, y }
}

/// A 100 % primary screen with a taskbar, and a 150 % screen to its right.
fn two() -> [Screen; 2] {
    [
        Screen {
            rect: rect(0, 0, 1920, 1040),
            dpi: BASE_DPI,
            primary: true,
        },
        Screen {
            rect: rect(1920, 0, 2560, 1400),
            dpi: HIGH_DPI,
            primary: false,
        },
    ]
}

/// The 150 % screen listed first and placed left of the 100 % primary, above its top edge.
fn left_first() -> [Screen; 2] {
    let [primary, mut high] = two();
    high.rect = rect(-2560, -200, 2560, 1400);
    [high, primary]
}

/// The window's physical size on `s`.
fn size_on(s: &Screen) -> (i32, i32) {
    (physical(PLATE.0, s.dpi), physical(PLATE.1, s.dpi))
}

/// True when a `size` window at `p` lies wholly in `work`.
fn within(p: Point, size: (i32, i32), work: Rect) -> bool {
    p.x >= work.x
        && p.y >= work.y
        && p.x + size.0 <= work.x + work.w
        && p.y + size.1 <= work.y + work.h
}

#[test]
fn a_box_sits_centred_at_the_bottom_of_the_work_area() {
    assert_eq!(
        bottom_centre(rect(0, 0, 1920, 1040), (900, 70)),
        at(510, 970)
    );
    let wide = bottom_centre(rect(100, 0, 400, 800), (600, 50));
    assert_eq!(
        wide.x, 100,
        "a box wider than the screen starts at its left edge"
    );
    let tall = bottom_centre(rect(0, 40, 800, 100), (200, 300));
    assert_eq!(tall.y, 40, "a box taller than the screen starts at its top");
}

#[test]
fn the_keyboard_starts_at_the_bottom_centre_of_the_primary_screen() {
    let home_spot = at(624, 844);
    assert_eq!(start(None, PLATE, &two()), Some(home_spot));
    let [high, primary] = left_first();
    assert_eq!(
        home(PLATE, &[high, primary]),
        Some(home_spot),
        "the primary, wherever it is listed"
    );
    let [a, b] = two();
    let none = [
        Screen {
            primary: false,
            ..b
        },
        Screen {
            primary: false,
            ..a
        },
    ];
    assert_eq!(
        home(PLATE, &none),
        home(PLATE, &[b]),
        "no primary: the first"
    );
    assert_eq!(start(None, PLATE, &[]), None);
}

#[test]
fn the_primary_home_uses_the_primary_dpi() {
    let high = Screen {
        rect: rect(0, 0, 2560, 1400),
        dpi: HIGH_DPI,
        primary: true,
    };
    let (w, h) = size_on(&high);
    assert_eq!(home(PLATE, &[high]), Some(at((2560 - w) / 2, 1400 - h)));
}

#[test]
fn a_saved_place_on_a_screen_stays() {
    assert_eq!(start(Some(at(100, 200)), PLATE, &two()), Some(at(100, 200)));
}

#[test]
fn an_off_screen_place_returns_to_the_bottom_centre() {
    let screens = two();
    let home_spot = home(PLATE, &screens);
    for far in [at(FAR, FAR), at(-FAR, 0), at(0, -FAR), at(1920, 1400)] {
        assert_eq!(start(Some(far), PLATE, &screens), home_spot, "{far:?}");
    }
}

#[test]
fn a_place_on_a_second_screen_with_another_dpi_stays_there() {
    let screens = two();
    assert_eq!(
        start(Some(at(2400, 900)), PLATE, &screens),
        Some(at(2400, 900))
    );
    let (w, _) = size_on(&screens[1]);
    let near_edge = fit(at(3600, 900), PLATE, &screens);
    assert_eq!(
        near_edge,
        Some(at(1920 + 2560 - w, 900)),
        "the 150 % width keeps it on"
    );
}

#[test]
fn a_place_past_the_edge_is_stopped_at_the_edge() {
    assert_eq!(fit(at(-300, 1000), PLATE, &two()), Some(at(0, 844)));
    assert_eq!(fit(at(300, -50), PLATE, &two()), Some(at(300, 0)));
    let r = fit(at(1320, 500), PLATE, &two());
    assert_eq!(r, Some(at(1920 - 671, 500)), "mostly on the primary");
}

#[test]
fn a_window_across_two_screens_goes_to_the_one_it_covers_most() {
    let screens = two();
    let (w, _) = size_on(&screens[1]);
    let r = fit(at(1920 - 100, 500), PLATE, &screens).unwrap();
    assert_eq!(r, at(1920, 500), "most of it is on the second screen");
    assert!(within(r, (w, physical(PLATE.1, HIGH_DPI)), screens[1].rect));
}

#[test]
fn the_windows_screen_is_the_one_it_covers_most_and_none_off_them() {
    let s = two();
    assert_eq!(screen_of(at(2000, 100), PLATE, &s), Some(&s[1]));
    assert_eq!(
        screen_of(at(1500, 100), PLATE, &s),
        Some(&s[0]),
        "most on the first"
    );
    assert_eq!(screen_of(at(FAR, FAR), PLATE, &s), None);
}

#[test]
fn the_room_is_the_largest_size_whose_plate_fits_the_work_area() {
    let [primary, high] = two();
    let wide = room(PLATE, &primary).unwrap();
    assert!((wide - 1920.0 / PLATE.0).abs() < 1e-9, "the width binds");
    let fits = room(PLATE, &high).unwrap();
    assert!(physical(PLATE.0 * fits, HIGH_DPI) <= high.rect.w);
    let flat = Screen {
        rect: rect(0, 0, 1920, 0),
        ..primary
    };
    assert_eq!(room(PLATE, &flat), None, "an empty work area");
    assert_eq!(room((0.0, 0.0), &primary), None, "an empty plate");
}

#[test]
fn a_window_lands_on_the_screen_it_covers_most_else_the_primary() {
    let s = two();
    assert_eq!(landing(Some(at(2000, 100)), PLATE, &s), Some(&s[1]));
    assert_eq!(landing(None, PLATE, &s), Some(&s[0]), "home");
    assert_eq!(landing(Some(at(FAR, FAR)), PLATE, &s), Some(&s[0]));
    assert_eq!(landing(None, PLATE, &[]), None);
}

#[test]
fn odd_screens_never_panic() {
    let odd = Screen {
        rect: rect(i32::MAX - 5, i32::MIN, -10, i32::MAX),
        dpi: 0,
        primary: true,
    };
    let _ = start(Some(at(i32::MAX, i32::MIN)), PLATE, &[odd]);
    let _ = start(None, (f64::NAN, f64::INFINITY), &[odd]);
}

proptest! {
    #[test]
    fn a_fitted_place_lies_wholly_on_one_screen(x in -FAR..FAR, y in -FAR..FAR, left in any::<bool>()) {
        let screens = if left { left_first() } else { two() };
        let p = fit(at(x, y), PLATE, &screens).unwrap();
        let on = screens.iter().any(|s| within(p, size_on(s), s.rect));
        prop_assert!(on, "{p:?}");
    }

    #[test]
    fn a_place_wholly_on_a_screen_never_moves(pick in any::<bool>(), fx in 0.0..1.0, fy in 0.0..1.0) {
        let s = two()[usize::from(pick)];
        let (w, h) = size_on(&s);
        let room = |span: i32, len: i32, f: f64| {
            let free = f64::from(span - len);
            #[allow(clippy::cast_possible_truncation, reason = "the value is between 0 and the free room")]
            let off = (free * f).floor() as i32;
            off
        };
        let p = at(s.rect.x + room(s.rect.w, w, fx), s.rect.y + room(s.rect.h, h, fy));
        prop_assert_eq!(fit(p, PLATE, &two()), Some(p));
    }

    #[test]
    fn a_place_off_every_screen_goes_home(x in FAR..2 * FAR, y in -FAR..FAR) {
        let screens = two();
        prop_assert_eq!(fit(at(x, y), PLATE, &screens), home(PLATE, &screens));
    }
}
