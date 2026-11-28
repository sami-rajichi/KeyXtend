//! The picture editor's model (plan task 12): each tool's click, move, click; Undo and Redo with `undo_max`; the
//! Eraser; Steps; Text, Arabic too; edge 18; and the properties that Undo then Redo gives the same picture.

#![allow(
    clippy::unwrap_used,
    reason = "these helpers run only in tests, where a failed setup should fail the test"
)]

use super::*;
use kx_module_api::Redacted;
use kx_module_api::ports::{CropKey, Drawing, Mark, Scale, Shape, TextSize, Thickness};

use crate::UNDO_MAX;

mod crop;
mod eraser;
mod history;
mod props;
mod reach;
mod text;
mod tools;

/// The test picture's width and height, in its pixels.
const W: u32 = 120;
const H: u32 = 90;
/// The theme's thickness steps, in the picture's pixels; the middle one draws a pen 6 wide.
const THICK: [Thickness; 5] = [
    thickness(2, 8, 16, 8),
    thickness(4, 12, 20, 12),
    thickness(6, 18, 24, 16),
    thickness(10, 28, 30, 24),
    thickness(16, 40, 36, 32),
];
/// The theme's text sizes, in the picture's pixels; the middle one is 28 high, its letters 16 wide.
const TEXT: [TextSize; 4] = [
    TextSize {
        height: 14,
        letter: 8,
    },
    TextSize {
        height: 20,
        letter: 11,
    },
    TextSize {
        height: 28,
        letter: 16,
    },
    TextSize {
        height: 40,
        letter: 23,
    },
];
/// The theme's slop, in window pixels.
const SLOP: u16 = 8;
/// The theme's handle click area across, in window pixels: eight each way of a handle.
const HIT: u16 = 16;
/// The least side a moved frame handle leaves, in the picture's pixels.
const MIN: u16 = 8;
/// The picture shown at its own size.
const SAME: Scale = match Scale::new(Scale::PER) {
    Some(scale) => scale,
    None => panic!("a scale of PER is not 0"),
};

/// A thickness step.
const fn thickness(pen: u16, marker: u16, disc: u16, head: u16) -> Thickness {
    Thickness {
        pen,
        marker,
        disc,
        head,
    }
}

/// The theme's sizes for the editor; the bars' sizes play no part here.
fn fit() -> SnipFit {
    SnipFit {
        strip: 0,
        bar: 0,
        gap: 0,
        thick: THICK.to_vec(),
        text: TEXT.to_vec(),
        slop: SLOP,
        handle: HIT,
    }
}

/// How many steps a list of `len` makes.
fn steps(len: usize) -> u8 {
    u8::try_from(len).unwrap()
}

/// A point of the picture.
const fn pt(x: i32, y: i32) -> Point {
    Point { x, y }
}

/// The test picture's last column and row.
fn far() -> Point {
    pt(i32::try_from(W).unwrap() - 1, i32::try_from(H).unwrap() - 1)
}

/// An editor on the test picture, whose Undo takes back the fewest steps `undo_max` allows.
fn canvas() -> Canvas {
    Canvas::new(W, H, *UNDO_MAX.start(), MIN, fit()).unwrap()
}

/// An editor with `tool` picked.
fn with(tool: Tool) -> Canvas {
    let mut c = canvas();
    c.tool(tool);
    c
}

/// The drawings shown, oldest first.
fn drawn(c: &Canvas) -> Vec<Drawing> {
    c.view().drawings
}

/// What the drawings shown are.
fn marks(c: &Canvas) -> Vec<Mark> {
    drawn(c).into_iter().map(|d| d.mark).collect()
}

/// Two clicks of the picked tool.
fn two(c: &mut Canvas, from: Point, to: Point) -> (Did, Did) {
    (c.press(from, SAME), c.press(to, SAME))
}

/// Crop's two clicks, then its ✓.
fn cut(c: &mut Canvas, from: Point, to: Point) -> Did {
    two(c, from, to);
    c.crop(CropKey::Ok)
}

/// A shape from one click to the next.
const fn shape(shape: Shape, from: Point, to: Point) -> Mark {
    Mark::Shape { shape, from, to }
}

/// Typed text.
fn words(at: Point, text: &str, rtl: bool) -> Mark {
    Mark::Text {
        at,
        text: Redacted::new(text.to_owned()),
        rtl,
    }
}

#[test]
fn a_picture_with_no_pixels_or_past_the_pixel_range_has_no_editor() {
    let undo = *UNDO_MAX.start();
    assert!(Canvas::new(0, H, undo, MIN, fit()).is_none());
    assert!(Canvas::new(W, 0, undo, MIN, fit()).is_none());
    assert!(Canvas::new(u32::MAX, H, undo, MIN, fit()).is_none());
}

#[test]
fn a_new_editor_shows_the_whole_picture_with_nothing_drawn() {
    let view = canvas().view();
    let whole = Rect {
        x: 0,
        y: 0,
        w: i32::try_from(W).unwrap(),
        h: i32::try_from(H).unwrap(),
    };
    assert_eq!(view.crop.rect(), whole);
    assert!(view.drawings.is_empty());
    assert_eq!((view.tool, view.ink), (TOOL, INK));
    assert_eq!((view.pending, view.typing, view.frame), (None, None, None));
    assert!(!view.undo && !view.redo, "nothing to take back or redo");
}
