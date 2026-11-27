//! The fake freeze, held to the port: screens of different scalings painted to the pixel, the windows the platform's
//! rule offers front first, the pointer, the cap before any paint and a refusal; every cap asked is kept.

use super::*;
use kx_module_api::ports::SHOT_CAP_MB;

/// A screen at 100 %.
const MAIN: Rect = Rect {
    x: 0,
    y: 0,
    w: 8,
    h: 5,
};
/// A screen to its left at 150 %, as a laptop's.
const LEFT: Rect = Rect {
    x: -6,
    y: -2,
    w: 6,
    h: 4,
};
/// `MAIN`'s DPI.
const MAIN_DPI: u32 = 96;
/// `LEFT`'s DPI.
const LEFT_DPI: u32 = 144;
/// Where the pointer is.
const POINTER: Point = Point { x: 3, y: 2 };
/// A refusal's code.
const CODE: i32 = 5;
/// The width of the screens that test the cap.
const WIDE: i32 = 1024;

/// The smallest cap `shot_cap_mb` allows.
fn cap() -> ShotCap {
    ShotCap::new(*SHOT_CAP_MB.start()).unwrap()
}

/// Both screens, the pointer on the main one.
fn desk() -> FakeScreenFreeze {
    FakeScreenFreeze::new(vec![(MAIN, MAIN_DPI), (LEFT, LEFT_DPI)], POINTER)
}

/// A screen `WIDE` pixels wide with `rows` rows.
fn tall(rows: u64) -> (Rect, u32) {
    let h = i32::try_from(rows).unwrap();
    let rect = Rect {
        x: 0,
        y: 0,
        w: WIDE,
        h,
    };
    (rect, MAIN_DPI)
}

/// The rows of a `WIDE` screen that fill `cap` exactly.
fn rows_in_cap() -> u64 {
    let row = u64::try_from(WIDE).unwrap() * u64::try_from(BGRX_BYTES).unwrap();
    cap().bytes() / row
}

#[test]
fn every_screen_is_frozen_at_its_place_and_scaling_with_the_pointer() {
    let fake = desk();
    let frozen = fake.freeze(cap()).unwrap();
    let facts: Vec<_> = frozen.screens.iter().map(|s| (s.rect(), s.dpi())).collect();
    assert_eq!(facts, [(MAIN, MAIN_DPI), (LEFT, LEFT_DPI)]);
    assert_eq!(frozen.pointer, POINTER);
    assert!(frozen.windows.is_empty());
    let moved = Point { x: -3, y: 0 };
    fake.set_screens(vec![(LEFT, LEFT_DPI)]);
    fake.set_pointer(moved);
    let frozen = fake.freeze(cap()).unwrap();
    assert_eq!((frozen.screens.len(), frozen.pointer), (1, moved));
}

#[test]
fn each_pixel_tells_its_desktop_point_on_every_screen() {
    let frozen = desk().freeze(cap()).unwrap();
    for screen in &frozen.screens {
        let rect = screen.rect();
        let (pixels, _) = screen.shot().bgrx().expose().as_chunks::<BGRX_BYTES>();
        for (i, pixel) in pixels.iter().enumerate() {
            let i = i32::try_from(i).unwrap();
            let at = Point {
                x: rect.x + i % rect.w,
                y: rect.y + i / rect.w,
            };
            assert_eq!(*pixel, pixel_at(at), "{at:?}");
        }
    }
    let near = [(0, 0), (-1, 0), (0, -1), (1, 1)].map(|(x, y)| pixel_at(Point { x, y }));
    let apart: std::collections::HashSet<_> = near.iter().collect();
    assert_eq!(apart.len(), near.len(), "points apart paint apart");
}

#[test]
fn only_shown_windows_with_pixels_are_offered_front_first_by_the_real_rule() {
    let fake = desk();
    let at = |x| Rect {
        x,
        y: 0,
        w: 2,
        h: 2,
    };
    let empty = Rect { w: 0, ..at(2) };
    let mut windows = vec![
        (at(0), Seen::Shown),
        (empty, Seen::Shown),
        (at(1), Seen::Shown),
    ];
    let left_out = [
        Seen::Hidden,
        Seen::Ours,
        Seen::Desktop,
        Seen::Minimised,
        Seen::Tool,
        Seen::Through,
        Seen::Cloaked,
    ];
    windows.extend((3..).zip(left_out).map(|(x, seen)| (at(x), seen)));
    fake.set_windows(windows);
    assert_eq!(fake.freeze(cap()).unwrap().windows, [at(0), at(1)]);
}

#[test]
fn screens_past_the_cap_together_freeze_nothing() {
    let rows = rows_in_cap();
    let fake = FakeScreenFreeze::new(vec![tall(rows + 1)], POINTER);
    assert_eq!(fake.freeze(cap()), Err(FreezeError::TooBig));
    let half = rows / 2 + 1;
    fake.set_screens(vec![tall(half), tall(half)]);
    assert_eq!(fake.freeze(cap()), Err(FreezeError::TooBig), "together");
    assert_eq!(fake.caps(), [cap(); 2], "refused asks are kept");
}

#[test]
fn a_refusal_comes_as_set_until_cleared() {
    let fake = desk();
    fake.set_error(Some(FreezeError::Refused(CODE)));
    assert_eq!(fake.freeze(cap()), Err(FreezeError::Refused(CODE)));
    fake.set_error(None);
    assert!(fake.freeze(cap()).is_ok());
    assert_eq!(fake.caps().len(), 2);
}

#[test]
fn a_screen_with_no_pixels_is_left_out() {
    let empty = Rect { w: 0, ..MAIN };
    let fake = FakeScreenFreeze::new(vec![(empty, MAIN_DPI), (LEFT, LEFT_DPI)], POINTER);
    let frozen = fake.freeze(cap()).unwrap();
    assert_eq!(frozen.screens.len(), 1);
    assert_eq!(painted(empty, MAIN_DPI), None);
}

#[test]
fn clones_share_one_desk() {
    let fake = desk();
    let port: Box<dyn ScreenFreeze> = Box::new(fake.clone());
    fake.set_error(Some(FreezeError::Refused(CODE)));
    assert_eq!(port.freeze(cap()), Err(FreezeError::Refused(CODE)));
    assert_eq!(fake.caps().len(), 1);
}
