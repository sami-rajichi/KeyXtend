//! The work area the ui tells (`SnipClick::Screens`): the record bar and a recorded video's strip keep inside it on
//! the live screen; a picture's strip stays on the frozen screen, and without a work area told the screen is used.

use kx_module_api::ports::{BarKey, Picking, RecKey, Rect, Screen, SnipClick, SnipStage};

use super::fakes::SCREEN;
use super::rig::{NEAR, Rig};
use crate::strip::{self, Size};

/// The screen's work area: its taskbar takes its bottom 8 pixels.
const WORK: Rect = Rect {
    x: 0,
    y: 0,
    w: 64,
    h: 40,
};

/// Tells the actor each screen's work area: `area`, on a screen of 96 dpi.
fn told(rig: &mut Rig, area: Rect) {
    let screen = Screen {
        rect: area,
        dpi: 96,
        primary: true,
    };
    rig.act(SnipClick::Screens(vec![screen]));
}

/// The record bar's screen and spot as told, against where it goes in `area`.
fn bar_in(rig: &Rig, area: Rect) {
    let SnipStage::Record(rec) = rig.stage() else {
        panic!("it records");
    };
    let size = Size::scaled(rig.actor.fit.bar, rig.actor.fit.gap, 96);
    assert_eq!(
        (rec.screen, rec.spot),
        (area, strip::bar(rec.part, area, size))
    );
}

#[test]
fn the_record_bar_and_a_recorded_videos_strip_keep_inside_the_work_area() {
    let mut rig = Rig::new();
    told(&mut rig, WORK);
    rig.record();
    bar_in(&rig, WORK);
    rig.act(SnipClick::Record(RecKey::Stop));
    let SnipStage::Strip(taken) = rig.stage() else {
        panic!("the video's strip shows");
    };
    let size = Size::scaled(rig.actor.fit.strip, rig.actor.fit.gap, 96);
    assert_eq!(
        (taken.screen, taken.spot),
        (WORK, strip::place(taken.part, WORK, size))
    );
}

#[test]
fn a_pictures_strip_stays_on_the_frozen_screen() {
    let mut rig = Rig::new();
    told(&mut rig, WORK);
    rig.snap();
    let SnipStage::Strip(taken) = rig.stage() else {
        panic!("the strip shows");
    };
    assert_eq!(taken.screen, SCREEN);
}

#[test]
fn without_a_work_area_of_its_screen_the_screen_is_used() {
    let mut rig = Rig::new();
    rig.record();
    bar_in(&rig, SCREEN);
    let mut rig = Rig::new();
    let elsewhere = Rect { x: 64, ..WORK };
    told(&mut rig, elsewhere);
    rig.record();
    bar_in(&rig, SCREEN);
}

/// The capture bar's view told last.
fn picking(rig: &Rig) -> Picking {
    let SnipStage::Pick(picking) = rig.stage() else {
        panic!("not picking: {:?}", rig.stage());
    };
    picking
}

#[test]
fn a_video_regions_first_corner_keeps_the_shown_part_to_its_screen() {
    let mut rig = Rig::new();
    rig.key();
    for (way, within) in [(BarKey::VideoRegion, Some(SCREEN)), (BarKey::Region, None)] {
        rig.act(SnipClick::Bar(way));
        assert_eq!(picking(&rig).within, None, "{way:?}: no corner yet");
        rig.act(SnipClick::Spot(NEAR));
        assert_eq!(picking(&rig).within, within, "{way:?}");
    }
}
