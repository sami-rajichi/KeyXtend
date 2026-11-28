//! Clicks on the frozen screen, read by the part rules: a region's corners in either order, the front window, the
//! screen holding a spot, and the screen holding the part, which places the strip and the record bar.

use super::*;
use crate::strip::{self, Size};
use kx_module_api::ports::SnipSpot;

/// The theme's bar height in the tests, in logical pixels.
const BAR_H: u16 = 4;
/// The theme's gap in the tests, in logical pixels.
const GAP: u16 = 1;

/// The right screen, at 150 %.
fn right() -> Home {
    Home {
        rect: RIGHT,
        dpi: 144,
    }
}

/// The part taken and the screen holding it, once the strip shows.
fn taken(rig: &Rig) -> Option<(Part, Home)> {
    match rig.stage() {
        Stage::Strip(strip) => Some((strip.part, strip.home)),
        _ => None,
    }
}

/// Where the strip and the record bar go for the part taken, sized at its screen's DPI.
fn places(rig: &Rig) -> Option<(SnipSpot, SnipSpot)> {
    let (part, home) = taken(rig)?;
    let size = Size::scaled(BAR_H, GAP, home.dpi);
    Some((
        strip::place(part, home.rect, size),
        strip::bar(part, home.rect, size),
    ))
}

#[test]
fn a_region_takes_the_part_between_its_corners_in_either_order() {
    let mut rig = Rig::new();
    rig.picking();
    assert_eq!(rig.on(In::Spot(FAR)), []);
    assert_eq!(rig.on(In::Spot(NEAR)), []);
    let strip = Strip {
        part: part(),
        home: home(),
        held: Held::Picture(shot()),
        done: None,
        frame: Some(region_frame()),
    };
    assert_eq!(rig.stage(), &Stage::Strip(strip));
}

#[test]
fn a_window_spot_takes_its_frame() {
    let mut rig = Rig::new();
    rig.picking();
    rig.on(In::Bar(BarKey::Window));
    rig.on(In::Spot(INSIDE));
    assert_eq!(taken(&rig), Some((part(), home())));
}

#[test]
fn full_screen_takes_the_screen_holding_the_spot() {
    for way in [BarKey::Full, BarKey::VideoFull] {
        let mut rig = Rig::new();
        rig.picking();
        rig.on(In::Bar(way));
        rig.spot(RIGHT.x + 10, 10);
        let screen = Some((Part::new(RIGHT).unwrap(), right()));
        assert_eq!(taken(&rig), screen, "{way:?}");
    }
}

#[test]
fn edge_2_a_part_across_two_screens_takes_the_screen_holding_most() {
    let mut rig = Rig::new();
    rig.picking();
    rig.spot(49, 5);
    rig.spot(36, 2);
    let across = Rect {
        x: 36,
        y: 2,
        w: 14,
        h: 4,
    };
    assert_eq!(taken(&rig), Some((Part::new(across).unwrap(), right())));
    let Stage::Strip(strip) = rig.stage() else {
        panic!("no strip");
    };
    let screens = &rig.flow.frozen().unwrap().screens;
    let cut = Part::new(across).unwrap().cut(screens);
    assert_eq!(strip.picture(), cut.as_ref());
}

#[test]
fn the_strip_goes_under_a_part_and_to_the_bottom_for_a_full_screen() {
    let mut rig = Rig::new();
    rig.picture();
    assert_eq!(places(&rig), Some((SnipSpot::Under, SnipSpot::Above)));
    let mut rig = Rig::new();
    rig.picking();
    rig.on(In::Bar(BarKey::Full));
    rig.spot(RIGHT.x + 10, 10);
    assert_eq!(places(&rig), Some((SnipSpot::Bottom, SnipSpot::Top)));
}

#[test]
fn a_video_keeps_the_screen_holding_its_part() {
    let mut rig = Rig::new();
    rig.recording();
    let Stage::Record(rec) = rig.stage() else {
        panic!("not recording");
    };
    assert_eq!(rec.film.home, home());
}

#[test]
fn a_spot_on_the_strip_does_nothing() {
    let mut rig = Rig::new();
    rig.picture();
    let before = rig.stage().clone();
    assert_eq!(rig.on(In::Spot(INSIDE)), []);
    assert_eq!(rig.stage(), &before);
}
