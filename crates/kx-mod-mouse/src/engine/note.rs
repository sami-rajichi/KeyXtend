//! What the engine tells the module and the UI; the module drains it after each callback.

use kx_module_api::Redacted;
use kx_module_api::ports::{GrabState, Point};
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
    /// The pointer moved while a mode is on.
    Moved {
        /// Where it is now.
        at: Point,
    },
    /// A click used the latched modifiers.
    LatchesUsed,
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
