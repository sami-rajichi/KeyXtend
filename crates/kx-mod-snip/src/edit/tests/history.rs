//! Undo and Redo: one step at a time, a gesture under way first, `undo_max` dropping the oldest step, a Crop undone,
//! and Steps counting on from what shows (edge 18).

use super::*;

/// An editor that dropped Steps at each `x`, on one row.
fn steps(xs: impl IntoIterator<Item = i32>) -> Canvas {
    let mut c = with(Tool::Steps);
    for x in xs {
        c.press(pt(x, 10), SAME);
    }
    c
}

#[test]
fn undo_takes_back_the_last_drawing_and_redo_brings_it_back() {
    let mut c = steps([10, 20]);
    let both = drawn(&c);
    assert!(c.view().undo && !c.view().redo);
    assert_eq!(c.undo(), Did::Change);
    assert_eq!(drawn(&c), both[..1]);
    assert!(c.view().redo);
    assert_eq!(c.redo(), Did::Change);
    assert_eq!(drawn(&c), both);
    assert_eq!(c.redo(), Did::Stay, "nothing left to redo");
}

#[test]
fn undo_drops_a_gesture_under_way_first() {
    let mut c = steps([10]);
    c.tool(Tool::Box);
    c.press(pt(30, 30), SAME);
    assert_eq!(c.undo(), Did::Show);
    assert_eq!(c.view().pending, None);
    assert_eq!(drawn(&c).len(), 1, "the step stays");
    assert_eq!(c.undo(), Did::Change);
    assert!(drawn(&c).is_empty());
}

#[test]
fn redo_drops_a_gesture_under_way_so_nothing_starts_outside_what_shows() {
    let mut c = with(Tool::Crop);
    cut(&mut c, pt(10, 10), pt(59, 49));
    c.undo();
    c.tool(Tool::Box);
    c.press(pt(100, 80), SAME);
    assert_eq!(c.redo(), Did::Change);
    assert_eq!(c.view().pending, None, "the gesture under way is dropped");
    c.press(pt(30, 30), SAME);
    assert!(drawn(&c).is_empty(), "the next click starts afresh");
}

#[test]
fn a_new_step_after_an_undo_leaves_nothing_to_redo() {
    let mut c = steps([10, 20]);
    c.undo();
    c.press(pt(30, 10), SAME);
    assert!(!c.view().redo);
    assert_eq!(c.redo(), Did::Stay);
}

#[test]
fn undo_and_redo_with_nothing_to_do_change_nothing() {
    let mut c = canvas();
    assert_eq!(c.undo(), Did::Stay);
    assert_eq!(c.redo(), Did::Stay);
}

#[test]
fn edge_18_undo_max_drops_the_oldest_step() {
    let max = usize::from(*UNDO_MAX.start());
    let last = i32::try_from(max).unwrap() + 2;
    let mut c = steps(1..=last);
    let mut undone = 0;
    for _ in 0..last {
        if c.undo() == Did::Change {
            undone += 1;
        }
    }
    assert_eq!(undone, max);
    assert_eq!(
        drawn(&c).len(),
        2,
        "the two oldest steps can no longer be taken back, and stay"
    );
    assert!(!c.view().undo);
}

#[test]
fn edge_18_undo_after_a_crop_shows_the_whole_picture_with_its_drawings() {
    let mut c = with(Tool::Line);
    two(&mut c, pt(5, 5), pt(100, 80));
    let whole = c.view().crop;
    c.tool(Tool::Crop);
    cut(&mut c, pt(20, 20), pt(40, 40));
    assert_ne!(c.view().crop, whole);
    assert_eq!(c.undo(), Did::Change);
    assert_eq!(c.view().crop, whole);
    assert_eq!(
        marks(&c),
        [shape(Shape::Line, pt(5, 5), pt(100, 80))],
        "the line, whole"
    );
    assert_eq!(c.redo(), Did::Change);
    let kept = Rect {
        x: 20,
        y: 20,
        w: 21,
        h: 21,
    };
    assert_eq!(c.view().crop.rect(), kept);
}

#[test]
fn edge_18_steps_count_on_from_what_shows_after_an_undo() {
    let mut c = steps([10, 20, 30]);
    c.undo();
    c.undo();
    c.press(pt(40, 10), SAME);
    let shown = [(10, 1), (40, 2)].map(|(x, n)| Mark::Step { at: pt(x, 10), n });
    assert_eq!(marks(&c), shown);
}
