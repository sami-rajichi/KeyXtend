//! One graphics device for every window (ADR-0021), over the app's QML and a hub with a fake face: the shared device,
//! the release and give-back after a look change, the soft shadows' adopter, losses, failed windows, fonts and the
//! fallback, each step told in `rig::run`. Qt wants its app object on the main thread, so this binary runs without the
//! test harness, and its probes exist only in debug builds, so release builds check nothing.

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
