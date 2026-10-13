//! The pointer hook: a module's engine runs on the platform's hook thread and decides each mouse event there
//! (ADR-0027).

use super::{Point, Scan};
use crate::{Capability, Mono, ServiceId, ServiceKey};
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

/// One event the hook saw, in physical pixels; it never leaves the hook thread.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
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

/// Input the host sends, in order, after the callback returns.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
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

/// A module's decisions, run by the platform on its hook thread.
pub trait PointerEngine: Send {
    /// Decides `event` inside the hook callback: pushes input to send into the lent `acts` and returns true to drop
    /// the event. No blocking, no allocation beyond `acts`, no logging and no I/O.
    fn on(&mut self, event: PointerEvent, now: Mono, acts: &mut Vec<PointerAct>) -> bool;

    /// Runs after the callback returned and `acts` went out, when woken, and when the deadline passed. It may apply
    /// queued commands, fire a due hold and hand notes to the module; brief channel sends are fine here.
    fn settle(&mut self, now: Mono, acts: &mut Vec<PointerAct>);

    /// When the host must call `settle` next; `None` when nothing is due.
    fn deadline(&self) -> Option<Mono>;

    /// Lets go of everything held, such as a latched button or pressed modifiers; the host sends `acts` before it
    /// stops.
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
    /// How long the hooks may miss input before they are put back, in milliseconds.
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

/// A running hook. Dropping it is the only way to stop, and the drop returns once the host has called `release`, sent
/// that input and removed the hooks, or after a bounded wait.
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
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn a_refusal_names_the_os_code() {
        let text = HookError::Refused(1428).to_string();
        assert!(text.contains("1428"), "{text}");
    }

    #[test]
    fn a_passed_event_is_sent_again_as_its_own_act_and_esc_has_none() {
        let at = Point { x: -3, y: 9 };
        let down = PointerEvent::Down {
            button: Button::Right,
            at,
            ours: true,
            dpi: 144,
        };
        assert_eq!(down.act(), Some(PointerAct::Down(Button::Right, at)));
        let up = PointerEvent::Up {
            button: Button::Middle,
            at,
        };
        assert_eq!(up.act(), Some(PointerAct::Up(Button::Middle, at)));
        assert_eq!(PointerEvent::Move { at }.act(), Some(PointerAct::Move(at)));
        assert_eq!(PointerEvent::Esc.act(), None, "Esc is never dropped");
    }

    #[test]
    fn each_error_says_something_else() {
        let errors = [HookError::Busy, HookError::Refused(0), HookError::Thread];
        let texts: HashSet<_> = errors.iter().map(ToString::to_string).collect();
        assert_eq!(texts.len(), errors.len());
    }
}
