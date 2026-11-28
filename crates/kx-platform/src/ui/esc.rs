//! The Esc watch port (ADR-0059): Esc presses we did not send, heard system-wide only while Snip's frozen screens show,
//! so a user with a physical keyboard or an assistive tool always has a way out of them.

use super::{UiError, WatchGuard};

/// Called once per press, on the adapter's own thread.
pub type OnEsc = Box<dyn Fn() + Send + Sync>;

/// Hears the Esc key we did not send, which still reaches the app in front.
pub trait EscWatch: Send + Sync {
    /// Calls `on` once per press of an Esc we did not send until the guard drops: the hand's, or another program's, as
    /// Voice Access or another on-screen keyboard sends it; never our own input, a held key's repeats or another key.
    /// `on` runs on the adapter's thread, which may hold every key system-wide until it returns, so it stays short and
    /// never blocks on the thread that drops the guard. Where the system lets no app hear keys, as on Wayland, none comes.
    ///
    /// # Errors
    /// `Refused` when the system will not let it listen; the bar's Close and our keyboard's Esc stay the way out.
    fn watch(&self, on: OnEsc) -> Result<WatchGuard, UiError>;
}
