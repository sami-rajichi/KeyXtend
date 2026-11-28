//! What ends Crop's frame (plan 2026-11-26 task 5): ✗, Esc, Undo and another tool drop it, re-picking Crop keeps it,
//! and while a handle is held ✓ and ✗ wait while Esc lets go of the handle first (Region's edge 4).

use super::*;

/// `PART`'s Top handle picked up and set 5 pixels higher.
fn adjusted() -> Canvas {
    let mut c = framed();
    let top = on(PART, Handle::Top);
    c.press(top, SAME);
    c.press(pt(top.x, top.y - 5), SAME);
    assert_eq!(waiting(&c).part.y, PART.y - 5, "set");
    c
}

#[test]
fn cancel_esc_and_another_tool_drop_the_frame_and_cut_nothing() {
    let ways: [fn(&mut Canvas) -> Did; 3] = [
        |c| c.crop(CropKey::Cancel),
        Canvas::esc,
        |c| c.tool(Tool::Pen),
    ];
    for (i, drop) in ways.into_iter().enumerate() {
        let mut c = adjusted();
        c.ink(Ink::Blue);
        c.size(true);
        assert!(c.view().frame.is_some(), "an ink or a size keeps it");
        assert_eq!(drop(&mut c), Did::Show, "way {i}");
        let view = c.view();
        assert_eq!((view.frame, view.crop.rect()), (None, whole()), "way {i}");
        assert!(!view.undo, "way {i}: no step to take back");
    }
    let mut c = framed();
    c.press(on(PART, Handle::Top), SAME);
    assert_eq!(c.tool(Tool::Pen), Did::Show);
    assert_eq!(
        c.view().frame,
        None,
        "another tool drops it while a handle is held"
    );
    let mut c = with(Tool::Crop);
    assert_eq!(c.crop(CropKey::Cancel), Did::Stay, "nothing waits");
    assert_eq!(c.crop(CropKey::Ok), Did::Stay);
    assert_eq!(c.esc(), Did::Stay, "so the flow takes Esc");
}

#[test]
fn picking_crop_again_keeps_the_frame_and_its_held_handle() {
    let mut c = adjusted();
    let before = waiting(&c);
    assert_eq!(c.tool(Tool::Crop), Did::Stay);
    assert_eq!(waiting(&c), before, "as it was");
    c.press(on(before.part, Handle::Left), SAME);
    let held = waiting(&c);
    assert!(held.frame.grip.is_some());
    assert_eq!(c.tool(Tool::Crop), Did::Stay);
    assert_eq!(waiting(&c), held, "the handle still held");
}

#[test]
fn while_a_handle_is_held_ok_and_cancel_wait_for_its_set() {
    let mut c = framed();
    let right = on(PART, Handle::Right);
    c.press(right, SAME);
    let held = waiting(&c);
    assert_eq!(
        c.crop(CropKey::Ok),
        Did::Stay,
        "an overshooting set never cuts"
    );
    assert_eq!(c.crop(CropKey::Cancel), Did::Stay, "nor drops the frame");
    assert_eq!(waiting(&c), held);
    assert_eq!(c.view().crop.rect(), whole(), "nothing cut");
    c.press(pt(right.x + 9, right.y), SAME);
    assert_eq!(c.crop(CropKey::Ok), Did::Change, "once set, ✓ cuts");
    assert_eq!(c.view().crop.rect(), Rect { w: 50, ..PART });
}

#[test]
fn esc_lets_go_of_a_held_handle_then_drops_the_frame_then_passes() {
    let mut c = adjusted();
    let set = waiting(&c);
    c.press(on(set.part, Handle::Right), SAME);
    assert!(waiting(&c).frame.grip.is_some(), "Right held");
    assert_eq!(c.esc(), Did::Show);
    assert_eq!(waiting(&c), set, "only the hold goes, the Top's set kept");
    assert_eq!(c.esc(), Did::Show);
    assert_eq!(c.view().frame, None, "the next Esc drops the frame");
    assert_eq!(c.esc(), Did::Stay, "and the one after goes to the flow");
}

#[test]
fn undo_drops_a_waiting_frame_first_and_redo_drops_it_too() {
    let mut c = framed();
    assert_eq!(c.undo(), Did::Show);
    assert_eq!(c.view().frame, None);
    assert_eq!(c.undo(), Did::Stay, "nothing else to take back");
    let mut c = framed();
    c.crop(CropKey::Ok);
    c.undo();
    two(&mut c, pt(5, 5), pt(30, 30));
    assert_eq!(c.redo(), Did::Change);
    assert_eq!(c.view().frame, None, "a frame within the old part goes");
    assert_eq!(c.view().crop.rect(), PART);
}

#[test]
fn ok_on_the_part_already_kept_drops_the_frame_and_changes_nothing() {
    let mut c = with(Tool::Crop);
    two(&mut c, pt(0, 0), far());
    assert_eq!(waiting(&c).part, whole());
    assert_eq!(c.crop(CropKey::Ok), Did::Show);
    assert_eq!(c.view().frame, None);
    assert!(!c.view().undo, "no step");
}
