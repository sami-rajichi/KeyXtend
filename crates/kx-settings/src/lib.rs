//! The settings file and the `Store`: safe save, recovery of a damaged file, and per-module sections
//! with versions, migrations, per-value repair and changes-only saving.

#![forbid(unsafe_code)]

mod file;
mod merge;
mod names;
mod section;
mod store;

pub use file::{Loaded, Outcome, SaveError, load, peek, save};
pub use names::{Files, MAX_FILE_BYTES, MAX_KEY_CHARS, MAX_NAMED_KEYS, VERSION_KEY};
pub use store::{LoadReport, Store, StoreError, Unsaved};
