//! Helpers for tests and test tools: golden files, temp folders, percentiles and frame stats.
//! Shipped code never depends on this crate.

#![forbid(unsafe_code)]

pub mod golden;
pub mod pct;
pub mod ringstats;
pub mod tempdir;
