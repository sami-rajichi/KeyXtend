//! The contract feature modules build against: time and redaction so far.
//! Modules depend on this crate and on nothing else in the workspace.

#![forbid(unsafe_code)]

mod clock;
mod redacted;

pub use clock::{Clock, Mono, US_PER_MS, US_PER_S};
pub use redacted::{REDACTED_MARKER, Redacted};
