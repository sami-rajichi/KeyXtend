//! Builds the `cargo` command that xtask's subcommands spawn.

use std::process::Command;

/// The env var cargo sets to its own path for every process it spawns.
const CARGO_ENV: &str = "CARGO";
/// The program to run when [`CARGO_ENV`] is unset.
const CARGO: &str = "cargo";

/// The `cargo` to run: [`CARGO_ENV`] when set, so it matches the toolchain running xtask.
fn program() -> String {
    std::env::var(CARGO_ENV).unwrap_or_else(|_| CARGO.to_string())
}

/// A new `cargo` [`Command`], with no arguments yet.
pub(crate) fn command() -> Command {
    Command::new(program())
}
