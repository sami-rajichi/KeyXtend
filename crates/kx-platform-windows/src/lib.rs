//! The Windows adapter: the only product crate that calls Win32.
//! OS code is Windows-only; its pure maths builds and tests on every system.

// Backstop to the workspace lint: each unsafe block still needs its own scoped allow.
#![deny(unsafe_code)]

pub mod backdrop;
pub mod clock;
#[cfg(windows)]
pub mod com;
pub mod dirs;
pub mod front;
#[cfg(windows)]
mod hidden;
pub mod inject;
pub mod instance;
pub mod keys;
pub mod langinfo;
pub mod layouts;
#[cfg(windows)]
mod platform;
pub mod screen;
pub mod strokes;
pub mod sysui;
#[cfg(windows)]
pub mod uiaccess;
pub mod watch;
pub mod window;

#[cfg(windows)]
pub use backdrop::WinBackdrop;
#[cfg(windows)]
pub use clock::QpcClock;
pub use clock::ticks_to_us;
#[cfg(windows)]
pub use inject::WinInjector;
#[cfg(windows)]
pub use instance::WinInstance;
#[cfg(windows)]
pub use layouts::WinLayouts;
#[cfg(windows)]
pub use platform::WindowsPlatform;
#[cfg(windows)]
pub use screen::WinScreens;
#[cfg(windows)]
pub use sysui::WinLook;
#[cfg(windows)]
pub use window::WinGuard;

use std::io;
use thiserror::Error;

/// A Windows error as a refused UI request, keeping its code.
#[cfg(windows)]
fn refused(e: &windows::core::Error) -> kx_platform::ui::UiError {
    kx_platform::ui::UiError::Refused(e.code().0)
}

/// The calling thread's last Windows error as a refused UI request.
#[cfg(windows)]
fn last_refused() -> kx_platform::ui::UiError {
    refused(&windows::core::Error::from_thread())
}

/// `text` as UTF-16 with an ending null, for Windows names.
#[cfg(windows)]
fn wide(text: &str) -> Vec<u16> {
    text.encode_utf16().chain([0]).collect()
}

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
