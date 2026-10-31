//! The platform ports: what an OS adapter hands the kernel and the UI, the rule that picks the data folder, and the
//! OS-neutral halves of the word packs, the field watch and the worker host. The UI ports live in `ui`; later phases add
//! more.

#![forbid(unsafe_code)]

mod dirs;
pub mod field;
pub mod packs;
pub mod pictures;
pub mod ui;

pub use dirs::{APP_DIR_NAME, AppDirs, CLIP_DIR, DataDir, PORTABLE_DIR, SEALED_DIR, pick_data_dir};

use kx_module_api::Clock;
use std::sync::Arc;
use std::time::{Duration, Instant};

/// What an OS adapter hands the kernel. Later phases add more ports.
pub trait Platform: Send + Sync {
    /// The monotonic clock.
    fn clock(&self) -> Arc<dyn Clock>;

    /// The program and per-user folders.
    fn dirs(&self) -> &AppDirs;
}

/// `cap` from now, or now when the clock cannot go that far.
#[must_use]
pub fn deadline(cap: Duration) -> Instant {
    let now = Instant::now();
    now.checked_add(cap).unwrap_or(now)
}
