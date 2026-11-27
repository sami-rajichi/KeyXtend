//! The crop maths from the spike's `capture.rs`: a part between two corners in either order, and a part cut from the
//! frozen screens exact to the pixel across scalings (edge 2), black off every screen, never bigger than its freeze.

use crate::ports::snip::{BGRX_BYTES, FrozenScreen, Part, Shot};
use crate::ports::{Point, Rect};

/// A screen at 100 %.
const MAIN: Rect = Rect {
    x: 0,
    y: 0,
    w: 8,
    h: 5,
};
/// A screen to its left at 150 %, as a laptop's, reaching higher and stopping sooner.
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
/// The pixel off every screen.
const BLACK: [u8; BGRX_BYTES] = [0; BGRX_BYTES];

/// The pixel painted at `at`: the low bytes of x, then of y, so each point paints apart.
fn pixel(at: Point) -> [u8; BGRX_BYTES] {
    let (x, y) = (at.x.to_le_bytes(), at.y.to_le_bytes());
    [x[0], x[1], y[0], y[1]]
}

/// The screen at `rect`, read at `dpi`, each pixel painted with its desktop point.
fn painted(rect: Rect, dpi: u32) -> FrozenScreen {
    let row = |y| (rect.x..rect.x + rect.w).flat_map(move |x| pixel(Point { x, y }));
    let bgrx = (rect.y..rect.y + rect.h).flat_map(row).collect();
    let size = |n: i32| u32::try_from(n).unwrap();
    let shot = Shot::new(size(rect.w), size(rect.h), bgrx).unwrap();
    FrozenScreen::new(rect, dpi, shot).unwrap()
}

/// Both screens.
fn screens() -> [FrozenScreen; 2] {
    [painted(MAIN, MAIN_DPI), painted(LEFT, LEFT_DPI)]
}

/// The part at `rect`.
fn part(rect: Rect) -> Part {
    Part::new(rect).unwrap()
}

/// The pixel `cut` holds at desktop point `at`, its part starting at `from`.
fn at(cut: &Shot, from: Point, at: Point) -> [u8; BGRX_BYTES] {
    let width = i32::try_from(cut.width()).unwrap();
    let i = usize::try_from((at.y - from.y) * width + (at.x - from.x)).unwrap() * BGRX_BYTES;
    cut.bgrx().expose()[i..i + BGRX_BYTES].try_into().unwrap()
}

/// Every desktop point of `rect`, row by row.
fn points(rect: Rect) -> impl Iterator<Item = Point> {
    (rect.y..rect.y + rect.h)
        .flat_map(move |y| (rect.x..rect.x + rect.w).map(move |x| Point { x, y }))
}

#[test]
fn a_part_spans_two_corners_in_either_order_with_both_corner_pixels_inside() {
    let (a, b) = (Point { x: 5, y: 9 }, Point { x: 2, y: 3 });
    let want = Rect {
        x: 2,
        y: 3,
        w: 4,
        h: 7,
    };
    assert_eq!(Part::spanning(a, b).map(Part::rect), Some(want));
    assert_eq!(Part::spanning(b, a).map(Part::rect), Some(want));
    let crossed = Part::spanning(Point { x: 2, y: 9 }, Point { x: 5, y: 3 });
    assert_eq!(crossed.map(Part::rect), Some(want));
    let same = Part::spanning(a, a).map(Part::rect);
    assert_eq!(
        same,
        Some(Rect {
            x: 5,
            y: 9,
            w: 1,
            h: 1
        }),
        "one pixel"
    );
}

#[test]
fn a_part_whose_corners_pass_the_pixel_range_is_none() {
    let far = Point { x: i32::MAX, y: 0 };
    assert_eq!(
        Part::spanning(far, Point::default()),
        None,
        "no pixel past it"
    );
    let ends = (
        Point { x: i32::MIN, y: 0 },
        Point {
            x: i32::MAX - 1,
            y: 0,
        },
    );
    assert_eq!(Part::spanning(ends.0, ends.1), None, "wider than the range");
}

#[test]
fn a_cut_inside_one_screen_keeps_its_pixels() {
    let inside = Rect {
        x: 1,
        y: 1,
        w: 3,
        h: 2,
    };
    let cut = part(inside).cut(&screens()).unwrap();
    assert_eq!((cut.width(), cut.height()), (3, 2));
    let from = Point { x: 1, y: 1 };
    assert!(points(inside).all(|p| at(&cut, from, p) == pixel(p)));
}

#[test]
fn a_cut_across_two_screens_of_different_scaling_keeps_exact_pixels() {
    let across = Rect {
        x: -3,
        y: 0,
        w: 6,
        h: 2,
    };
    let cut = part(across).cut(&screens()).unwrap();
    let from = Point { x: -3, y: 0 };
    for p in points(across) {
        assert_eq!(at(&cut, from, p), pixel(p), "{p:?}");
    }
}

#[test]
fn a_pixel_on_no_screen_is_black() {
    let gap = Rect {
        x: -2,
        y: 1,
        w: 4,
        h: 3,
    };
    let cut = part(gap).cut(&screens()).unwrap();
    let from = Point { x: -2, y: 1 };
    for p in points(gap) {
        let on_left = p.x < 0 && p.y < LEFT.y + LEFT.h;
        let want = if p.x >= 0 || on_left { pixel(p) } else { BLACK };
        assert_eq!(at(&cut, from, p), want, "{p:?}");
    }
}

#[test]
fn a_cut_touching_no_screen_is_none() {
    let away = Rect {
        x: 100,
        y: 100,
        w: 2,
        h: 2,
    };
    assert_eq!(part(away).cut(&screens()), None);
    assert_eq!(part(MAIN).cut(&[]), None, "no screen frozen");
}

#[test]
fn a_cut_never_holds_more_bytes_than_its_screens_together() {
    let one = Rect {
        x: 0,
        y: 0,
        w: 2,
        h: 2,
    };
    let far = Rect {
        x: 10,
        y: 10,
        ..one
    };
    let apart = [painted(one, MAIN_DPI), painted(far, MAIN_DPI)];
    let both = Rect {
        w: 12,
        h: 12,
        ..one
    };
    assert_eq!(part(both).cut(&apart), None, "the gap between them");
    let as_much = Rect { h: 4, ..one };
    assert!(part(as_much).cut(&apart).is_some(), "exactly their bytes");
}

/// Whether desktop point `p` lies on `rect`.
fn on(rect: Rect, p: Point) -> bool {
    (rect.x..rect.x + rect.w).contains(&p.x) && (rect.y..rect.y + rect.h).contains(&p.y)
}

proptest::proptest! {
    #[test]
    fn every_pixel_of_a_cut_is_the_screens_or_black(x in -9..9, y in -4..6, w in 1..15, h in 1..9) {
        let rect = Rect { x, y, w, h };
        let cut = part(rect).cut(&screens());
        let lit = |p: Point| on(MAIN, p) || on(LEFT, p);
        let fits = w * h <= MAIN.w * MAIN.h + LEFT.w * LEFT.h;
        proptest::prop_assert_eq!(cut.is_some(), fits && points(rect).any(lit));
        let from = Point { x, y };
        for p in points(rect).filter(|_| cut.is_some()) {
            let want = if lit(p) { pixel(p) } else { BLACK };
            proptest::prop_assert_eq!(at(cut.as_ref().unwrap(), from, p), want, "{:?}", p);
        }
    }
}
