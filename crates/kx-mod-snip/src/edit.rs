//! The picture editor's model (design "The editor"): the crop and the drawings in the picture's own pixels, the tool,
//! ink and sizes, the gesture under way, Crop's waiting frame, the text being typed, and Undo and Redo. Pure: the actor
//! feeds it clicks.

mod cut;
mod gesture;
mod history;
mod hit;
mod text;

use kx_module_api::ports::{Crop, Ink, PictureEdit, Point, Rect, SnipFit, Tool};

use cut::Waiting;
use history::{History, Sheet};
use text::Typing;

/// The tool a new editor starts with.
const TOOL: Tool = Tool::Pen;
/// The ink a new editor starts with.
const INK: Ink = Ink::Red;

/// A list's length as a step count; a longer list stops at the most steps a count holds.
fn count<T>(list: &[T]) -> u8 {
    u8::try_from(list.len()).unwrap_or(u8::MAX)
}

/// What a click or a key did to the editor, least first.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Did {
    /// Nothing changed.
    Stay,
    /// Only the view changed: the tool, ink or size, the gesture under way, or the text being typed.
    Show,
    /// The picture changed: a drawing, a crop, an erase, an undo or a redo; the flow hears `In::Changed`.
    Change,
}

/// A thickness or a text size: its step through the theme's list, and the list's length.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
struct Size {
    step: u8,
    count: u8,
}

impl Size {
    /// The middle of a list of `count` steps, where a new editor starts.
    const fn middle(count: u8) -> Self {
        Self {
            step: count / 2,
            count,
        }
    }

    /// The last step of the list, or 0 for an empty one.
    const fn last(self) -> u8 {
        self.count.saturating_sub(1)
    }

    /// One step up or down, stopping at the list's ends; true when it moved.
    fn by(&mut self, up: bool) -> bool {
        let was = self.step;
        self.step = if up {
            self.step.saturating_add(1).min(self.last())
        } else {
            self.step.saturating_sub(1)
        };
        self.step != was
    }

    /// The theme's new list of `count` steps: the step stays, or stops at the new end; true when it moved.
    fn fit(&mut self, count: u8) -> bool {
        let was = self.step;
        self.count = count;
        self.step = self.step.min(self.last());
        self.step != was
    }
}

/// The picture editor: what shows, and the steps Undo and Redo take.
#[derive(Debug)]
pub struct Canvas {
    width: u32,
    height: u32,
    sheet: Sheet,
    history: History,
    tool: Tool,
    ink: Ink,
    thick: Size,
    text: Size,
    /// The theme's sizes and slop, which the Eraser and the gestures measure by.
    fit: SnipFit,
    /// The first click of a two-click gesture, while the second is awaited.
    open: Option<Point>,
    /// Crop's frame after its second click, until ✓ or ✗.
    waiting: Option<Waiting>,
    /// The least side a moved frame handle leaves, in the picture's pixels (`[snip] frame_min_px`).
    min: u16,
    typing: Option<Typing>,
}

impl Canvas {
    /// An editor on a `width` by `height` picture, whose Undo takes back up to `undo_max` steps, whose frame keeps at
    /// least `min` pixels a side, and whose sizes step through the theme's lists in `fit`; `None` when a side is 0 or
    /// past the pixel range.
    #[must_use]
    pub fn new(width: u32, height: u32, undo_max: u16, min: u16, fit: SnipFit) -> Option<Self> {
        let whole = Rect {
            x: 0,
            y: 0,
            w: i32::try_from(width).ok()?,
            h: i32::try_from(height).ok()?,
        };
        let crop = Crop::new(whole, width, height)?;
        Some(Self {
            width,
            height,
            sheet: Sheet {
                drawings: Vec::new(),
                crop,
            },
            history: History::new(undo_max),
            tool: TOOL,
            ink: INK,
            thick: Size::middle(count(&fit.thick)),
            text: Size::middle(count(&fit.text)),
            fit,
            open: None,
            waiting: None,
            min,
            typing: None,
        })
    }

    /// What the editor's picture shows now.
    #[must_use]
    pub fn view(&self) -> PictureEdit {
        let open = self.open.is_some() || self.waiting.is_some() || self.typing.is_some();
        PictureEdit {
            crop: self.sheet.crop,
            drawings: self.sheet.drawings.clone(),
            tool: self.tool,
            ink: self.ink,
            size: self.picked().step,
            pending: self.open,
            frame: self.frame(),
            typing: self.typing.as_ref().map(Typing::view),
            undo: open || self.history.can_undo(),
            redo: self.can_redo(),
        }
    }

    /// A tool picked: the text being typed ends and a gesture under way is dropped, as by a click elsewhere; Crop's
    /// frame goes only for another tool.
    pub fn tool(&mut self, tool: Tool) -> Did {
        let ended = self.end();
        let picked = self.tool != tool;
        let dropped = self.open.take().is_some() | (picked && self.waiting.take().is_some());
        self.tool = tool;
        let shown = if dropped || picked {
            Did::Show
        } else {
            Did::Stay
        };
        ended.max(shown)
    }

    /// An ink picked, for the next drawings and the text being typed.
    pub fn ink(&mut self, ink: Ink) -> Did {
        let picked = self.ink != ink;
        self.ink = ink;
        if picked { Did::Show } else { Did::Stay }
    }

    /// The picked tool's minus or plus: the text size for Text, else the thickness.
    pub fn size(&mut self, up: bool) -> Did {
        let size = if self.tool == Tool::Text {
            &mut self.text
        } else {
            &mut self.thick
        };
        if size.by(up) { Did::Show } else { Did::Stay }
    }

    /// The theme's sizes changed; each size keeps its step, or stops at its list's new end.
    pub fn fit(&mut self, fit: SnipFit) -> Did {
        let moved = self.thick.fit(count(&fit.thick)) | self.text.fit(count(&fit.text));
        self.fit = fit;
        if moved { Did::Show } else { Did::Stay }
    }

    /// The picked tool's size: the text size for Text, else the thickness.
    const fn picked(&self) -> Size {
        match self.tool {
            Tool::Text => self.text,
            _ => self.thick,
        }
    }

    /// `at` moved onto the nearest pixel the picture shows, so a drawing begun outside is clipped (edge 18).
    fn clamp(&self, at: Point) -> Point {
        let r = self.sheet.crop.rect();
        // A crop is never empty, so its last pixel is at or after its first.
        Point {
            x: at.x.clamp(r.x, r.x + r.w - 1),
            y: at.y.clamp(r.y, r.y + r.h - 1),
        }
    }
}

#[cfg(test)]
mod tests;
