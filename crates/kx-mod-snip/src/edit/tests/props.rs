//! Properties over random clicks and keys at random scales: Undo then Redo gives the same drawings and crop, each Undo
//! and Redo shows the picture of its step, a change is told exactly when they change, and a new drawing lies inside
//! what shows.

use std::collections::VecDeque;

use proptest::collection::vec;
use proptest::prelude::*;
use proptest::sample::select;

use super::*;
use kx_module_api::ports::Handle;

/// The most clicks and keys one case takes.
const OPS: usize = 40;
/// How far past the picture's edges a click may land.
const PAST: i32 = 20;

/// One click or key.
#[derive(Clone, Debug)]
enum Op {
    Tool(Tool),
    Ink(Ink),
    Size(bool),
    Press(Point, Scale),
    /// Crop from one point to another, its ✓, then the tool picked before again.
    Crop(Point, Point),
    /// Crop's ✓ or ✗.
    Key(CropKey),
    /// A reader's press on a handle of Crop's frame.
    Handle(Handle),
    /// Our keyboard's Esc.
    Esc,
    Stroke(Vec<Point>),
    Undo,
    Redo,
    Write(String),
    Erase,
    End,
}

/// A click on or near the picture.
fn point() -> impl Strategy<Value = Point> {
    let far = far();
    (-PAST..=far.x + PAST, -PAST..=far.y + PAST).prop_map(|(x, y)| pt(x, y))
}

/// The picture shown at half, once or twice its size.
fn scale() -> impl Strategy<Value = Scale> {
    let shown = [Scale::PER / 2, Scale::PER, Scale::PER * 2];
    select(shown.to_vec()).prop_filter_map("a scale is never 0", Scale::new)
}

fn op() -> impl Strategy<Value = Op> {
    prop_oneof![
        2 => select(Tool::ALL.to_vec()).prop_map(Op::Tool),
        1 => select(Ink::ALL.to_vec()).prop_map(Op::Ink),
        1 => any::<bool>().prop_map(Op::Size),
        6 => (point(), scale()).prop_map(|(at, scale)| Op::Press(at, scale)),
        2 => (point(), point()).prop_map(|(from, to)| Op::Crop(from, to)),
        1 => select(CropKey::ALL.to_vec()).prop_map(Op::Key),
        1 => select(Handle::ALL.to_vec()).prop_map(Op::Handle),
        1 => Just(Op::Esc),
        2 => vec(point(), 0..5).prop_map(Op::Stroke),
        2 => Just(Op::Undo),
        1 => Just(Op::Redo),
        1 => "[a-zم ]{0,3}".prop_map(Op::Write),
        1 => Just(Op::Erase),
        1 => Just(Op::End),
    ]
}

/// Clicks and keys with no letters, so no text joins the drawings and each step is one click.
fn quiet() -> impl Strategy<Value = Op> {
    op().prop_filter("no letters", |op| !matches!(op, Op::Write(_)))
}

fn run(c: &mut Canvas, op: &Op) -> Did {
    match op {
        Op::Tool(tool) => c.tool(*tool),
        Op::Ink(ink) => c.ink(*ink),
        Op::Size(up) => c.size(*up),
        Op::Press(at, scale) => c.press(*at, *scale),
        Op::Crop(from, to) => {
            let tool = c.view().tool;
            let did = [
                c.tool(Tool::Crop),
                c.press(*from, SAME),
                c.press(*to, SAME),
                c.crop(CropKey::Ok),
            ];
            did.into_iter().fold(c.tool(tool), Did::max)
        }
        Op::Key(key) => c.crop(*key),
        Op::Handle(handle) => c.handle(*handle),
        Op::Esc => c.esc(),
        Op::Stroke(path) => c.stroke(path),
        Op::Undo => c.undo(),
        Op::Redo => c.redo(),
        Op::Write(text) => c.write(text),
        Op::Erase => c.erase(),
        Op::End => c.end(),
    }
}

/// What Undo and Redo change: the drawings and the crop.
fn picture(c: &Canvas) -> (Vec<Drawing>, Crop) {
    let view = c.view();
    (view.drawings, view.crop)
}

/// Every point a drawing holds.
fn points(mark: &Mark) -> Vec<Point> {
    match mark {
        Mark::Pen(path) | Mark::Highlight(path) => path.clone(),
        Mark::Shape { from, to, .. } => vec![*from, *to],
        Mark::Text { at, .. } | Mark::Step { at, .. } => vec![*at],
    }
}

proptest! {
    #[test]
    fn undo_then_redo_gives_the_same_picture(ops in vec(op(), 0..OPS)) {
        let mut c = canvas();
        for op in &ops {
            run(&mut c, op);
        }
        c.end();
        c.tool(c.view().tool);
        let before = picture(&c);
        let mut undone = 0;
        while c.undo() == Did::Change {
            undone += 1;
            prop_assert!(undone <= OPS, "Undo never stops");
        }
        for _ in 0..undone {
            prop_assert_eq!(c.redo(), Did::Change);
        }
        prop_assert_eq!(picture(&c), before);
    }

    #[test]
    fn each_undo_and_redo_shows_the_picture_before_or_after_its_step(ops in vec(quiet(), 0..OPS)) {
        let max = usize::from(*UNDO_MAX.start());
        let mut c = canvas();
        let (mut past, mut future) = (VecDeque::new(), Vec::new());
        for op in &ops {
            let before = picture(&c);
            if run(&mut c, op) != Did::Change {
                continue;
            }
            let now = picture(&c);
            match op {
                Op::Undo => {
                    prop_assert_eq!(Some(now), past.pop_back(), "Undo shows the picture before");
                    future.push(before);
                }
                Op::Redo => {
                    prop_assert_eq!(Some(now), future.pop(), "Redo shows the picture after");
                    past.push_back(before);
                }
                _ => {
                    past.push_back(before);
                    if past.len() > max {
                        past.pop_front();
                    }
                    future.clear();
                }
            }
        }
    }

    #[test]
    fn a_change_is_told_exactly_when_the_picture_changes(ops in vec(op(), 0..OPS)) {
        let mut c = canvas();
        for op in &ops {
            let before = picture(&c);
            let did = run(&mut c, op);
            prop_assert_eq!(did == Did::Change, picture(&c) != before, "{:?}", op);
        }
    }

    #[test]
    fn a_waiting_frame_lies_inside_what_shows_and_cuts_nothing(ops in vec(op(), 0..OPS)) {
        let mut c = canvas();
        for op in &ops {
            let before = picture(&c);
            run(&mut c, op);
            let view = c.view();
            let Some(f) = view.frame else {
                continue;
            };
            let (r, k) = (f.part, view.crop.rect());
            prop_assert_eq!(f.frame.within, k, "{:?}", op);
            let inside = r.w > 0 && r.h > 0 && r.x >= k.x && r.y >= k.y;
            prop_assert!(inside && r.x + r.w <= k.x + k.w && r.y + r.h <= k.y + k.h, "{:?} past {:?}", r, k);
            prop_assert_eq!(picture(&c), before, "a frame waits after {:?}", op);
        }
    }

    #[test]
    fn a_new_drawing_lies_inside_what_shows(ops in vec(op(), 0..OPS)) {
        let mut c = canvas();
        for op in &ops {
            let had = drawn(&c).len();
            run(&mut c, op);
            let view = c.view();
            // A crop step may end a text first, which was drawn inside the part shown before the crop.
            let added = view.drawings.len() > had && !matches!(op, Op::Undo | Op::Redo | Op::Crop(..));
            let Some(new) = view.drawings.last().filter(|_| added) else {
                continue;
            };
            let r = view.crop.rect();
            for p in points(&new.mark) {
                let inside = (r.x..r.x + r.w).contains(&p.x) && (r.y..r.y + r.h).contains(&p.y);
                prop_assert!(inside, "{:?} outside {:?} after {:?}", p, r, op);
            }
        }
    }
}
