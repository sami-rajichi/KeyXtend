//! Mouse-assist events on the kernel's bus, so the keyboard, the mouse module and the UI never call each other.
//! A click's point can show what was clicked (an on-screen PIN pad), so no event here prints a point or a key.

use super::{Point, Scan, hide_at};
use crate::{Capability, Event, Redacted};
use std::fmt;

/// A mouse-assist mode the keyboard can flip.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum Toggle {
    /// Right-click by holding still.
    RightClick,
    /// The next press picks up, and the next click drops.
    Grab,
}

/// Where Grab stands.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum GrabState {
    /// Grab is off.
    Off,
    /// The next press picks up.
    Armed,
    /// A press is held down until the drop.
    Carrying,
}

/// The keyboard asks the mouse module to flip a mode; it needs `InjectInput`, as a toggle leads to injected clicks.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct AssistToggle {
    /// The mode to flip.
    pub which: Toggle,
}

impl Event for AssistToggle {
    const NAME: &'static str = "assist-toggle";
    const CAPABILITY: Option<Capability> = Some(Capability::InjectInput);
}

/// The mouse module's state, sent on each change and on `AssistAsk`.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct AssistChanged {
    /// The mouse module runs; false when it stops.
    pub live: bool,
    /// Right-click mode is on.
    pub rclick: bool,
    /// Where Grab stands.
    pub grab: GrabState,
}

impl AssistChanged {
    /// The mouse module not running, with every mode off: before it speaks, and after it stops or fails.
    pub const OFF: Self = Self {
        live: false,
        rclick: false,
        grab: GrabState::Off,
    };
}

impl Event for AssistChanged {
    const NAME: &'static str = "assist-changed";
}

/// Asks the mouse module to send `AssistChanged` now, for a module that starts later.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct AssistAsk;

impl Event for AssistAsk {
    const NAME: &'static str = "assist-ask";
}

/// Feedback for the UI; it needs `KeyboardUi` (ADR-0033), and its `Debug` hides every point.
#[derive(Copy, Clone, PartialEq, Eq)]
pub enum AssistNote {
    /// A still hold began.
    HoldStart {
        /// Where the hold began.
        at: Point,
        /// When the ring shows, in milliseconds from the press.
        ring_after_ms: u32,
        /// When the hold fires, in milliseconds from the press.
        hold_ms: u32,
    },
    /// The hold was cancelled, or turned into a click or a drag.
    HoldEnd,
    /// The hold right-clicked.
    Fired {
        /// Where it clicked.
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
}

impl Event for AssistNote {
    const NAME: &'static str = "assist-note";
    const CAPABILITY: Option<Capability> = Some(Capability::KeyboardUi);
}

impl fmt::Debug for AssistNote {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Self::HoldStart {
                at,
                ring_after_ms,
                hold_ms,
            } => f
                .debug_struct("HoldStart")
                .field("at", &Redacted::new(at))
                .field("ring_after_ms", &ring_after_ms)
                .field("hold_ms", &hold_ms)
                .finish(),
            Self::HoldEnd => f.write_str("HoldEnd"),
            Self::Fired { at } => hide_at(f, "Fired", at),
            Self::Picked { at } => hide_at(f, "Picked", at),
            Self::Dropped { at } => hide_at(f, "Dropped", at),
            Self::Cancelled => f.write_str("Cancelled"),
        }
    }
}

/// The latest pointer point, for the badge; it needs `KeyboardUi`, and its `Debug` hides the point.
#[derive(Copy, Clone, PartialEq, Eq)]
pub struct PointerMoved {
    /// The point, in physical pixels.
    pub at: Point,
}

impl Event for PointerMoved {
    const NAME: &'static str = "pointer-moved";
    const CAPABILITY: Option<Capability> = Some(Capability::KeyboardUi);
}

impl fmt::Debug for PointerMoved {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        hide_at(f, "PointerMoved", self.at)
    }
}

/// A button went down while the hook runs, so the scroll module can stop; it carries no point, so it needs no
/// capability.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct PointerPressed {
    /// The press landed on one of our own windows.
    pub ours: bool,
}

impl Event for PointerPressed {
    const NAME: &'static str = "pointer-pressed";
}

/// The keyboard's latched modifiers changed; its `Debug` shows only how many are held.
#[derive(Clone, PartialEq, Eq)]
pub struct LatchesChanged {
    /// The latched modifiers' scan codes.
    pub held: Vec<Scan>,
}

impl Event for LatchesChanged {
    const NAME: &'static str = "latches-changed";
    const CAPABILITY: Option<Capability> = Some(Capability::InjectInput);
}

/// Shows only how many are held, as `kx-mod-keyboard`'s `Latches` hides which: a run of clicks would show what was
/// typed.
impl fmt::Debug for LatchesChanged {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("LatchesChanged")
            .field("count", &self.held.len())
            .finish()
    }
}

/// The mouse module says a click used the latched modifiers.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct LatchesUsed;

impl Event for LatchesUsed {
    const NAME: &'static str = "latches-used";
    const CAPABILITY: Option<Capability> = Some(Capability::InjectInput);
}

/// Asks the keyboard to send `LatchesChanged` now.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct LatchesAsk;

impl Event for LatchesAsk {
    const NAME: &'static str = "latches-ask";
    const CAPABILITY: Option<Capability> = Some(Capability::InjectInput);
}

#[cfg(test)]
mod tests;
