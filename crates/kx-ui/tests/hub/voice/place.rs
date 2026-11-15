//! Where the bar goes (design "The bar"): just above the keyboard and centred on it, else the bottom centre of its
//! screen, never over it; `place` puts it at the bottom or the top of the screen first (edge 40). Sizes and gaps follow
//! the scale of the keyboard's screen, and the bar always lands on the work area.

use super::{heard, listening_bar, page};
use crate::{DPI, Rig, main_screen};
use kx_module_api::ports::{BarPlace, Point, Rect, Screen, physical};
use kx_theme::Themes;
use proptest::prelude::*;

/// The bar's size in Qt units, at 100 % one pixel each.
const W: f64 = 400.0;
const H: f64 = 56.0;
/// The bar's height in pixels.
const H_PX: i32 = 56;
/// The bar's left edge when centred on the main screen.
const MID_X: i32 = (1920 - 400) / 2;
/// Windows' 150 % DPI.
const DPI_150: u32 = 144;
/// Cases the property tries.
const CASES: u32 = 128;

fn rect(x: i32, y: i32, w: i32, h: i32) -> Rect {
    Rect { x, y, w, h }
}

fn at(x: i32, y: i32) -> Point {
    Point { x, y }
}

/// The theme's gaps, to the keyboard and to the screen edge, in pixels at `dpi`.
fn gaps_at(dpi: u32) -> [i32; 2] {
    let themes = Themes::shipped().expect("the shipped themes");
    let gap = themes.shape.panel.gap_px;
    gap.map(|g| physical(f64::from(g), dpi))
}

/// The theme's gaps in pixels at 100 %.
fn gaps() -> [i32; 2] {
    gaps_at(DPI)
}

/// The bottom of the main screen's work area.
fn floor() -> i32 {
    let s = main_screen().rect;
    s.y + s.h
}

/// A hub showing a listening bar at `place`.
fn placed(place: BarPlace) -> Rig {
    let rig = Rig::new();
    let heard = heard(&rig);
    let mut shown = page(1, listening_bar(), None);
    shown.place = place;
    heard.view(shown);
    rig
}

#[test]
fn the_bar_sits_the_gap_above_the_keyboard_centred_on_it() {
    let rig = placed(BarPlace::Above);
    let kb = rect(300, 600, 1000, 400);
    let want = at(300 + 500 - 200, 600 - gaps()[0] - H_PX);
    assert_eq!(rig.hub.voice_at(Some(kb), W, H), Some(want));
}

#[test]
fn before_any_view_the_bar_goes_above_the_keyboard() {
    let rig = Rig::new();
    let kb = rect(300, 600, 1000, 400);
    let want = at(600, 600 - gaps()[0] - H_PX);
    assert_eq!(rig.hub.voice_at(Some(kb), W, H), Some(want));
}

#[test]
fn with_no_room_above_it_goes_to_the_bottom_centre_of_the_screen() {
    let rig = placed(BarPlace::Above);
    let kb = rect(300, 20, 1000, 500);
    let want = at(MID_X, floor() - gaps()[1] - H_PX);
    assert_eq!(rig.hub.voice_at(Some(kb), W, H), Some(want));
}

#[test]
fn it_is_never_over_the_keyboard_so_with_the_bottom_taken_it_goes_to_the_top() {
    let rig = placed(BarPlace::Above);
    let [gap, edge] = gaps();
    assert!(gap > 1, "the keyboard's gap leaves no room above it");
    let top = edge + H_PX + 1;
    let kb = rect(560, top, 800, floor() - top);
    assert_eq!(rig.hub.voice_at(Some(kb), W, H), Some(at(MID_X, edge)));
}

#[test]
fn with_no_keyboard_it_sits_at_the_bottom_centre_of_the_main_screen() {
    let rig = placed(BarPlace::Above);
    let want = at(MID_X, floor() - gaps()[1] - H_PX);
    assert_eq!(rig.hub.voice_at(None, W, H), Some(want));
}

#[test]
fn place_bottom_puts_it_at_the_bottom_centre_even_with_room_above_the_keyboard() {
    let rig = placed(BarPlace::Bottom);
    let kb = rect(300, 300, 1000, 400);
    let want = at(MID_X, floor() - gaps()[1] - H_PX);
    assert_eq!(rig.hub.voice_at(Some(kb), W, H), Some(want));
}

#[test]
fn place_bottom_with_the_keyboard_at_the_bottom_goes_above_it() {
    let rig = placed(BarPlace::Bottom);
    let kb = rect(300, 600, 1000, floor() - 600);
    let want = at(600, 600 - gaps()[0] - H_PX);
    assert_eq!(rig.hub.voice_at(Some(kb), W, H), Some(want));
}

#[test]
fn place_top_puts_it_at_the_top_centre_with_a_keyboard_or_none() {
    let rig = placed(BarPlace::Top);
    let edge = gaps()[1];
    let kb = rect(300, 600, 1000, 400);
    assert_eq!(rig.hub.voice_at(Some(kb), W, H), Some(at(MID_X, edge)));
    assert_eq!(rig.hub.voice_at(None, W, H), Some(at(MID_X, edge)));
}

#[test]
fn place_top_with_the_keyboard_at_the_top_goes_below_it() {
    let rig = placed(BarPlace::Top);
    let kb = rect(300, 0, 1000, 400);
    let want = at(600, 400 + gaps()[0]);
    assert_eq!(rig.hub.voice_at(Some(kb), W, H), Some(want));
}

#[test]
fn a_bar_wider_than_the_keyboards_side_room_stays_on_the_screen() {
    let rig = placed(BarPlace::Above);
    let kb = rect(1700, 600, 220, 400);
    let edge = gaps()[1];
    let spot = rig.hub.voice_at(Some(kb), W, H).expect("a spot");
    assert_eq!(spot.x, 1920 - edge - 400, "kept inside the right edge");
}

#[test]
fn on_a_second_screen_at_150_percent_size_and_gaps_scale_with_it() {
    let rig = placed(BarPlace::Above);
    let big = Screen {
        rect: rect(1920, 0, 2560, 1400),
        dpi: DPI_150,
        primary: false,
    };
    rig.screens.set_areas(vec![main_screen(), big]);
    // 400 by 56 Qt units are 600 by 84 pixels at 150 %, and so are the gaps.
    let [near, edge] = gaps_at(DPI_150);
    let kb = rect(2000, 800, 1500, 500);
    let want = at(2000 + 750 - 300, 800 - near - 84);
    assert_eq!(rig.hub.voice_at(Some(kb), W, H), Some(want));
    let tall = rect(2000, edge + 84 + 1, 1500, 1400 - edge - 85);
    assert_eq!(
        rig.hub.voice_at(Some(tall), W, H),
        Some(at(1920 + (2560 - 600) / 2, edge)),
        "no room above and the bottom taken, so the top of that screen"
    );
}

#[test]
fn with_no_screen_there_is_no_spot() {
    let rig = placed(BarPlace::Above);
    rig.screens.set_areas(Vec::new());
    assert_eq!(rig.hub.voice_at(None, W, H), None);
    assert_eq!(rig.hub.voice_at(Some(rect(0, 0, 10, 10)), W, H), None);
}

fn place() -> impl Strategy<Value = BarPlace> {
    prop_oneof![
        Just(BarPlace::Above),
        Just(BarPlace::Bottom),
        Just(BarPlace::Top)
    ]
}

/// A keyboard box wholly on the main screen.
fn keyboard() -> impl Strategy<Value = Rect> {
    let s = main_screen().rect;
    (0..s.w, 0..s.h)
        .prop_flat_map(move |(x, y)| (Just(x), Just(y), 1..=s.w - x, 1..=s.h - y))
        .prop_map(|(x, y, w, h)| rect(x, y, w, h))
}

/// True when the `w` by `h` box at `p` shares a pixel with `r`.
fn meets(p: Point, w: i32, h: i32, r: Rect) -> bool {
    p.x < r.x + r.w && r.x < p.x + w && p.y < r.y + r.h && r.y < p.y + h
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(CASES))]

    /// For any place and keyboard on the main screen, the bar lands inside the work area's edge gap, and it stays off
    /// the keyboard whenever there is room above or below it.
    #[test]
    fn the_bar_lands_on_the_work_area_and_off_the_keyboard_while_there_is_room(
        place in place(),
        kb in keyboard(),
        w in 1..=600_i32,
        h in 1..=80_i32,
    ) {
        let rig = placed(place);
        let spot = rig.hub.voice_at(Some(kb), f64::from(w), f64::from(h)).expect("a spot");
        let [gap, edge] = gaps();
        prop_assert!(spot.x >= edge && spot.x + w <= 1920 - edge, "{:?}", spot);
        prop_assert!(spot.y >= edge && spot.y + h <= floor() - edge, "{:?}", spot);
        let above = kb.y >= edge + gap + h;
        let below = kb.y + kb.h + gap + h <= floor() - edge;
        if above || below {
            prop_assert!(!meets(spot, w, h, kb), "{:?} over {:?} at {:?}", spot, kb, place);
        }
    }
}
