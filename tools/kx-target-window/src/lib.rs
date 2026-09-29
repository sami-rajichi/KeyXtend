//! The CI test window's shared part: its log format and its settings.
//!
//! The window itself (`main.rs`) is Windows-only; this library builds everywhere.
#![forbid(unsafe_code)]

pub mod config;
pub mod log;
