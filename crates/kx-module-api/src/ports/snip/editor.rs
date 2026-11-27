//! The editor (design "The editor"): the picture, its crop and drawings in the picture's own pixels, the tool, ink and
//! size, the text being typed, and where the window opens; a video has none (ADR-0060). Pixels and typed text never
//! print.

use super::frame::Frame;
use super::shot::Shot;
use super::view::{Dim, Keep};
use crate::Redacted;
use crate::ports::{Point, Rect};

listed! {
    /// A tool of the top row.
    pub enum Tool ("The eleven tools, in the top row's order.") {
        /// Keep a part: click the start, move, click the end, adjust its frame, then click ✓.
        Crop,
        /// Draw: click to put the pen down, move, click to lift it.
        Pen,
        /// Draw see-through, as Pen does.
        Highlighter,
        /// A box from one click to the next.
        Box,
        /// A circle in the box from one click to the next.
        Circle,
        /// An arrow from one click to the next.
        Arrow,
        /// A line from one click to the next.
        Line,
        /// Typed text where clicked.
        Text,
        /// A filled box hiding what is under it.
        Cover,
        /// The next number where clicked.
        Steps,
        /// Remove the drawing clicked.
        Eraser,
    }
}

listed! {
    /// An ink of the second row.
    pub enum Ink ("The six inks, in the second row's order.") {
        /// Red.
        Red,
        /// Yellow.
        Yellow,
        /// Green.
        Green,
        /// Blue.
        Blue,
        /// Black.
        Black,
        /// White.
        White,
    }
}

listed! {
    /// A shape drawn from one click to the next.
    pub enum Shape ("Every shape.") {
        /// A box.
        Box,
        /// A circle in that box.
        Circle,
        /// An arrow pointing at the end.
        Arrow,
        /// A line.
        Line,
        /// A filled box.
        Cover,
    }
}

listed! {
    /// An editor key besides tools, inks and sizes; a picture's Copy and Save come as `SnipClick::Flat`.
    pub enum EditKey ("Every editor key.") {
        /// Take back the last step.
        Undo,
        /// Do the step taken back again.
        Redo,
        /// Close, asking first when the snip was not kept.
        Close,
    }
}

listed! {
    /// A key of Crop's frame, which shows only while a frame waits.
    pub enum CropKey ("Both of Crop's keys.") {
        /// Cut to the frame, as one step.
        Ok,
        /// Drop the frame.
        Cancel,
    }
}

/// What a drawing is, in the picture's own pixels.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Mark {
    /// A pen's path.
    Pen(Vec<Point>),
    /// A highlighter's path, drawn see-through.
    Highlight(Vec<Point>),
    /// A shape from one point to another.
    Shape {
        /// Which shape.
        shape: Shape,
        /// The first click.
        from: Point,
        /// The second click.
        to: Point,
    },
    /// Typed text at its point.
    Text {
        /// Where it starts: its top left, or its top right when `rtl`.
        at: Point,
        /// The text.
        text: Redacted<String>,
        /// It reads right to left.
        rtl: bool,
    },
    /// A numbered step.
    Step {
        /// Its centre.
        at: Point,
        /// Its number, from 1.
        n: u16,
    },
}

/// One drawing on the picture.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Drawing {
    /// What it is.
    pub mark: Mark,
    /// Its ink.
    pub ink: Ink,
    /// Its thickness's step, or its text size's for text, from 0. A step past `SnipFit`'s list draws with its last
    /// entry.
    pub size: u8,
}

/// The text being typed, which takes the keyboard's letters as a panel's line does (ADR-0040).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TextEdit {
    /// Where it starts: its top left, or its top right when `rtl`.
    pub at: Point,
    /// The text so far.
    pub text: Redacted<String>,
    /// Its caret, in UTF-16 units of the text.
    pub caret: Redacted<u32>,
    /// It reads right to left.
    pub rtl: bool,
}

/// The part of a picture Crop keeps: a rectangle never empty and wholly inside it, in its pixels.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Crop(Rect);

impl Crop {
    /// `rect` as a crop of a `width` by `height` picture; `None` when it is empty or not wholly inside.
    #[must_use]
    pub fn new(rect: Rect, width: u32, height: u32) -> Option<Self> {
        let inside = |at: i32, len: i32, side: u32| {
            let end = i64::from(at) + i64::from(len);
            at >= 0 && len > 0 && end <= i64::from(side)
        };
        let fits = inside(rect.x, rect.w, width) && inside(rect.y, rect.h, height);
        fits.then_some(Self(rect))
    }

    /// The kept rectangle.
    #[must_use]
    pub const fn rect(self) -> Rect {
        self.0
    }
}

/// Crop's frame after its second click, until ✓ cuts to it or ✗ drops it: the part it would keep, inside the part
/// kept so far, in the picture's pixels.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct CropFrame {
    /// The part it would keep.
    pub part: Rect,
    /// Its frame: within the part kept so far, at least `[snip] frame_min_px`, and the handle held.
    pub frame: Frame,
}

/// A picture's tools and drawings.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PictureEdit {
    /// The part of the picture Crop keeps; the whole picture before any.
    pub crop: Crop,
    /// The drawings, oldest first.
    pub drawings: Vec<Drawing>,
    /// The tool picked.
    pub tool: Tool,
    /// The ink picked.
    pub ink: Ink,
    /// The thickness's step, or the text size's for Text, from 0.
    pub size: u8,
    /// The first click of a two-click gesture, while the second is awaited; the ui draws on to the pointer.
    pub pending: Option<Point>,
    /// Crop's frame waiting for ✓ or ✗, if any.
    pub frame: Option<CropFrame>,
    /// The text being typed, if any.
    pub typing: Option<TextEdit>,
    /// Undo acts.
    pub undo: bool,
    /// Redo acts.
    pub redo: bool,
}

/// The editor window; it never takes focus. The ui sends its size, place and maximised state as `SnipClick::Placed`,
/// and the module keeps them in `[snip] editor` and `editor_full`, like the keyboard's own (`FaceAction::Placed`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Editor {
    /// The picture.
    pub picture: Shot,
    /// Its tools and drawings.
    pub body: PictureEdit,
    /// Why Save is dimmed, if it is.
    pub save_dim: Option<Dim>,
    /// What was just done, which shows a short Copied or Saved.
    pub done: Option<Keep>,
    /// Where the window opens, in physical pixels, as last placed on a screen still there; `None` opens it at its
    /// default (edge 19).
    pub place: Option<Rect>,
    /// The work area it filled when last maximised: it opens maximised on the screen holding it, over `place`, or at
    /// its default on the main screen once that screen is gone (edge 9); `None` opens it unmaximised.
    pub full: Option<Rect>,
}
