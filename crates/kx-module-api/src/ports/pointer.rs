//! The pointer hook: a module's engine runs on the platform's hook thread and decides each mouse event there
//! (ADR-0027).

use super::{Point, Scan, hide_at};
use crate::{Capability, Mono, Redacted, ServiceId, ServiceKey};
use std::fmt;
use thiserror::Error;

/// A mouse button.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum Button {
    /// The left button.
    Left,
    /// The right button.
    Right,
    /// The middle button.
    Middle,
}

/// One event the hook saw, in physical pixels; it never leaves the hook thread, and its `Debug` hides every point.
#[derive(Copy, Clone, PartialEq, Eq)]
pub enum PointerEvent {
    /// A button went down.
    Down {
        /// The button.
        button: Button,
        /// Where it went down.
        at: Point,
        /// The press landed on one of our own windows.
        ours: bool,
        /// The DPI of the screen under the press; the engine scales its still radius by it.
        dpi: u32,
    },
    /// A button went up.
    Up {
        /// The button.
        button: Button,
        /// Where it went up.
        at: Point,
    },
    /// The pointer moved.
    Move {
        /// Where it is now.
        at: Point,
    },
    /// The Esc key went down.
    Esc,
}

impl PointerEvent {
    /// The act that sends this event again behind our own input; Esc has none, as it is never dropped.
    #[must_use]
    pub const fn act(self) -> Option<PointerAct> {
        match self {
            Self::Down { button, at, .. } => Some(PointerAct::Down(button, at)),
            Self::Up { button, at } => Some(PointerAct::Up(button, at)),
            Self::Move { at } => Some(PointerAct::Move(at)),
            Self::Esc => None,
        }
    }
}

/// Input the host sends, in order, after the callback returns; its `Debug` hides every point and key.
#[derive(Copy, Clone, PartialEq, Eq)]
pub enum PointerAct {
    /// Moves the pointer to the point.
    Move(Point),
    /// Presses the button at the point.
    Down(Button, Point),
    /// Lets go of the button at the point.
    Up(Button, Point),
    /// Presses a key, such as a latched modifier.
    KeyDown(Scan),
    /// Lets go of a key.
    KeyUp(Scan),
}

/// Shows the button and flags only: a click's point can show what was clicked (an on-screen PIN pad).
impl fmt::Debug for PointerEvent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Self::Down {
                button,
                at,
                ours,
                dpi,
            } => f
                .debug_struct("Down")
                .field("button", &button)
                .field("at", &Redacted::new(at))
                .field("ours", &ours)
                .field("dpi", &dpi)
                .finish(),
            Self::Up { button, at } => f
                .debug_struct("Up")
                .field("button", &button)
                .field("at", &Redacted::new(at))
                .finish(),
            Self::Move { at } => hide_at(f, "Move", at),
            Self::Esc => f.write_str("Esc"),
        }
    }
}

/// Shows the button only: a point can show what was clicked, and a run of keys what was typed.
impl fmt::Debug for PointerAct {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Self::Move(at) => hide(f, "Move", None, at),
            Self::Down(button, at) => hide(f, "Down", Some(button), at),
            Self::Up(button, at) => hide(f, "Up", Some(button), at),
            Self::KeyDown(key) => hide(f, "KeyDown", None, key),
            Self::KeyUp(key) => hide(f, "KeyUp", None, key),
        }
    }
}

/// Writes the act `name`: its `button`, if any, then `value` hidden.
fn hide<T>(
    f: &mut fmt::Formatter<'_>,
    name: &str,
    button: Option<Button>,
    value: T,
) -> fmt::Result {
    let mut tuple = f.debug_tuple(name);
    if let Some(button) = button {
        tuple.field(&button);
    }
    tuple.field(&Redacted::new(value)).finish()
}

/// A module's decisions, run by the platform on its hook thread. After a panic the host calls `release` once, or not at
/// all when `release` itself panicked, then drops the engine while the run goes on, so its `Drop` tells the module.
pub trait PointerEngine: Send {
    /// Decides `event` inside the hook callback: pushes input to send into the lent `acts` and returns true to drop
    /// the event. No blocking, no allocation beyond `acts`, no logging and no I/O.
    fn on(&mut self, event: PointerEvent, now: Mono, acts: &mut Vec<PointerAct>) -> bool;

    /// Runs after the callback returned and `acts` went out, when woken, and when the deadline passed. It may apply
    /// queued commands, fire a due hold and hand notes to the module; brief channel sends are fine here.
    fn settle(&mut self, now: Mono, acts: &mut Vec<PointerAct>);

    /// When the host must call `settle` next; `None` when nothing is due.
    fn deadline(&self) -> Option<Mono>;

    /// True while `settle` has notes for the module, so the host asks for a turn. The callback asks it after `on`, so it
    /// never blocks or allocates.
    fn has_notes(&self) -> bool;

    /// Lets go of everything held, such as a carried button or pressed modifiers; the host sends `acts` at once. It may
    /// come while the run goes on, after a health reinstall or the panic hook's let-go, and the engine then works on
    /// from a clean state.
    fn release(&mut self, acts: &mut Vec<PointerAct>);

    /// The most acts one `on`, `settle` or `release` call may push; a host reserves the lent buffer with it once, so
    /// pushes never allocate.
    fn max_acts(&self) -> usize;
}

/// The most keys an engine may hold down at once; a host keeps room for this many in its record of held input.
pub const MAX_HELD_KEYS: usize = 16;

/// How the host runs; the module fills it from its settings.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct HookConfig {
    /// The health check's period, in milliseconds.
    pub health_ms: u32,
    /// How far Windows' last input may run ahead of the hooks before a check counts them as lagging, in milliseconds;
    /// two lagging checks in a row put them back.
    pub health_slack_ms: u32,
    /// How long input waits behind our own before it goes on, in milliseconds.
    pub echo_ms: u32,
}

/// A running hook's numbers, never a point or a key.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub struct HookStats {
    /// Events the hooks gave the engine.
    pub events: u64,
    /// The 99th percentile of the callback's time, in microseconds.
    pub p99_us: u32,
    /// The longest callback, in microseconds.
    pub max_us: u32,
    /// The longest time an event waited before the hook saw it, in milliseconds.
    pub max_wait_ms: u32,
    /// How many times the health check put the hooks back.
    pub reinstalls: u32,
    /// How many times the OS refused our input.
    pub send_errors: u32,
    /// How many times our input did not come back through the hook in time.
    pub lost_echoes: u32,
    /// The user's events heard inside our own send, which passed without the host.
    pub passed_inside: u32,
}

/// Why the pointer hook did not start.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Error)]
pub enum HookError {
    /// A hook already runs.
    #[error("a pointer hook already runs")]
    Busy,
    /// The OS refused to install the hook; it carries the OS error code.
    #[error("the system refused the pointer hook (code {0})")]
    Refused(i32),
    /// The hook thread could not start or did not answer.
    #[error("the pointer hook thread did not start or answer")]
    Thread,
}

/// A running hook. Dropping it is the only way to stop, and the drop returns once the host has called `release` (never
/// again for an engine it gave up), sent that input and removed the hooks, or after a bounded wait.
pub trait HookRun: Send + Sync {
    /// Asks the host to call `settle` soon; never blocks.
    fn wake(&self);

    /// The hook's numbers so far.
    fn stats(&self) -> HookStats;
}

/// Runs a module's engine on a system-wide pointer hook; the platform provides it.
pub trait PointerHook: Send + Sync {
    /// Installs the hooks and runs `engine` on the hook thread, as `config` says.
    ///
    /// # Errors
    /// `Busy` when a hook already runs, `Refused` when the OS refuses it, `Thread` when its thread fails.
    fn start(
        &self,
        engine: Box<dyn PointerEngine>,
        config: HookConfig,
    ) -> Result<Box<dyn HookRun>, HookError>;
}

/// The pointer hook service; it needs `PointerHook`.
pub struct PointerHookKey;

impl ServiceKey for PointerHookKey {
    type Api = dyn PointerHook;
    const ID: ServiceId = ServiceId::new("pointer-hook");
    const CAPABILITY: Option<Capability> = Some(Capability::PointerHook);
}

#[cfg(test)]
mod tests;
