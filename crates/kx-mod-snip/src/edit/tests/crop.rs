//! Crop's frame (plan 2026-11-26 task 5, edges 1-3 and 5): the second click shows a frame and cuts nothing, its handles
//! adjust it within what shows and at least `frame_min_px`, and ✓ cuts as one step that Undo takes back and Redo does
//! again.

use super::*;
use crate::frame;
use kx_module_api::ports::{CropFrame, Frame, Grip, Handle};

mod keys;

/// The part the tests frame: from (20, 10) to (60, 40), both pixels in.
const PART: Rect = Rect {
    x: 20,
    y: 10,
    w: 41,
    h: 31,
};

/// The whole test picture.
fn whole() -> Rect {
    canvas().view().crop.rect()
}

/// An editor with Crop's frame round `PART`.
fn framed() -> Canvas {
    let mut c = with(Tool::Crop);
    two(&mut c, pt(20, 10), pt(60, 40));
    c
}

/// The frame waiting.
fn waiting(c: &Canvas) -> CropFrame {
    c.view().frame.expect("a frame waits")
}

/// Where `handle` sits on `r`.
fn on(r: Rect, handle: Handle) -> Point {
    frame::spot(r, handle).unwrap()
}

/// `handle` picked up on its spot of `PART`, then set at `to`.
fn moved(handle: Handle, to: Point) -> Canvas {
    let mut c = framed();
    c.press(on(PART, handle), SAME);
    assert_eq!(c.press(to, SAME), Did::Show, "{handle:?} set");
    c
}

#[test]
fn a_second_click_shows_a_frame_round_both_corners_and_cuts_nothing() {
    let mut c = with(Tool::Crop);
    assert_eq!(two(&mut c, pt(60, 40), pt(20, 10)), (Did::Show, Did::Show));
    let frame = Frame {
        within: whole(),
        min: MIN,
        grip: None,
    };
    let want = CropFrame { part: PART, frame };
    assert_eq!(c.view().frame, Some(want), "clicked in either order");
    assert_eq!(c.view().crop.rect(), whole(), "nothing cut yet");
    assert_eq!(c.view().pending, None);
    assert!(c.view().undo, "Undo drops it");
}

#[test]
fn each_handle_is_picked_up_then_set_by_the_pointers_travel() {
    let mid = pt(PART.x + PART.w / 2, PART.y + PART.h / 2);
    for handle in Handle::ALL {
        let mut c = framed();
        let at = on(PART, handle);
        assert_eq!(c.press(at, SAME), Did::Show, "{handle:?}");
        let grip = Grip { handle, at };
        assert_eq!(waiting(&c).frame.grip, Some(grip));
        let out = |a: i32, m: i32, by: i32| a + (a - m).signum() * by;
        let to = pt(out(at.x, mid.x, 3), out(at.y, mid.y, 2));
        assert_eq!(c.press(to, SAME), Did::Show);
        let want = frame::moved(PART, grip, to, whole(), MIN).unwrap();
        assert_ne!(want, PART, "{handle:?} moved");
        assert_eq!(waiting(&c).part, want);
        assert_eq!(waiting(&c).frame.grip, None, "{handle:?} set");
        assert_eq!(c.view().crop.rect(), whole(), "still nothing cut");
    }
}

#[test]
fn a_click_within_reach_of_a_handle_picks_it_up_at_each_scale_and_one_elsewhere_does_nothing() {
    let corner = on(PART, Handle::BottomRight);
    // At 2, 1, 0.667 (a 150 % screen's picture shown at 1 / 1.5, where 11.99 rounds to 12) and 0.5 window pixels a
    // picture pixel.
    for shown in [Scale::PER * 2, Scale::PER, 667, Scale::PER / 2] {
        let at = Scale::new(shown).unwrap();
        // Half the click area, in window pixels, in the picture's pixels at this scale, rounded half up.
        let (half, per, by) = (u32::from(HIT / 2), Scale::PER, shown);
        let reach = i32::try_from((2 * half * per + by) / (2 * by)).unwrap();
        let mut c = framed();
        let edge = pt(corner.x + reach, corner.y + reach);
        assert_eq!(c.press(edge, at), Did::Show, "at its reach, at {shown}");
        let grip = Grip {
            handle: Handle::BottomRight,
            at: edge,
        };
        assert_eq!(waiting(&c).frame.grip, Some(grip));
        let mut c = framed();
        let past = pt(corner.x + reach + 1, corner.y);
        assert_eq!(c.press(past, at), Did::Stay, "past its reach, at {shown}");
        assert_eq!(c.press(pt(110, 85), at), Did::Stay, "away from the frame");
        assert_eq!(waiting(&c), waiting(&framed()), "the frame as it was");
    }
    let mut c = framed();
    assert_eq!(c.press(pt(40, 25), SAME), Did::Stay, "inside the frame");
    assert_eq!(waiting(&c).frame.grip, None);
}

#[test]
fn edge_1_a_handle_set_past_the_opposite_side_swaps_it() {
    let c = moved(Handle::Right, pt(10, 25));
    let want = Rect {
        x: 10,
        w: 10,
        ..PART
    };
    assert_eq!(waiting(&c).part, want, "never a negative width");
}

#[test]
fn edge_2_a_handle_stops_at_what_shows_and_keeps_frame_min_px() {
    let c = moved(Handle::TopLeft, pt(-30, -30));
    let want = Rect {
        x: 0,
        y: 0,
        w: 61,
        h: 41,
    };
    assert_eq!(waiting(&c).part, want, "at the picture's edge");
    let c = moved(Handle::Left, pt(58, 25));
    let least = Rect {
        x: 53,
        w: i32::from(MIN),
        ..PART
    };
    assert_eq!(waiting(&c).part, least);
}

#[test]
fn edge_3_a_set_on_the_pick_up_spot_leaves_the_frame_as_it_was() {
    let mut c = framed();
    let off = pt(62, 26);
    c.press(off, SAME);
    assert!(
        waiting(&c).frame.grip.is_some(),
        "Right, picked up off its spot"
    );
    assert_eq!(c.press(off, SAME), Did::Show);
    assert_eq!(waiting(&c), waiting(&framed()));
}

#[test]
fn edge_5_ok_cuts_as_one_step_that_undo_takes_back_and_redo_does_again() {
    let mut c = with(Tool::Line);
    two(&mut c, pt(5, 5), pt(100, 80));
    let line = marks(&c);
    c.tool(Tool::Crop);
    two(&mut c, pt(20, 10), pt(60, 40));
    c.press(on(PART, Handle::Right), SAME);
    c.press(pt(70, 25), SAME);
    let kept = Rect { w: 50, ..PART };
    assert_eq!(c.crop(CropKey::Ok), Did::Change);
    let view = c.view();
    assert_eq!(
        (view.crop.rect(), view.frame, view.tool),
        (kept, None, Tool::Crop)
    );
    assert_eq!(c.undo(), Did::Change);
    assert_eq!(c.view().crop.rect(), whole(), "the whole picture again");
    assert_eq!(marks(&c), line, "with its drawings");
    assert_eq!(c.undo(), Did::Change);
    assert!(drawn(&c).is_empty(), "the frame's moves were no steps");
    c.redo();
    assert_eq!(c.redo(), Did::Change);
    assert_eq!(c.view().crop.rect(), kept, "Redo cuts again");
}

#[test]
fn after_ok_crop_stays_picked_and_a_new_frame_keeps_within_the_part_kept() {
    let mut c = framed();
    c.crop(CropKey::Ok);
    two(&mut c, pt(25, 15), pt(50, 35));
    assert_eq!(waiting(&c).frame.within, PART);
    let new = waiting(&c).part;
    c.press(on(new, Handle::TopLeft), SAME);
    c.press(pt(0, 0), SAME);
    let r = waiting(&c).part;
    assert_eq!((r.x, r.y), (PART.x, PART.y), "it stops at what shows");
}

#[test]
fn a_readers_press_picks_a_handle_up_at_its_spot_and_another_takes_its_place() {
    let mut c = framed();
    assert_eq!(c.handle(Handle::Right), Did::Show);
    let right = Grip {
        handle: Handle::Right,
        at: on(PART, Handle::Right),
    };
    assert_eq!(waiting(&c).frame.grip, Some(right));
    assert_eq!(c.handle(Handle::Left), Did::Show);
    assert_eq!(c.handle(Handle::Left), Did::Stay, "pressed again");
    c.press(pt(10, 25), SAME);
    let want = Rect {
        x: 10,
        w: 51,
        ..PART
    };
    assert_eq!(waiting(&c).part, want, "Left moved, Right dropped");
    assert_eq!(with(Tool::Crop).handle(Handle::Top), Did::Stay, "no frame");
}
