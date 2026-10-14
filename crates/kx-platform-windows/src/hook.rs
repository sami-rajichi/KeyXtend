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
mod echo;
mod health;
mod held;
mod host;
mod inbox;
mod queue;
mod stats;
#[cfg(test)]
mod tests;
mod timing;
#[cfg(windows)]
mod win;

use kx_module_api::Mono;
use kx_module_api::ports::{Point, PointerAct};

use inbox::Inbox;

#[cfg(windows)]
pub use win::{WinPointerHook, release_now};

/// Whose input the engine acts on.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Source {
    /// The hand on the mouse: physical input only, as the product runs.
    Physical,
    /// The gate tool's input tagged `SIM_TAG`; the hand's input then passes untouched. It exists only with the
    /// `sim-input` feature, and in this crate's unit tests.
    #[cfg(any(test, feature = "sim-input"))]
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

    /// True while the input desktop is not ours, as on the secure desktop or the lock screen; the hooks see no input
    /// there.
    fn away(&self) -> bool;

    /// True when the window at `at` belongs to this process.
    fn ours(&self, at: Point) -> bool;

    /// The effective DPI of the screen at `at`, or the default.
    fn dpi(&self, at: Point) -> u32;

    /// Asks the hook thread for a turn; false when the request was refused.
    fn wake(&self) -> bool;

    /// Sends `acts` in one batch, each tagged `REPLAY` with its own id counted from `first`, wrapping, and returns how
    /// many the OS took. Echoes our hooks hear during the send go to `inbox`, as the host is busy then.
    fn send(&self, acts: &[PointerAct], first: u16, inbox: &mut Inbox) -> usize;

    /// Sets the one deadline timer to fire every `ms`, or stops it; false when Windows refused the timer.
    fn arm(&self, ms: Option<u32>) -> bool;

    /// Installs both hooks, or neither; the OS error code when refused.
    ///
    /// # Errors
    /// The code Windows gave.
    fn hook(&self) -> Result<(), i32>;

    /// Removes both hooks; removing none is fine.
    fn unhook(&self);
}
