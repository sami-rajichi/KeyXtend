//! One graphics device for every window (ADR-0021): the app's QML over a hub with a fake face, where the keyboard,
//! its shadow and the language list a Loader makes later all draw on the shared device, on the GUI thread, made on
//! Qt's default adapter when the pinned one is gone; a lost device, or a window that cannot draw, brings the windows
//! back over the same hub, as the user left them, with Qt's loop running between tries, and such a window ends the app
//! once the tries are spent; after a frame off the GUI thread, no rebuild shares a device; Qt's font list takes only
//! the theme's bundled fonts; without a shared device, each window draws on its own. Qt wants its app object on the
//! main thread, so this binary runs without the test harness. It reads probes and marks devices lost, which only debug
//! builds have, so a release build of it checks nothing.

#![allow(
    clippy::expect_used,
    reason = "the shared board helper runs only in tests, where a failed setup should fail the test"
)]

#[cfg(debug_assertions)]
#[path = "../shared/board.rs"]
mod board;
#[cfg(debug_assertions)]
mod checks;
#[cfg(debug_assertions)]
mod face;
#[cfg(debug_assertions)]
mod left;
#[cfg(debug_assertions)]
mod logs;
#[cfg(debug_assertions)]
mod rebuild;
#[cfg(debug_assertions)]
mod rig;

use std::process::ExitCode;

#[cfg(debug_assertions)]
fn main() -> ExitCode {
    match rig::run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(why) => {
            eprintln!("{why}");
            ExitCode::FAILURE
        }
    }
}

#[cfg(not(debug_assertions))]
fn main() -> ExitCode {
    eprintln!("the device test needs a debug build's probes, so it checks nothing here");
    ExitCode::SUCCESS
}
