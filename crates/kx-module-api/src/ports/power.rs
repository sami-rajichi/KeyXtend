//! The Power key: the dialog port, which only `Power` reaches, and the bus events, which need no capability as the
//! keyboard publishes and hears them.

use crate::{Capability, Event, ServiceId, ServiceKey};
use thiserror::Error;

/// Why the system's power dialog did not open.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Error)]
pub enum PowerError {
    /// No system shell runs to show it.
    #[error("no system shell can show the power dialog")]
    Unavailable,
    /// The OS refused.
    #[error("the system refused to show the power dialog")]
    Failed,
}

/// Opens the system's own power dialog; it never shuts down by itself.
pub trait PowerDialog: Send + Sync {
    /// Shows the dialog, where the user picks what to do.
    ///
    /// # Errors
    /// `Unavailable` with no system shell, `Failed` when the OS refuses.
    fn open(&self) -> Result<(), PowerError>;
}

/// The power dialog service; it needs `Power`.
pub struct PowerDialogKey;

impl ServiceKey for PowerDialogKey {
    type Api = dyn PowerDialog;
    const ID: ServiceId = ServiceId::new("power-dialog");
    const CAPABILITY: Option<Capability> = Some(Capability::Power);
}

/// The Power key was clicked: the power module opens the dialog.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct PowerAsked;

impl Event for PowerAsked {
    const NAME: &'static str = "power-asked";
}

/// The power module's state, sent on its start, on `PowerAsk` and on its stop.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct PowerChanged {
    /// The power module runs; false when it stops.
    pub live: bool,
}

impl PowerChanged {
    /// The power module not running: before it speaks, and after it stops or fails.
    pub const OFF: Self = Self { live: false };
}

impl Event for PowerChanged {
    const NAME: &'static str = "power-changed";
}

/// Asks the power module to send `PowerChanged` now, for a module or the UI that starts later.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct PowerAsk;

impl Event for PowerAsk {
    const NAME: &'static str = "power-ask";
}

#[cfg(test)]
mod tests;
