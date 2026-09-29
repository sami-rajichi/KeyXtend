//! The platform ports: what an OS adapter hands the kernel and the UI, and the rule that picks the data folder.
//! The UI ports live in `ui`; later phases add more.

#![forbid(unsafe_code)]

mod dirs;
pub mod ui;

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
