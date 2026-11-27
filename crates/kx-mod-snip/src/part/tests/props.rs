//! Properties over any clicks on and around both screens: two corners in any order give one part holding both, the
//! front window holding a point wins, the home screen holds the most of a part, and every cut keeps exact pixels.

use super::*;
use proptest::collection::vec;
use proptest::prelude::*;

/// Random cases per property.
const CASES: u32 = 256;

/// A point on or around both screens.
fn point() -> impl Strategy<Value = Point> {
    (-30..=45_i32, -15..=35_i32).prop_map(|(x, y)| at(x, y))
}

/// A window frame inside the main screen, so nothing of it is cut away.
fn frame() -> impl Strategy<Value = Rect> {
    (0..39_i32, 0..29_i32)
        .prop_flat_map(|(x, y)| (Just(x), Just(y), 1..=MAIN.w - x, 1..=MAIN.h - y))
        .prop_map(|(x, y, w, h)| Rect { x, y, w, h })
}

/// The pixels `a` and `b` share.
fn shared(a: Rect, b: Rect) -> i32 {
    let w = (a.x + a.w).min(b.x + b.w) - a.x.max(b.x);
    let h = (a.y + a.h).min(b.y + b.h) - a.y.max(b.y);
    w.max(0) * h.max(0)
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(CASES))]

    #[test]
    fn two_corners_in_any_order_take_one_part_holding_both(one in point(), two in point()) {
        let frozen = frozen();
        let picked = read(BarKey::Region, Some(one), two, &frozen);
        prop_assert_eq!(picked, read(BarKey::Region, Some(two), one, &frozen));
        if one == two {
            prop_assert_eq!(picked, Some(Pick::Unset));
        } else {
            let Some(Pick::Part(part)) = picked else {
                return Err(TestCaseError::fail(format!("no part: {picked:?}")));
            };
            let r = part.rect();
            prop_assert_eq!((r.x, r.y), (one.x.min(two.x), one.y.min(two.y)));
            prop_assert_eq!((r.x + r.w - 1, r.y + r.h - 1), (one.x.max(two.x), one.y.max(two.y)));
        }
    }

    #[test]
    fn a_video_region_lies_on_its_first_corners_screen_even_and_at_least_two_by_two(
        one in point(),
        two in point(),
    ) {
        let frozen = frozen();
        prop_assume!(one != two);
        let picked = read(BarKey::VideoRegion, Some(one), two, &frozen);
        let Some(on) = screen(one, &frozen.screens).map(FrozenScreen::rect) else {
            prop_assert_eq!(picked, None, "a corner on no screen takes nothing");
            return Ok(());
        };
        let Some(Pick::Part(part)) = picked else {
            return Err(TestCaseError::fail(format!("no part: {picked:?}")));
        };
        let r = part.rect();
        prop_assert!(r.w >= 2 && r.h >= 2 && r.w % 2 == 0 && r.h % 2 == 0, "{r:?}");
        prop_assert!(on.x <= r.x && r.x + r.w <= on.x + on.w, "{r:?} on {on:?}");
        prop_assert!(on.y <= r.y && r.y + r.h <= on.y + on.h, "{r:?} on {on:?}");
    }

    #[test]
    fn the_front_window_holding_the_point_wins(windows in vec(frame(), 0..6), to in point()) {
        let frozen = Frozen { windows: windows.clone(), ..frozen() };
        let held = |w: &Rect| holds(*w, to);
        match window(to, &frozen) {
            Some(part) => {
                let first = windows.iter().position(|w| *w == part.rect());
                prop_assert!(first.is_some_and(|i| held(&windows[i]) && !windows[..i].iter().any(held)));
            }
            None => prop_assert!(!windows.iter().any(held)),
        }
    }

    #[test]
    fn the_home_screen_holds_the_most_of_the_part(one in point(), two in point()) {
        let (part, frozen) = (Part::spanning(one, two).unwrap(), frozen());
        let shares: Vec<i32> = frozen.screens.iter().map(|s| shared(part.rect(), s.rect())).collect();
        match home(part, &frozen.screens) {
            Some(home) => {
                let i = frozen.screens.iter().position(|s| s.rect() == home.rect).unwrap();
                prop_assert_eq!(home.dpi, frozen.screens[i].dpi());
                prop_assert!(shares[..i].iter().all(|s| *s < shares[i]), "{shares:?}");
                prop_assert!(shares[i] > 0 && shares.iter().all(|s| *s <= shares[i]), "{shares:?}");
            }
            None => prop_assert!(shares.iter().all(|s| *s == 0), "{shares:?}"),
        }
    }

    #[test]
    fn edge_2_any_part_keeps_exact_pixels(one in point(), two in point()) {
        prop_assume!(one != two);
        let frozen = frozen();
        let picked = read(BarKey::Region, Some(one), two, &frozen);
        let Some(Pick::Part(part)) = picked else {
            return Err(TestCaseError::fail(format!("no part: {picked:?}")));
        };
        let touched = frozen.screens.iter().any(|s| shared(part.rect(), s.rect()) > 0);
        // A cut never holds more pixels than the freeze, as `Part::cut` says.
        let total: i32 = frozen.screens.iter().map(|s| s.rect().w * s.rect().h).sum();
        let within = part.rect().w * part.rect().h <= total;
        prop_assert_eq!(exact(part, &frozen.screens), touched && within);
    }
}
