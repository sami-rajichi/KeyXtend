//! The settings file: its names in the data folder, safe save, and recovery of a damaged file.
//! Sections and the `Store` are built on top of this whole-file layer.

#![forbid(unsafe_code)]

mod file;
mod names;

pub use file::{Loaded, Outcome, SaveError, load, save};
pub use names::{Files, MAX_FILE_BYTES, VERSION_KEY};
