//! The session port (ADR-0042 decision 9): the platform tells when the session locks, unlocks, sleeps or wakes, so a
//! dictation stops and wipes its sound. It carries no content, so it needs no capability.

use crate::{Capability, ServiceId, ServiceKey};
use thiserror::Error;

listed! {
    /// A change of the user's session, as the platform tells it.
    pub enum SessionNote ("Every note.") {
        /// The session locked.
        Locked,
        /// The session unlocked.
        Unlocked,
        /// The computer is going to sleep.
        Sleeping,
        /// The computer woke from sleep.
        Woke,
    }
}

impl SessionNote {
    /// True when the user is away, after a lock or before a sleep, so no microphone may stay open.
    #[must_use]
    pub fn away(self) -> bool {
        matches!(self, Self::Locked | Self::Sleeping)
    }
}

/// Why the session could not be watched.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Error)]
pub enum SessionError {
    /// The platform cannot tell session changes.
    #[error("the system cannot tell session changes")]
    Unavailable,
    /// The OS refused.
    #[error("the system refused to tell session changes")]
    Refused,
}

/// A running watch on the platform's side. Dropping it stops the watch.
pub trait SessionRun: Send {}

/// Watches the user's session; the platform provides it.
pub trait SessionWatch: Send + Sync {
    /// Starts a watch; `tell` hears each lock, unlock, sleep and wake, on any thread.
    ///
    /// # Errors
    /// `Unavailable` when the platform cannot watch, `Refused` when the OS refuses.
    fn watch(
        &self,
        tell: Box<dyn Fn(SessionNote) + Send>,
    ) -> Result<Box<dyn SessionRun>, SessionError>;
}

/// The session watch service; it needs no capability, as it carries no content.
pub struct SessionWatchKey;

impl ServiceKey for SessionWatchKey {
    type Api = dyn SessionWatch;
    const ID: ServiceId = ServiceId::new("session-watch");
    const CAPABILITY: Option<Capability> = None;
}

#[cfg(test)]
mod tests;
