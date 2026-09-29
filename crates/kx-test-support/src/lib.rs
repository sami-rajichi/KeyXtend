//! Helpers for tests and test tools: golden files, temp folders, file locks, percentiles and frame stats.
//! Shipped code never depends on this crate.

#![forbid(unsafe_code)]

pub mod golden;
#[cfg(windows)]
pub mod lock;
pub mod pct;
pub mod ringstats;
pub mod tempdir;
