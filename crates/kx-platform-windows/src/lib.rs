//! The Windows adapter: the only product crate that calls Win32.
//! OS code is Windows-only; its pure maths builds and tests on every system.

// Backstop to the workspace lint: each unsafe block still needs its own scoped allow.
#![deny(unsafe_code)]

pub mod clock;
#[cfg(windows)]
pub mod com;
pub mod dirs;
pub mod front;
pub mod inject;
pub mod keys;
pub mod langinfo;
pub mod layouts;
#[cfg(windows)]
mod platform;
pub mod strokes;
#[cfg(windows)]
pub mod uiaccess;

#[cfg(windows)]
pub use clock::QpcClock;
pub use clock::ticks_to_us;
#[cfg(windows)]
pub use inject::WinInjector;
#[cfg(windows)]
pub use layouts::WinLayouts;
#[cfg(windows)]
pub use platform::WindowsPlatform;

use std::io;
use thiserror::Error;

/// Why Windows refused the adapter something. No variant names a path, so no user name reaches a log.
#[derive(Debug, Error)]
pub enum PlatformError {
    /// The performance counter gave no frequency.
    #[error("the performance counter gave no frequency: {0}")]
    Clock(io::Error),
    /// Windows did not name the local app-data folder.
    #[error("Windows did not name the local app-data folder: {0}")]
    LocalData(io::Error),
    /// The running program's path is unknown.
    #[error("the running program's path is unknown: {0}")]
    ExePath(io::Error),
    /// The running program's path names no folder.
    #[error("the running program's path names no folder")]
    NoExeDir,
    /// COM did not start on this thread.
    #[error("COM did not start on this thread: {0}")]
    Com(io::Error),
}
