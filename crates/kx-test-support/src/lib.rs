//! Helpers for tests and test tools: golden files, temp folders, file locks, percentiles and a fake kernel host for
//! module tests.
//! Shipped code never depends on this crate.

#![forbid(unsafe_code)]

pub mod golden;
pub mod host;
#[cfg(windows)]
pub mod lock;
pub mod pct;
pub mod tempdir;
