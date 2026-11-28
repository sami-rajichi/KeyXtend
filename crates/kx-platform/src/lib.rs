//! The platform ports: what an OS adapter hands the kernel and the UI, the rule that picks the data folder, and the
//! OS-neutral halves of the word packs, the field and selection watches, the worker host and the hidden-run probe. The
//! UI ports live in `ui`; later phases add more.

#![forbid(unsafe_code)]

pub mod boxes;
pub mod cloud;
mod dirs;
mod ear;
pub mod field;
pub mod mirror;
pub mod models;
pub mod packs;
pub mod pictures;
pub mod selection;
pub mod snip;
pub mod speech;
pub mod ui;
pub mod worker;

pub use dirs::{
    APP_DIR_NAME, AppDirs, CLIP_DIR, DataDir, PORTABLE_DIR, SEALED_DIR, SNIP_DIR, UNSAVED_DIR,
    VAULT_DIR, VOICE_DIR, pick_data_dir, snip_dirs, unsaved_dir,
};

use kx_module_api::Clock;
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};
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

/// The lock of `m`, whose state stays whole even if a holder panicked.
pub(crate) fn lock<T>(m: &Mutex<T>) -> MutexGuard<'_, T> {
    m.lock().unwrap_or_else(PoisonError::into_inner)
}
