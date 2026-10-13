//! The pointer hook host: it runs a module's engine inside low-level mouse and keyboard hooks, sends the engine's input
//! after each callback and keeps that input in order (ADR-0027). The rules are pure and tested everywhere; `win` calls
//! Windows.
#![cfg_attr(
    not(windows),
    allow(
        dead_code,
        reason = "Only the Windows hook runs this logic; tests cover it everywhere."
    )
)]

mod accept;
mod batch;
mod health;
mod host;
mod queue;
mod stats;
#[cfg(test)]
mod tests;
mod timing;
#[cfg(windows)]
mod win;

use kx_module_api::Mono;
use kx_module_api::ports::{Point, PointerAct};

#[cfg(windows)]
pub use win::{WinPointerHook, release_now};

/// Whose input the engine acts on.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Source {
    /// The hand on the mouse: physical input only, as the product runs.
    Physical,
    /// The gate tool's input tagged `SIM_TAG`, in a test build only; the hand's input then passes untouched.
    Simulated,
}

/// The marks Windows puts on a hook event.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub(crate) struct Mark {
    /// A program sent it (`LLMHF_INJECTED`, `LLKHF_INJECTED`).
    pub injected: bool,
    /// Its `dwExtraInfo`, where our tags ride.
    pub extra: usize,
    /// When it happened, by Windows' tick count in ms.
    pub time: u32,
}

/// The OS calls the host makes: Win32 on Windows, a recorder in tests.
pub(crate) trait Os {
    /// The platform clock.
    fn now(&self) -> Mono;

    /// Windows' tick count in ms, the time base of hook events.
    fn tick(&self) -> u32;

    /// The tick of the last input Windows saw; `None` when it will not say.
    fn last_input(&self) -> Option<u32>;

    /// True when the window at `at` belongs to this process.
    fn ours(&self, at: Point) -> bool;

    /// The effective DPI of the screen at `at`, or the default.
    fn dpi(&self, at: Point) -> u32;

    /// Asks the hook thread for a turn; false when the request was refused.
    fn wake(&self) -> bool;

    /// Sends `acts` in one batch tagged `REPLAY` and returns how many the OS took.
    fn send(&self, acts: &[PointerAct]) -> usize;

    /// Sets the one deadline timer to fire after `ms`, or stops it.
    fn arm(&self, ms: Option<u32>);

    /// Installs both hooks, or neither; the OS error code when refused.
    ///
    /// # Errors
    /// The code Windows gave.
    fn hook(&self) -> Result<(), i32>;

    /// Removes both hooks; removing none is fine.
    fn unhook(&self);
}
