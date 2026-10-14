//! The CI test window's shared part: its log format, its settings and its `--layout` switch.
//!
//! The window itself (`main.rs`) is Windows-only; this library builds everywhere.
#![forbid(unsafe_code)]

pub mod config;
pub mod hang;
pub mod layout;
pub mod log;
