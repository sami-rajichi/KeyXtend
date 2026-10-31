//! Embeds the worker's Windows manifest: it runs as the user who starts it, with no elevation and never with uiAccess,
//! which the stage check refuses (ADR-0004).

use std::env;
use std::error::Error;

use embed_manifest::manifest::ExecutionLevel;
use embed_manifest::{embed_manifest, new_manifest};

/// Program name inside the manifest.
const NAME: &str = "KeyXtend.Worker";

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
    println!("cargo::rerun-if-changed=build.rs");
    Ok(())
}
