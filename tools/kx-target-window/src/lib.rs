//! The CI test window's shared part: its log format, its settings, its `--layout` switch, its fill text and the hang
//! message a gate sends it.
//!
//! The window itself (`main.rs`) is Windows-only; this library builds everywhere.
#![forbid(unsafe_code)]

pub mod config;
pub mod fill;
pub mod hang;
pub mod layout;
pub mod log;
