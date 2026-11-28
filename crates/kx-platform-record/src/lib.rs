//! Worker-only screen recording, re-timed copies and the system's file dialogs (P12): the `record` job's recorder and
//! speed change, over the system's capture, sound devices and encoders, and the `pick` job's Save and folder dialogs.
//! The uiAccess app never depends on this crate: tidy D4 keeps it out of `keyxtend`'s tree.

// Backstop to the workspace lint: each unsafe block in the adapter still needs its own scoped allow.
#![deny(unsafe_code)]

#[cfg(any(test, feature = "fake"))]
pub mod fake;
#[cfg(not(windows))]
mod none;
pub mod pick;
pub mod record;
pub mod speed;
pub mod table;
#[cfg(windows)]
mod win;

/// The system's studio for the `record` job: its capture, sound devices and encoders.
#[cfg(not(windows))]
pub use none::NoStudio as LiveStudio;
/// The system's studio for the `record` job: its capture, sound devices and encoders.
#[cfg(windows)]
pub use win::WinStudio as LiveStudio;
