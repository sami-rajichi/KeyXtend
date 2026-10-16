//! What the engine tells the module and the UI; the module drains it after each callback.

use super::{Engine, Mode};
use kx_module_api::Redacted;
use kx_module_api::ports::{GrabState, Point, PointerEvent};
use std::fmt;

/// One thing that happened, for the UI and the module; its `Debug` hides every point.
#[derive(Copy, Clone, PartialEq, Eq)]
pub enum Note {
    /// Right-click or Grab changed; this is the new state.
    State {
        /// Right-click is on.
        rclick: bool,
        /// Where Grab stands.
        grab: GrabState,
    },
    /// A still hold began.
    HoldStart {
        /// Where the press was held back.
        at: Point,
    },
    /// The hold turned into a click or a drag, or a mode switch let it through.
    HoldEnd,
    /// The hold right-clicked.
    Fired {
        /// Where it right-clicked.
        at: Point,
    },
    /// Grab picked up.
    Picked {
        /// Where it picked up.
        at: Point,
    },
    /// Grab dropped.
    Dropped {
        /// Where it dropped.
        at: Point,
    },
    /// A carry was cancelled.
    Cancelled,
    /// The pointer moved while a mode is on or the pad scrolls.
    Moved {
        /// Where it is now.
        at: Point,
    },
    /// A click used the latched modifiers.
    LatchesUsed,
    /// A button went down while the pad scrolls, which may stop it.
    Pressed {
        /// It landed on one of our own windows.
        ours: bool,
    },
}

/// Hides each point, as a click's point can show what was clicked.
impl fmt::Debug for Note {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (name, at) = match *self {
            Self::State { rclick, grab } => {
                return f
                    .debug_struct("State")
                    .field("rclick", &rclick)
                    .field("grab", &grab)
                    .finish();
            }
            Self::HoldEnd => return f.write_str("HoldEnd"),
            Self::Cancelled => return f.write_str("Cancelled"),
            Self::LatchesUsed => return f.write_str("LatchesUsed"),
            Self::Pressed { ours } => {
                return f.debug_struct("Pressed").field("ours", &ours).finish();
            }
            Self::HoldStart { at } => ("HoldStart", at),
            Self::Fired { at } => ("Fired", at),
            Self::Picked { at } => ("Picked", at),
            Self::Dropped { at } => ("Dropped", at),
            Self::Moved { at } => ("Moved", at),
        };
        f.debug_struct(name)
            .field("at", &Redacted::new(at))
            .finish()
    }
}

impl Engine {
    /// Keeps the last point. While a mode is on or the pad scrolls a move is noted, and while it scrolls a press is.
    pub(super) fn track(&mut self, event: PointerEvent) {
        match event {
            PointerEvent::Down { at, ours, .. } => {
                self.last = at;
                if self.scrolling {
                    self.note(Note::Pressed { ours });
                }
            }
            PointerEvent::Up { at, .. } => self.last = at,
            PointerEvent::Move { at } => {
                self.last = at;
                if self.mode != Mode::Off || self.scrolling {
                    self.note(Note::Moved { at });
                }
            }
            PointerEvent::Esc => {}
        }
    }

    /// Notes the new state when it differs from `before`.
    pub(super) fn note_state(&mut self, before: (bool, GrabState)) {
        let (rclick, grab) = self.view();
        if (rclick, grab) != before {
            self.note(Note::State { rclick, grab });
        }
    }

    /// Keeps `note` within the reserved room, so the hot path never allocates; a move replaces a move just before.
    pub(super) fn note(&mut self, note: Note) {
        if let (Note::Moved { .. }, Some(last @ Note::Moved { .. })) = (note, self.notes.last_mut())
        {
            *last = note;
        } else if self.notes.len() < self.notes.capacity() {
            self.notes.push(note);
        }
    }
}
