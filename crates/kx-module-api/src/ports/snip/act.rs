//! The clicks Snip's windows send: their keys, a click on the frozen screen or the picture, a pen's path, a caret, and
//! the editor's flattened picture, whose pixels never print.

use std::num::NonZeroU32;

use super::editor::{CropKey, EditKey, Ink, Tool};
use super::frame::Handle;
use super::panel::{FolderKey, SnipKeyboard, SnipTab, VideoRow};
use super::shot::Shot;
use super::view::{AskKey, BarKey, Keep, RecKey, StripKey};
use crate::Redacted;
use crate::ports::{Point, Rect, Screen};

/// One thickness step of the editor, in the picture's own pixels, as Copy and Save draw it.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub struct Thickness {
    /// The width of a pen's line and of the shapes' lines.
    pub pen: u16,
    /// The Highlighter's width.
    pub marker: u16,
    /// A step's disc, across.
    pub disc: u16,
    /// How far an arrow's head reaches back from its tip.
    pub head: u16,
}

/// One text size of the editor, in the picture's own pixels.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub struct TextSize {
    /// A line's height.
    pub height: u16,
    /// An average letter's width, as the font's metrics give it.
    pub letter: u16,
}

/// What the ui's theme sets that Snip places, steps and aims by: the bars' heights and gap in logical pixels, the
/// editor's sizes in the picture's own pixels, and a click's slop in the editor's window pixels. A drawing's step past
/// the end of `thick` or `text` takes that list's last entry.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SnipFit {
    /// The strip's height.
    pub strip: u16,
    /// The record bar's height.
    pub bar: u16,
    /// The gap between a part and the strip or bar.
    pub gap: u16,
    /// The editor's thickness steps, thinnest first.
    pub thick: Vec<Thickness>,
    /// The editor's text sizes, smallest first.
    pub text: Vec<TextSize>,
    /// How far a click may miss a drawing and still land on it, and how near two clicks are one spot, in window pixels.
    pub slop: u16,
    /// A frame handle's click area across, in logical pixels on the frozen screens and in the editor's window.
    pub handle: u16,
}

/// How large the editor shows its picture: window pixels for every `PER` of the picture's pixels, never 0.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Scale(NonZeroU32);

impl Scale {
    /// The picture's pixels a scale counts window pixels for.
    pub const PER: u32 = 1_000;

    /// `shown` window pixels for every `PER` of the picture's; `None` for 0.
    #[must_use]
    pub const fn new(shown: u32) -> Option<Self> {
        match NonZeroU32::new(shown) {
            Some(shown) => Some(Self(shown)),
            None => None,
        }
    }

    /// The window pixels for every `PER` of the picture's.
    #[must_use]
    pub const fn shown(self) -> u32 {
        self.0.get()
    }
}

/// What a click on Snip's windows names.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SnipClick {
    /// The theme's sizes; the module places and steps by the last one heard.
    Fit(SnipFit),
    /// Each screen as the `Screens` port gives it, whose work area (taskbar out, physical pixels) holds the record bar
    /// and a recorded video's strip on the live screen; told at start and on each change, the last one heard counts.
    Screens(Vec<Screen>),
    /// The keyboard did as `SnipAside` asked and the view told before it shows, so the screens may freeze; an answer
    /// with an older round, or a late one, does nothing.
    Aside,
    /// A capture window could not be kept out of captures, so it hides while a video records: told after `Screens` at
    /// each start while one could not, and at once when one fails during a snip.
    Bare,
    /// A capture bar key.
    Bar(BarKey),
    /// A click on the frozen screen, in desktop pixels: a corner, a window or a screen.
    Spot(Point),
    /// A reader's press on a frame's handle: it picks that handle up, dropping the held one; on the held one it does
    /// nothing.
    Handle(Handle),
    /// A strip key.
    Strip(StripKey),
    /// A record bar key.
    Record(RecKey),
    /// A tool.
    Tool(Tool),
    /// An ink.
    Ink(Ink),
    /// The thickness's or text size's minus or plus.
    Size {
        /// Plus.
        up: bool,
    },
    /// A click on the picture: a gesture's start or end, a step, a text's place or an erase.
    Press {
        /// Where, in the picture's pixels.
        at: Point,
        /// How large the picture showed, which turns the slop into its pixels.
        scale: Scale,
    },
    /// A pen's or highlighter's path since its first click, in the picture's pixels; the last point lifts it.
    Stroke(Vec<Point>),
    /// A click on the text being typed: its caret goes this many UTF-16 units in.
    Caret(Redacted<u32>),
    /// An editor key.
    Edit(EditKey),
    /// Crop's ✓ or ✗, while its frame waits.
    Crop(CropKey),
    /// The editor ended at `rect` after a move, a resize, a maximise or a restore, with the screens the ui sees, in
    /// physical pixels as `FaceAction::Placed`; the module keeps it for next time.
    Placed {
        /// The editor's window, or the box a maximised one comes back to.
        rect: Rect,
        /// The work area it fills while maximised; `None` when it is not.
        full: Option<Rect>,
        /// Each screen as the `Screens` port gives it.
        screens: Vec<Screen>,
    },
    /// The editor's picture flattened at its own pixels, for its Copy or Save, or the question's Save.
    Flat {
        /// Copy or Save.
        keep: Keep,
        /// The pixels, the drawings drawn in.
        shot: Shot,
    },
    /// An answer to the close question.
    Ask(AskKey),
    /// The hand pressed the physical Esc key (ADR-0059): over frozen screens it does what our keyboard's Esc does, and
    /// at any other time, or while a system dialog sits in front, nothing, as that Esc is the app's.
    Esc,
    /// Open the Snip panel from the tray's item; the bar's settings key is `Bar(BarKey::Settings)`.
    Open,
    /// Show a panel page.
    Tab(SnipTab),
    /// The keyboard during a capture.
    Keyboard(SnipKeyboard),
    /// A Video row's minus or plus, or of a pair its first or second choice.
    Step {
        /// The row.
        row: VideoRow,
        /// Plus, or the second choice.
        up: bool,
    },
    /// A folder row's key.
    Folder(FolderKey),
    /// Close the panel.
    ClosePanel,
}
