//! The Windows adapter: the only product crate that calls Win32.
//! OS code is Windows-only; its pure maths builds and tests on every system.

// Backstop to the workspace lint: each unsafe block still needs its own scoped allow.
#![deny(unsafe_code)]

pub mod backdrop;
pub mod calendar;
pub mod clip;
pub mod clock;
#[cfg(windows)]
pub mod com;
pub mod dirs;
pub mod disk;
#[cfg(windows)]
pub mod dll;
pub mod field;
pub mod front;
pub mod gpu;
#[cfg(windows)]
pub mod guard_key;
#[cfg(windows)]
mod hidden;
pub mod hook;
pub mod inject;
pub mod instance;
pub mod keys;
pub mod langinfo;
pub mod layouts;
#[cfg(windows)]
pub mod memory;
#[cfg(windows)]
pub mod packs;
#[cfg(windows)]
mod platform;
pub mod power;
pub mod reveal;
pub mod screen;
pub mod scroll;
pub mod shortcut;
pub mod sound;
pub mod strokes;
pub mod sysui;
pub mod tray;
#[cfg(windows)]
pub mod uia;
#[cfg(windows)]
pub mod uiaccess;
pub mod watch;
pub mod window;
#[cfg(windows)]
pub mod worker;

#[cfg(windows)]
pub use backdrop::WinBackdrop;
#[cfg(windows)]
pub use calendar::WinCalendar;
#[cfg(windows)]
pub use clip::{WinClipboard, WinClipboardWriter};
#[cfg(windows)]
pub use clock::QpcClock;
pub use clock::ticks_to_us;
#[cfg(windows)]
pub use disk::WinDiskRoom;
#[cfg(windows)]
pub use field::WinFieldWatch;
#[cfg(windows)]
pub use guard_key::WinKeyGuard;
#[cfg(windows)]
pub use hook::{WinPointerHook, release_now};
#[cfg(windows)]
pub use inject::WinInjector;
#[cfg(windows)]
pub use instance::WinInstance;
#[cfg(windows)]
pub use layouts::WinLayouts;
#[cfg(windows)]
pub use memory::WinMemory;
#[cfg(windows)]
pub use packs::{WinPackDisk, WinWordPacks};
#[cfg(windows)]
pub use platform::WindowsPlatform;
#[cfg(windows)]
pub use power::WinPower;
#[cfg(windows)]
pub use reveal::WinReveal;
#[cfg(windows)]
pub use screen::WinScreens;
#[cfg(windows)]
pub use scroll::WinScroll;
#[cfg(windows)]
pub use shortcut::WinShortcuts;
#[cfg(windows)]
pub use sound::WinSpeaker;
#[cfg(windows)]
pub use sysui::WinLook;
#[cfg(windows)]
pub use tray::WinTray;
#[cfg(windows)]
pub use window::WinGuard;
#[cfg(windows)]
pub use worker::{WinChild, WinSpawner};

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

/// A refused UI request from a call that sets no last error, with Windows' generic failure code.
#[cfg(windows)]
fn failed() -> kx_platform::ui::UiError {
    kx_platform::ui::UiError::Refused(windows::Win32::Foundation::E_FAIL.0)
}

/// Locks `m`, taking the value even when a panicking thread poisoned it.
fn lock<T>(m: &std::sync::Mutex<T>) -> std::sync::MutexGuard<'_, T> {
    m.lock().unwrap_or_else(std::sync::PoisonError::into_inner)
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
    /// Windows did not limit where libraries load from.
    #[error("Windows did not limit where libraries load from: {0}")]
    DllDirs(io::Error),
    /// Windows did not say whether the process has uiAccess.
    #[error("Windows did not say whether the process has uiAccess: {0}")]
    UiAccess(io::Error),
}
