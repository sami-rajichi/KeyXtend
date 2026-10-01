//! The app's entry point on Windows: the command line, the one running copy, the log, the kernel with every module,
//! then Qt's loop on this thread while the kernel drains the UI's requests on its own.
// The uiAccess build (`kx_ui_access`, from `build.rs`) opens no console window, which would take the foreground.
#![cfg_attr(kx_ui_access, windows_subsystem = "windows")]
#![forbid(unsafe_code)]

#[cfg(windows)]
mod qt;
#[cfg(windows)]
mod run;

use std::process::ExitCode;

/// What the app says on any other system.
#[cfg(not(windows))]
const WINDOWS_ONLY: &str = "KeyXtend runs only on Windows for now.";

#[cfg(windows)]
fn main() -> ExitCode {
    run::main()
}

#[cfg(not(windows))]
fn main() -> ExitCode {
    eprintln!("{WINDOWS_ONLY}");
    ExitCode::FAILURE
}
