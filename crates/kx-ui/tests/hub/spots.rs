//! Where the pop-ups go: beside the keyboard, at home with no keyboard, the tray menu by its icon, and the bubble.

#![allow(
    clippy::float_cmp,
    reason = "a panel that fits keeps exactly 1, and one clamped exactly the least share"
)]

use crate::{DPI, Rig, main_screen};
use kx_module_api::ports::{Corner, Point, Rect, Screen, physical};
use kx_theme::Themes;

/// Windows' 150 % DPI.
const DPI_150: u32 = 144;

fn rect(x: i32, y: i32, w: i32, h: i32) -> Rect {
    Rect { x, y, w, h }
}

fn at(x: i32, y: i32) -> Point {
    Point { x, y }
}

/// The theme's bubble diameter and its inset from the screen corner, in pixels at `dpi`.
fn bubble(dpi: u32) -> [i32; 2] {
    let themes = Themes::shipped().expect("the shipped themes");
    let [d, _, inset] = themes.shape.extra.bubble_px;
    [d, inset].map(|v| physical(f64::from(v), dpi))
}

/// The theme's pop-up gaps, to the keyboard and to the screen edge, in pixels at `dpi`.
fn gaps(dpi: u32) -> [i32; 2] {
    let themes = Themes::shipped().expect("the shipped themes");
    themes
        .shape
        .panel
        .gap_px
        .map(|g| physical(f64::from(g), dpi))
}

#[test]
fn a_pop_up_sits_the_theme_gap_above_the_keyboard_centred_or_at_a_left_edge() {
    let rig = Rig::new();
    let kb = rect(460, 600, 1000, 400);
    let y = 600 - 100 - gaps(DPI)[0];
    assert_eq!(
        rig.hub.pop_at(Some(kb), None, 300.0, 100.0),
        Some(at(460 + 500 - 150, y))
    );
    assert_eq!(
        rig.hub.pop_at(Some(kb), Some(500), 300.0, 100.0),
        Some(at(500, y))
    );
}

#[test]
fn with_no_keyboard_a_pop_up_sits_at_the_bottom_centre_of_the_main_screen() {
    let rig = Rig::new();
    let y = 1040 - 60 - gaps(DPI)[1];
    assert_eq!(
        rig.hub.pop_at(None, None, 400.0, 60.0),
        Some(at((1920 - 400) / 2, y))
    );
}

#[test]
fn sizes_and_gaps_follow_the_scale_of_the_screen_they_land_on() {
    let rig = Rig::new();
    let big = Screen {
        rect: rect(1920, 0, 2560, 1400),
        dpi: DPI_150,
        primary: false,
    };
    rig.screens.set_areas(vec![main_screen(), big]);
    let kb = rect(2000, 800, 1500, 500);
    // 200 by 100 Qt units are 300 by 150 pixels at 150 %, and so are the gaps.
    let [near, edge] = gaps(DPI_150);
    assert_eq!(
        rig.hub.pop_at(Some(kb), Some(2000), 200.0, 100.0),
        Some(at(2000, 800 - 150 - near))
    );
    // A click on the hidden icons' flyout, near the bottom of the big screen.
    assert_eq!(
        rig.hub.menu_at(at(4000, 1395), 200.0, 100.0),
        Some(at(4000, 1400 - 150 - edge))
    );
}

#[test]
fn a_tray_click_outside_every_work_area_opens_on_the_main_screen() {
    let rig = Rig::new();
    let below = Screen {
        rect: rect(0, 1080, 1920, 1040),
        dpi: DPI,
        primary: false,
    };
    rig.screens.set_areas(vec![main_screen(), below]);
    // On the main taskbar, though nearer the lower screen's work area.
    let edge = gaps(DPI)[1];
    assert_eq!(
        rig.hub.menu_at(at(1800, 1070), 200.0, 90.0),
        Some(at(1920 - 200 - edge, 1040 - 90 - edge))
    );
}

#[test]
fn a_tray_menu_opens_above_its_anchor_on_the_taskbar() {
    let rig = Rig::new();
    let edge = gaps(DPI)[1];
    let y = 1040 - 90 - edge;
    assert_eq!(
        rig.hub.menu_at(at(1700, 1060), 200.0, 90.0),
        Some(at(1700, y))
    );
    assert_eq!(
        rig.hub.menu_at(at(1900, 1060), 200.0, 90.0),
        Some(at(1920 - 200 - edge, y))
    );
}

#[test]
fn with_no_screen_there_is_no_spot() {
    let rig = Rig::new();
    rig.screens.set_areas(Vec::new());
    assert_eq!(rig.hub.pop_at(None, None, 1.0, 1.0), None);
    assert_eq!(rig.hub.menu_at(at(0, 0), 1.0, 1.0), None);
}

#[test]
fn the_bubble_takes_the_theme_size_at_the_screen_dpi() {
    let rig = Rig::new();
    let [d, inset] = bubble(DPI);
    let at = rig.hub.bubble_at(Point { x: 100, y: 100 }, Corner::Right);
    assert_eq!(
        at,
        Some(Point {
            x: 1920 - d - inset,
            y: 1040 - d - inset
        })
    );
    let mut big = main_screen();
    big.dpi = 2 * DPI;
    rig.screens.set_areas(vec![big]);
    let [d, inset] = bubble(big.dpi);
    let left = rig.hub.bubble_at(Point { x: 100, y: 100 }, Corner::Left);
    assert_eq!(
        left,
        Some(Point {
            x: inset,
            y: 1040 - d - inset
        })
    );
    rig.screens.set_areas(Vec::new());
    assert_eq!(rig.hub.bubble_at(Point::default(), Corner::Left), None);
}

/// Windows' 125 % DPI.
const DPI_125: u32 = 120;

/// A work area `w` by `h` at `dpi`, and a keyboard `kb_h` Qt units high docked at its bottom edge, in its pixels.
fn docked(rig: &Rig, w: i32, h: i32, dpi: u32, kb_h: f64) -> Rect {
    let area = Screen {
        rect: rect(0, 0, w, h),
        dpi,
        primary: true,
    };
    rig.screens.set_areas(vec![area]);
    let kb = physical(kb_h, dpi);
    rect(w / 4, h - gaps(dpi)[1] - kb, w / 2, kb)
}

#[test]
fn a_panel_too_tall_beside_the_keyboard_shrinks_to_the_room_above_it() {
    let rig = Rig::new();
    let kb = docked(&rig, 1366, 720, DPI, 196.0);
    let [near, edge] = gaps(DPI);
    let room = kb.y - near - edge;
    let f = rig
        .hub
        .pop_fit(Some(kb), None, 480.0, 600.0)
        .expect("a spot");
    assert!((f.fit - f64::from(room) / 600.0).abs() < 1e-9, "{}", f.fit);
    assert_eq!(f.y, edge, "its bottom the gap above the keys");
    let small = rig
        .hub
        .pop_fit(Some(kb), None, 480.0, 300.0)
        .expect("a spot");
    assert_eq!(
        (small.fit, small.y),
        (1.0, kb.y - near - 300),
        "one that fits keeps its size"
    );
}

#[test]
fn a_panel_shrinks_at_the_scale_of_its_screen_but_never_below_the_least_share() {
    let rig = Rig::new();
    let kb = docked(&rig, 1920, 1020, DPI_125, 196.0);
    let [near, edge] = gaps(DPI_125);
    let tall = physical(600.0, DPI_125);
    let f = rig
        .hub
        .pop_fit(Some(kb), None, 480.0, 600.0)
        .expect("a spot");
    let room = f64::from(kb.y - near - edge);
    assert!(
        (f.fit - (room / f64::from(tall)).min(1.0)).abs() < 1e-9,
        "{}",
        f.fit
    );
    let least = Themes::shipped()
        .expect("the shipped themes")
        .shape
        .panel
        .min_fit;
    let huge = rig
        .hub
        .pop_fit(Some(kb), None, 480.0, 5000.0)
        .expect("a spot");
    assert_eq!(huge.fit, f64::from(least));
    assert_eq!(huge.y, edge, "over the keys, inside the work area");
    let home = rig.hub.pop_fit(None, None, 480.0, 600.0).expect("a spot");
    assert_eq!(home.fit, 1.0, "with no keyboard nothing shrinks");
}
