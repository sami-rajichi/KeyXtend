//! The CI test window: a text box that logs each character it receives, with a microsecond timestamp.
//! It logs focus losses and mouse presses too; the log format is in the library.
#![deny(unsafe_code)]
#![cfg_attr(windows, windows_subsystem = "windows")]

#[cfg(windows)]
mod procs;
#[cfg(windows)]
mod window;

use std::process::ExitCode;

/// What the tool says on any other system.
#[cfg(not(windows))]
const WINDOWS_ONLY: &str = "kx-target-window runs only on Windows.";

#[cfg(windows)]
fn main() -> ExitCode {
    match window::run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("{e}");
            ExitCode::FAILURE
        }
    }
}

#[cfg(not(windows))]
fn main() -> ExitCode {
    eprintln!("{WINDOWS_ONLY}");
    ExitCode::FAILURE
}
