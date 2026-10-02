//! The keyboard always fits the screen it is on, so its size buttons on the top bar's right stay reachable.

#![allow(
    clippy::unwrap_used,
    reason = "these fixtures run only in tests, where a failed setup should fail the test"
)]

use crate::host::{GOLDEN, saved};
use crate::rig::Rig;
use kx_module_api::ports::{BASE_DPI, FaceAction, Point, Rect, Screen, physical};

/// A 150 % screen.
const HIGH_DPI: u32 = 144;
/// A small work area, narrower than the size-1 board at 150 %.
const SMALL: Rect = Rect {
    x: 0,
    y: 0,
    w: 1000,
    h: 600,
};
/// A full-HD work area above a taskbar.
const WIDE: Rect = Rect {
    x: 0,
    y: 0,
    w: 1920,
    h: 1040,
};

fn screen(rect: Rect, dpi: u32) -> Screen {
    Screen {
        rect,
        dpi,
        primary: true,
    }
}

fn placed(x: i32, y: i32, s: Screen) -> FaceAction {
    let rect = Rect { x, y, w: 1, h: 1 };
    FaceAction::Placed {
        rect,
        screens: vec![s],
    }
}

/// The plate's size on screen `s`, in its physical pixels.
fn plate_on(r: &Rig, s: &Screen) -> (i32, i32) {
    let plate = &r.board()["plate"];
    let side = |k: &str| physical(plate[k].as_f64().unwrap(), s.dpi);
    (side("w"), side("h"))
}

/// Where the window is, in physical pixels.
fn at(r: &Rig) -> Point {
    serde_json::from_value(r.state()["at"].clone()).unwrap()
}

/// True when the whole plate lies in `s`'s work area.
fn on_screen(r: &Rig, s: &Screen) -> bool {
    let (from, (wide, high)) = (at(r), plate_on(r, s));
    let work = s.rect;
    from.x >= work.x
        && from.y >= work.y
        && from.x + wide <= work.x + work.w
        && from.y + high <= work.y + work.h
}

#[test]
fn an_oversized_keyboard_shrinks_to_its_screen_and_keeps_its_saved_size() {
    let r = Rig::new(&GOLDEN, Some(("size", 1.8.into())));
    let small = screen(SMALL, HIGH_DPI);
    let reply = r.act(placed(0, 0, small)).unwrap();
    assert!(
        reply.board,
        "the board is drawn again at the size that fits"
    );
    assert!(
        on_screen(&r, &small),
        "plate {:?} at {:?}",
        plate_on(&r, &small),
        at(&r)
    );
    assert!(r.host.sink.writes().is_empty(), "the saved size stays");
    r.act(FaceAction::Bigger).unwrap();
    r.act(FaceAction::Preset { index: 2 }).unwrap();
    assert!(on_screen(&r, &small), "no step grows it past the screen");
}

#[test]
fn a_bigger_step_by_the_right_edge_keeps_the_keyboard_on_its_screen() {
    let r = Rig::new(&GOLDEN, None);
    let wide = screen(WIDE, BASE_DPI);
    r.act(placed(0, 0, wide)).unwrap();
    let edge = WIDE.w - plate_on(&r, &wide).0;
    r.act(placed(edge, 200, wide)).unwrap();
    assert_eq!(at(&r), Point { x: edge, y: 200 });
    let reply = r.act(FaceAction::Bigger).unwrap();
    assert!(reply.board && reply.state, "{reply:?}");
    assert!(
        on_screen(&r, &wide),
        "plate {:?} at {:?}",
        plate_on(&r, &wide),
        at(&r)
    );
    r.act(FaceAction::Preset { index: 2 }).unwrap();
    assert!(on_screen(&r, &wide), "a preset too");
    assert_eq!(
        saved(&r.host).place(),
        Some(Point { x: edge, y: 200 }),
        "only the move saved a place"
    );
}

#[test]
fn a_move_to_a_smaller_screen_fits_it_there() {
    let r = Rig::new(&GOLDEN, Some(("size", 1.5.into())));
    let wide = screen(WIDE, BASE_DPI);
    let mut small = screen(SMALL, HIGH_DPI);
    small.rect.x = WIDE.w;
    small.primary = false;
    let both = vec![wide, small];
    let place = |x: i32| FaceAction::Placed {
        rect: Rect {
            x,
            y: 0,
            w: 1,
            h: 1,
        },
        screens: both.clone(),
    };
    r.act(place(0)).unwrap();
    assert!(on_screen(&r, &wide));
    r.act(place(WIDE.w + 10)).unwrap();
    assert!(
        on_screen(&r, &small),
        "plate {:?} at {:?}",
        plate_on(&r, &small),
        at(&r)
    );
}
