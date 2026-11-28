//! Undo and Redo: each step done keeps the step that takes it back, up to `undo_max`; past it the oldest can no longer
//! be taken back, and what it did stays.

use std::collections::VecDeque;

use kx_module_api::ports::{Crop, Drawing};

use super::text::Typing;
use super::{Canvas, Did};

/// What Undo and Redo change: the drawings, oldest first, and the part Crop keeps.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct Sheet {
    pub(super) drawings: Vec<Drawing>,
    pub(super) crop: Crop,
}

/// One change to the sheet.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum Step {
    /// Put this drawing on top.
    Add(Drawing),
    /// Take the top drawing off.
    Take,
    /// Remove the drawing at this place.
    Erase(usize),
    /// Put this drawing back at this place.
    Put(usize, Drawing),
    /// Keep this part.
    Crop(Crop),
}

impl Step {
    /// Makes the change on `sheet` and gives the step that takes it back; `None` when it does not fit the sheet.
    fn flip(self, sheet: &mut Sheet) -> Option<Self> {
        let drawings = &mut sheet.drawings;
        Some(match self {
            Self::Add(drawing) => {
                drawings.push(drawing);
                Self::Take
            }
            Self::Take => Self::Add(drawings.pop()?),
            Self::Erase(at) => {
                let drawing = (at < drawings.len()).then(|| drawings.remove(at))?;
                Self::Put(at, drawing)
            }
            Self::Put(at, drawing) => {
                (at <= drawings.len()).then(|| drawings.insert(at, drawing))?;
                Self::Erase(at)
            }
            Self::Crop(crop) => Self::Crop(std::mem::replace(&mut sheet.crop, crop)),
        })
    }
}

/// The steps Undo and Redo take.
#[derive(Debug)]
pub(super) struct History {
    /// What takes back each step done, oldest first.
    done: VecDeque<Step>,
    /// What does again each step taken back, the last one taken back on top.
    undone: Vec<Step>,
    /// The most steps Undo takes back.
    max: usize,
}

impl History {
    /// A history that takes back up to `max` steps.
    pub(super) fn new(max: u16) -> Self {
        Self {
            done: VecDeque::new(),
            undone: Vec::new(),
            max: usize::from(max),
        }
    }

    /// Does `step` on `sheet` as a new step: nothing is left to redo, and past `max` the oldest step can no longer be
    /// taken back. True when it fit.
    pub(super) fn push(&mut self, step: Step, sheet: &mut Sheet) -> bool {
        let Some(back) = step.flip(sheet) else {
            return false;
        };
        self.undone.clear();
        self.keep(back);
        true
    }

    /// Keeps `back` for Undo; past `max` the oldest step can no longer be taken back.
    fn keep(&mut self, back: Step) {
        self.done.push_back(back);
        if self.done.len() > self.max {
            self.done.pop_front();
        }
    }

    /// Puts `step`, never done, alone on Redo's pile, as a text typed then taken back; Undo keeps every step it had.
    fn shelve(&mut self, step: Step) {
        self.undone.clear();
        self.undone.push(step);
    }

    /// Takes the last step done back; true when there was one.
    fn undo(&mut self, sheet: &mut Sheet) -> bool {
        let again = self.done.pop_back().and_then(|back| back.flip(sheet));
        again.map(|step| self.undone.push(step)).is_some()
    }

    /// Does the last step taken back again; true when there was one.
    fn redo(&mut self, sheet: &mut Sheet) -> bool {
        let back = self.undone.pop().and_then(|step| step.flip(sheet));
        back.map(|back| self.keep(back)).is_some()
    }

    /// Undo has a step to take back.
    pub(super) fn can_undo(&self) -> bool {
        !self.done.is_empty()
    }

    /// Redo has a step to do again.
    pub(super) fn can_redo(&self) -> bool {
        !self.undone.is_empty()
    }
}

impl Canvas {
    /// Undo: a gesture under way or Crop's frame is dropped first, and the text being typed goes, for Redo to bring
    /// back; else the last step is taken back.
    pub fn undo(&mut self) -> Did {
        if self.open.take().is_some() | self.waiting.take().is_some() {
            return Did::Show;
        }
        if let Some(typing) = self.typing.take() {
            if let Some(text) = self.typed(typing) {
                self.history.shelve(Step::Add(text));
            }
            return Did::Show;
        }
        changed(self.history.undo(&mut self.sheet))
    }

    /// Redo, offered only while no shown text is being typed, as that text would replace Redo's step: an empty text and
    /// a gesture under way or Crop's frame are dropped, then the last step taken back is done again.
    pub fn redo(&mut self) -> Did {
        if !self.can_redo() {
            return Did::Stay;
        }
        let dropped = self.typing.take().is_some()
            | self.open.take().is_some()
            | self.waiting.take().is_some();
        let shown = if dropped { Did::Show } else { Did::Stay };
        shown.max(changed(self.history.redo(&mut self.sheet)))
    }

    /// Redo acts: a step was taken back, and no shown text is being typed.
    pub(super) fn can_redo(&self) -> bool {
        self.history.can_redo() && !self.typing.as_ref().is_some_and(Typing::shows)
    }

    /// Does `step` as a new step.
    pub(super) fn apply(&mut self, step: Step) -> Did {
        changed(self.history.push(step, &mut self.sheet))
    }
}

/// A change when `done`, else nothing.
const fn changed(done: bool) -> Did {
    if done { Did::Change } else { Did::Stay }
}
