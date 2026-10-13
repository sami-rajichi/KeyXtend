//! The face's size buttons, resize and placement: redraws, saves in one write, and the window's screen DPI.

#![allow(
    clippy::unwrap_used,
    reason = "these fixtures run only in tests, where a failed setup should fail the test"
)]

use crate::host::{GOLDEN, saved};
use crate::rig::{BOARD, Rig, STATE};
use kx_mod_keyboard::place::{Extent, home};
use kx_module_api::ports::{
    BASE_DPI, FaceAction, FaceError, FaceReply, Point, Rect, Screen, physical,
};
use kx_module_api::{WriteError, keys};
use serde_json::{Value, json};

/// The size-1 plate in logical pixels.
const PLATE: (f64, f64) = (670.78, 196.0);
/// How far the resize corner hangs past the plate at size 1: half the shipped themes' 20 px corner.
const OVERHANG: f64 = 10.0;

fn screen(x: i32, dpi: u32, primary: bool) -> Screen {
    let rect = Rect {
        x,
        y: 0,
        w: 1920,
        h: 1040,
    };
    Screen { rect, dpi, primary }
}

fn placed(x: i32, y: i32, screens: &[Screen]) -> FaceAction {
    place_with(x, y, screens, 0.0)
}

/// The window at `x`, `y` with the resize corner hanging `overhang` past the plate at size 1.
fn place_with(x: i32, y: i32, screens: &[Screen], overhang: f64) -> FaceAction {
    let rect = Rect { x, y, w: 1, h: 1 };
    let screens = screens.to_vec();
    FaceAction::Placed {
        rect,
        screens,
        overhang,
    }
}

fn at(p: Option<Point>) -> Value {
    json!(p)
}

#[test]
fn size_buttons_redraw_the_board_and_save_the_size() {
    let r = Rig::new(&GOLDEN, None);
    assert_eq!(r.act(FaceAction::Bigger), Ok(BOARD));
    assert_eq!(r.board()["size"], 1.05);
    assert_eq!(r.act(FaceAction::Preset { index: 2 }), Ok(BOARD));
    assert_eq!(
        (r.board()["preset"].clone(), saved(&r.host).size),
        (json!(2), 1.5)
    );
    assert_eq!(r.act(FaceAction::Smaller), Ok(BOARD));
    let missing = r.act(FaceAction::Preset { index: 9 });
    assert_eq!(missing, Err(FaceError::NoChoice(9)));
}

#[test]
fn a_size_that_cannot_be_saved_holds_and_shows_the_settings_notice() {
    let r = Rig::new(&GOLDEN, None);
    r.host.sink.refuse(Some(WriteError::Unsaved));
    let want = FaceReply {
        notice: Some(keys::SETTINGS_UNSAVED),
        ..BOARD
    };
    assert_eq!(r.act(FaceAction::Bigger), Ok(want));
    assert_eq!(r.board()["size"], 1.05);
}

#[test]
fn a_move_that_cannot_be_saved_holds_and_shows_the_settings_notice() {
    let r = Rig::new(&GOLDEN, None);
    let screens = [screen(0, BASE_DPI, true)];
    r.act(placed(0, 0, &screens)).unwrap();
    r.host.sink.refuse(Some(WriteError::Unsaved));
    let want = FaceReply {
        notice: Some(keys::SETTINGS_UNSAVED),
        ..STATE
    };
    assert_eq!(r.act(placed(300, 200, &screens)), Ok(want));
    assert_eq!(r.state()["at"], json!({ "x": 300, "y": 200 }), "it stays");
    assert!(r.host.sink.writes().is_empty());
}

#[test]
fn the_first_placement_starts_home_and_saves_nothing() {
    let r = Rig::new(&GOLDEN, None);
    assert!(r.state()["at"].is_null(), "no screen known yet");
    let screens = [screen(0, BASE_DPI, true)];
    assert_eq!(r.act(placed(5, 5, &screens)), Ok(STATE));
    assert_eq!(r.state()["at"], at(home(PLATE.into(), &screens)));
    assert!(r.host.sink.writes().is_empty());
}

#[test]
fn placement_counts_the_resize_corner_hanging_past_the_plate() {
    let r = Rig::new(&GOLDEN, None);
    let screens = [screen(0, BASE_DPI, true)];
    r.act(place_with(0, 0, &screens, OVERHANG)).unwrap();
    let whole = Extent {
        plate: PLATE,
        overhang: OVERHANG,
    };
    assert_eq!(r.state()["at"], at(home(whole, &screens)));
    let side = |v: f64| physical(v, BASE_DPI);
    let (w, h) = (side(PLATE.0 + OVERHANG), side(PLATE.1 + OVERHANG));
    r.act(place_with(1900, 1000, &screens, OVERHANG)).unwrap();
    assert_eq!(r.state()["at"], json!({ "x": 1920 - w, "y": 1040 - h }));
    r.act(FaceAction::Bigger).unwrap();
    let grown = whole.scale(r.board()["size"].as_f64().unwrap()).whole();
    let p: Point = serde_json::from_value(r.state()["at"].clone()).unwrap();
    let reach = (p.x + side(grown.0), p.y + side(grown.1));
    assert_eq!(reach, (1920, 1040), "a bigger step keeps the corner on");
    let odd = Rig::new(&GOLDEN, None);
    odd.act(place_with(0, 0, &screens, f64::NAN)).unwrap();
    assert_eq!(
        odd.state()["at"],
        at(home(PLATE.into(), &screens)),
        "no number, no corner"
    );
}

#[test]
fn a_move_is_kept_on_its_screen_and_saved_in_one_write() {
    let r = Rig::new(&GOLDEN, None);
    let screens = [screen(0, BASE_DPI, true)];
    r.act(placed(0, 0, &screens)).unwrap();
    assert_eq!(
        r.act(placed(300, 200, &screens)),
        Ok(FaceReply::default()),
        "already there"
    );
    assert_eq!(r.state()["at"], json!({ "x": 300, "y": 200 }));
    assert_eq!(
        r.act(placed(1800, 200, &screens)),
        Ok(STATE),
        "past the edge"
    );
    assert_eq!(r.state()["at"], json!({ "x": 1249, "y": 200 }));
    assert_eq!(r.host.sink.writes().len(), 2);
    assert_eq!(saved(&r.host).place(), Some(Point { x: 1249, y: 200 }));
}

#[test]
fn new_screens_bring_the_saved_place_back_without_saving() {
    let r = Rig::new(&GOLDEN, None);
    let two = [screen(0, BASE_DPI, true), screen(1920, BASE_DPI, false)];
    r.act(placed(0, 0, &two)).unwrap();
    r.act(placed(2000, 100, &two)).unwrap();
    let one = [screen(0, BASE_DPI, true)];
    assert_eq!(
        r.act(placed(2000, 100, &one)),
        Ok(STATE),
        "its screen is gone"
    );
    assert_eq!(r.state()["at"], at(home(PLATE.into(), &one)));
    let home = r.state()["at"].clone();
    let (x, y) = (home["x"].as_i64().unwrap(), home["y"].as_i64().unwrap());
    let (x, y) = (i32::try_from(x).unwrap(), i32::try_from(y).unwrap());
    r.act(placed(x, y, &two)).unwrap();
    assert_eq!(
        r.state()["at"],
        json!({ "x": 2000, "y": 100 }),
        "the screen came back"
    );
    assert_eq!(r.host.sink.writes().len(), 1, "only the move saved");
}

#[test]
fn a_resize_counts_the_plate_in_the_pixels_of_the_windows_screen() {
    let r = Rig::new(&GOLDEN, None);
    let screens = [screen(0, 144, true)];
    r.act(placed(0, 0, &screens)).unwrap();
    r.act(FaceAction::ResizeBegin { x: 0, y: 0 }).unwrap();
    let quarter = physical(PLATE.0, 144) / 4;
    assert_eq!(r.act(FaceAction::ResizeAt { x: quarter, y: 0 }), Ok(BOARD));
    assert_eq!(r.board()["size"], 1.25, "a quarter of the plate at 144 DPI");
    assert!(
        r.host.sink.writes().is_empty(),
        "nothing saved while it moves"
    );
    let moved_up = FaceReply {
        state: true,
        ..BOARD
    };
    assert_eq!(
        r.act(FaceAction::ResizeEnd),
        Ok(moved_up),
        "it grew past the bottom, so it moves up"
    );
    assert!((saved(&r.host).size - 1.25).abs() < f64::EPSILON);
}

/// What the window sends after a display scale change: the same spot, on the same screen at its new DPI.
fn rescaled(r: &Rig, dpi: u32) -> Result<FaceReply, FaceError> {
    let p: Point = serde_json::from_value(r.state()["at"].clone()).unwrap();
    r.act(placed(p.x, p.y, &[screen(0, dpi, true)]))
}

#[test]
fn a_scale_change_fits_the_keyboard_again_for_its_new_size_and_saves_nothing() {
    let r = Rig::new(&GOLDEN, None);
    r.act(placed(0, 0, &[screen(0, BASE_DPI, true)])).unwrap();
    assert_eq!(
        rescaled(&r, 144),
        Ok(STATE),
        "home moved for the bigger plate"
    );
    assert_eq!(
        r.state()["at"],
        at(home(PLATE.into(), &[screen(0, 144, true)]))
    );
    r.act(placed(300, 200, &[screen(0, 144, true)])).unwrap();
    assert_eq!(r.host.sink.writes().len(), 1, "only the move saved");
    assert_eq!(
        rescaled(&r, 120),
        Ok(FaceReply::default()),
        "a moved one stays"
    );
    assert_eq!(
        r.host.sink.writes().len(),
        1,
        "a scale change saves nothing"
    );
}

#[test]
fn after_a_scale_change_a_resize_counts_the_new_pixels() {
    let r = Rig::new(&GOLDEN, None);
    r.act(placed(0, 0, &[screen(0, 120, true)])).unwrap();
    rescaled(&r, 144).unwrap();
    r.act(FaceAction::ResizeBegin { x: 0, y: 0 }).unwrap();
    let quarter = physical(PLATE.0, 144) / 4;
    r.act(FaceAction::ResizeAt { x: quarter, y: 0 }).unwrap();
    assert_eq!(
        r.board()["size"],
        1.25,
        "a quarter of the plate at the new 144 DPI"
    );
}
