//! Shared core of the P1 spike faces. Throwaway: deleted at the end of P1.

pub mod config;
pub mod inject;
pub mod layout;
pub mod uiaccess;
pub mod window;

/// Scan-code bit that marks an extended key (`0xE0xx`).
pub const EXTENDED: u32 = 0xE000;
