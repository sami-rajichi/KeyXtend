//! Where the strip and the record bar go: the strip under the part, above it or at the bottom centre, kept on the screen
//! holding the part; the record bar above the part or at the top centre; and the theme's sizes at the screen's DPI.

#![allow(
    clippy::unwrap_used,
    reason = "these helpers run only in tests, where a failed setup should fail the test"
)]

use super::*;
use crate::part::home;
use kx_platform_fake::painted;
use proptest::prelude::*;

/// Random cases per property.
const CASES: u32 = 256;

/// The screen, at negative coordinates: its rows run from -100 to 199.
const SCREEN: Rect = Rect {
    x: -200,
    y: -100,
    w: 400,
    h: 300,
};
/// The strip's or the bar's size, in physical pixels: 25 rows with its gap.
const SIZE: Size = Size { h: 20, gap: 5 };

/// A part `h` rows tall from row `y`.
fn rows(y: i32, h: i32) -> Part {
    Part::new(Rect {
        x: -50,
        y,
        w: 100,
        h,
    })
    .unwrap()
}

/// A bar `h` rows tall from row `top` lies on `SCREEN`.
fn on_screen(top: i32, h: i32) -> bool {
    top >= SCREEN.y && top + h <= SCREEN.y + SCREEN.h
}

#[test]
fn the_strip_goes_under_the_part() {
    assert_eq!(place(rows(0, 50), SCREEN, SIZE), SnipSpot::Under);
}

#[test]
fn the_strip_goes_above_when_there_is_no_room_under() {
    assert_eq!(place(rows(100, 80), SCREEN, SIZE), SnipSpot::Above);
}

#[test]
fn the_strip_fits_to_the_pixel() {
    assert_eq!(
        place(rows(125, 50), SCREEN, SIZE),
        SnipSpot::Under,
        "ends on row 199"
    );
    assert_eq!(
        place(rows(126, 50), SCREEN, SIZE),
        SnipSpot::Above,
        "one row more"
    );
    assert_eq!(
        place(rows(-75, 260), SCREEN, SIZE),
        SnipSpot::Above,
        "starts on row -100"
    );
    assert_eq!(
        place(rows(-76, 260), SCREEN, SIZE),
        SnipSpot::Bottom,
        "one row less"
    );
}

#[test]
fn the_strip_goes_to_the_bottom_centre_for_a_full_screen() {
    let full = Part::new(SCREEN).unwrap();
    assert_eq!(place(full, SCREEN, SIZE), SnipSpot::Bottom);
    assert_eq!(
        place(rows(-100, 300), SCREEN, SIZE),
        SnipSpot::Bottom,
        "as tall"
    );
}

#[test]
fn the_strip_stays_on_the_screen_holding_the_part() {
    let top = Rect {
        x: 0,
        y: 0,
        w: 60,
        h: 40,
    };
    let low = Rect { y: 40, ..top };
    let screens = [painted(top, 96).unwrap(), painted(low, 144).unwrap()];
    let part = Part::new(Rect {
        x: 5,
        y: 2,
        w: 20,
        h: 42,
    })
    .unwrap();
    let home = home(part, &screens).unwrap();
    assert_eq!(
        home.rect, top,
        "38 rows on the top screen, 4 on the low one"
    );
    let size = Size { h: 6, gap: 2 };
    assert_eq!(
        place(part, low, size),
        SnipSpot::Under,
        "the low one has room"
    );
    assert_eq!(
        place(part, home.rect, size),
        SnipSpot::Bottom,
        "yet it stays"
    );
}

#[test]
fn the_record_bar_goes_above_the_part() {
    assert_eq!(bar(rows(0, 50), SCREEN, SIZE), SnipSpot::Above);
    assert_eq!(
        bar(rows(-75, 50), SCREEN, SIZE),
        SnipSpot::Above,
        "to the pixel"
    );
}

#[test]
fn the_record_bar_goes_to_the_top_centre_for_a_full_screen() {
    let full = Part::new(SCREEN).unwrap();
    assert_eq!(bar(full, SCREEN, SIZE), SnipSpot::Top);
    assert_eq!(
        bar(rows(-76, 50), SCREEN, SIZE),
        SnipSpot::Top,
        "no room above"
    );
}

#[test]
fn the_themes_sizes_scale_with_the_screens_dpi() {
    let read = |s: Size| (s.h(), s.gap());
    assert_eq!(read(Size::scaled(44, 10, 96)), (44, 10));
    assert_eq!(read(Size::scaled(44, 10, 144)), (66, 15));
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(CASES))]

    #[test]
    fn the_strip_goes_under_first_then_above_and_always_lies_on_the_screen(
        y in -100..200_i32,
        tall in 1..=300_i32,
        h in 1..60_i32,
        gap in 0..20_i32,
    ) {
        let (part, size) = (rows(y, tall.min(200 - y)), Size { h, gap });
        let r = part.rect();
        let under = on_screen(r.y + r.h + gap, h);
        let above = on_screen(r.y - gap - h, h);
        let strip = if under { SnipSpot::Under } else if above { SnipSpot::Above } else { SnipSpot::Bottom };
        prop_assert_eq!(place(part, SCREEN, size), strip);
        let record = if above { SnipSpot::Above } else { SnipSpot::Top };
        prop_assert_eq!(bar(part, SCREEN, size), record);
    }
}
