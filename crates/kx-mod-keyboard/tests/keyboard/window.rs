//! The face's size buttons, resize and placement: redraws, saves in one write, and the window's screen DPI.

use crate::host::{GOLDEN, saved};
use crate::rig::{BOARD, Rig, STATE};
use kx_mod_keyboard::place::home;
use kx_module_api::ports::{
    BASE_DPI, FaceAction, FaceError, FaceReply, Point, Rect, Screen, physical,
};
use kx_module_api::{WriteError, keys};
use serde_json::{Value, json};

/// The size-1 plate in logical pixels.
const PLATE: (f64, f64) = (670.78, 196.0);

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
    let rect = Rect { x, y, w: 1, h: 1 };
    let screens = screens.to_vec();
    FaceAction::Placed { rect, screens }
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
fn the_first_placement_starts_home_and_saves_nothing() {
    let r = Rig::new(&GOLDEN, None);
    assert!(r.state()["at"].is_null(), "no screen known yet");
    let screens = [screen(0, BASE_DPI, true)];
    assert_eq!(r.act(placed(5, 5, &screens)), Ok(STATE));
    assert_eq!(r.state()["at"], at(home(PLATE, &screens)));
    assert!(r.host.sink.writes().is_empty());
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
    assert_eq!(r.state()["at"], at(home(PLATE, &one)));
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
    assert_eq!(r.act(FaceAction::ResizeEnd), Ok(BOARD));
    assert!((saved(&r.host).size - 1.25).abs() < f64::EPSILON);
}
