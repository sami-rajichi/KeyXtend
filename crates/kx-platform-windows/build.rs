//! Sets `kx_live_clipboard` when `KX_LIVE_CLIPBOARD=1`, so the tests that write the real clipboard run, as in CI's
//! Windows job. Without it they report as ignored, and a local `cargo test` leaves the owner's clipboard alone.

use std::env;

/// The variable that opts in to the live clipboard tests.
const LIVE_VAR: &str = "KX_LIVE_CLIPBOARD";
/// Its value that opts in.
const LIVE_ON: &str = "1";
/// The cfg the live clipboard tests read.
const LIVE_CFG: &str = "kx_live_clipboard";
/// The compile-time variable that hands `LIVE_VAR=LIVE_ON` to the crate, so a test checks that CI sets it.
const OPT_IN_ENV: &str = "KX_LIVE_OPT_IN";

#[allow(
    clippy::print_stdout,
    reason = "Cargo reads build-script directives from stdout."
)]
fn main() {
    println!("cargo::rustc-check-cfg=cfg({LIVE_CFG})");
    println!("cargo::rustc-env={OPT_IN_ENV}={LIVE_VAR}={LIVE_ON}");
    println!("cargo::rerun-if-env-changed={LIVE_VAR}");
    println!("cargo::rerun-if-changed=build.rs");
    if env::var(LIVE_VAR).is_ok_and(|v| v == LIVE_ON) {
        println!("cargo::rustc-cfg={LIVE_CFG}");
    }
}
