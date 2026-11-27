//! Embeds the worker's Windows manifest: it runs as the user who starts it, with no elevation and never with uiAccess,
//! which the stage check refuses (ADR-0004). It also delay-loads the libraries only a video job calls, as the recorder's
//! table names them, so a system without one still starts every other job (ADR-0063); `tests/imports.rs` checks it.

use std::env;
use std::error::Error;

use embed_manifest::manifest::ExecutionLevel;
use embed_manifest::{embed_manifest, new_manifest};

/// The recorder's own table reader, so its checks and names hold here too; the build uses only its libraries. Its
/// crate formats it, so rustfmt skips it here.
#[allow(dead_code, reason = "The build reads only the table's libraries.")]
#[path = "../../crates/kx-platform-record/src/table.rs"]
#[rustfmt::skip]
mod table;

/// Program name inside the manifest.
const NAME: &str = "KeyXtend.Worker";
/// The linker's switch that delay-loads a library, and the helper library it needs.
const DELAY: &str = "/DELAYLOAD:";
const HELPER: &str = "delayimp.lib";
/// The only toolchain whose linker takes those.
const MSVC: &str = "msvc";

#[allow(
    clippy::print_stdout,
    reason = "Cargo reads build-script directives from stdout."
)]
fn main() -> Result<(), Box<dyn Error>> {
    if env::var_os("CARGO_CFG_WINDOWS").is_some() {
        let manifest = new_manifest(NAME)
            .requested_execution_level(ExecutionLevel::AsInvoker)
            .ui_access(false);
        embed_manifest(manifest)?;
    }
    if env::var("CARGO_CFG_TARGET_ENV").is_ok_and(|e| e == MSVC) {
        // A table with no libraries, or none at all, fails the build rather than delay-loading nothing.
        let shipped = table::Table::parse(table::TEXT)?;
        for dll in &shipped.media_dlls {
            println!("cargo::rustc-link-arg-bins={DELAY}{dll}");
        }
        println!("cargo::rustc-link-arg-bins={HELPER}");
    }
    // The table's files are compiled into this script, so a change to them rebuilds and reruns it.
    println!("cargo::rerun-if-changed=build.rs");
    Ok(())
}
