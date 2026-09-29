//! The platform port: what an OS adapter hands the kernel, and the rule that picks the data folder.
//! Later phases add more ports here.

#![forbid(unsafe_code)]

mod dirs;

pub use dirs::{AppDirs, DataDir, PORTABLE_DIR, pick_data_dir};

use kx_module_api::Clock;
use std::sync::Arc;

/// What an OS adapter hands the kernel. Later phases add more ports.
pub trait Platform: Send + Sync {
    /// The monotonic clock.
    fn clock(&self) -> Arc<dyn Clock>;

    /// The program and per-user folders.
    fn dirs(&self) -> &AppDirs;
}
