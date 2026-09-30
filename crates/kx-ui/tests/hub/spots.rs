//! Where the pop-ups go: beside the keyboard, at home with no keyboard, and the tray menu by its icon.

use crate::Rig;
use kx_module_api::ports::{Point, Rect, Screen};

fn rect(x: i32, y: i32, w: i32, h: i32) -> Rect {
    Rect { x, y, w, h }
}

fn at(x: i32, y: i32) -> Point {
    Point { x, y }
}

#[test]
fn a_pop_up_sits_the_theme_gap_above_the_keyboard_centred_or_at_a_left_edge() {
    let rig = Rig::new();
    let kb = rect(460, 600, 1000, 400);
    assert_eq!(
        rig.hub.pop_at(Some(kb), None, 300.0, 100.0),
        Some(at(810, 490))
    );
    assert_eq!(
        rig.hub.pop_at(Some(kb), Some(500), 300.0, 100.0),
        Some(at(500, 490))
    );
}

#[test]
fn with_no_keyboard_a_pop_up_sits_at_the_bottom_centre_of_the_main_screen() {
    let rig = Rig::new();
    assert_eq!(rig.hub.pop_at(None, None, 400.0, 60.0), Some(at(760, 972)));
}

#[test]
fn sizes_and_gaps_follow_the_scale_of_the_screen_they_land_on() {
    let rig = Rig::new();
    let big = Screen {
        rect: rect(1920, 0, 2560, 1400),
        dpi: 144,
        primary: false,
    };
    rig.screens.set_areas(vec![crate::main_screen(), big]);
    let kb = rect(2000, 800, 1500, 500);
    // 200 by 100 Qt units are 300 by 150 pixels, and the 10-unit gap is 15.
    assert_eq!(
        rig.hub.pop_at(Some(kb), Some(2000), 200.0, 100.0),
        Some(at(2000, 635))
    );
    // A click on the hidden icons' flyout, near the bottom of the big screen.
    assert_eq!(
        rig.hub.menu_at(at(4000, 1395), 200.0, 100.0),
        Some(at(4000, 1238))
    );
}

#[test]
fn a_tray_click_outside_every_work_area_opens_on_the_main_screen() {
    let rig = Rig::new();
    let below = Screen {
        rect: rect(0, 1080, 1920, 1040),
        dpi: 96,
        primary: false,
    };
    rig.screens.set_areas(vec![crate::main_screen(), below]);
    // On the main taskbar, though nearer the lower screen's work area.
    assert_eq!(
        rig.hub.menu_at(at(1800, 1070), 200.0, 90.0),
        Some(at(1712, 942))
    );
}

#[test]
fn a_tray_menu_opens_above_its_anchor_on_the_taskbar() {
    let rig = Rig::new();
    assert_eq!(
        rig.hub.menu_at(at(1700, 1060), 200.0, 90.0),
        Some(at(1700, 942))
    );
    assert_eq!(
        rig.hub.menu_at(at(1900, 1060), 200.0, 90.0),
        Some(at(1712, 942))
    );
}

#[test]
fn with_no_screen_there_is_no_spot() {
    let rig = Rig::new();
    rig.screens.set_areas(Vec::new());
    assert_eq!(rig.hub.pop_at(None, None, 1.0, 1.0), None);
    assert_eq!(rig.hub.menu_at(at(0, 0), 1.0, 1.0), None);
}
