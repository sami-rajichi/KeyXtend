//! Embeds the Windows manifest: per-monitor DPI awareness, so logged mouse points are physical pixels.

use std::error::Error;

use embed_manifest::manifest::{DpiAwareness, ExecutionLevel};
use embed_manifest::{embed_manifest, new_manifest};

/// Application name inside the manifest.
const APP: &str = "KeyXtend.Tools.TargetWindow";

#[allow(
    clippy::print_stdout,
    reason = "Cargo reads build-script directives from stdout."
)]
fn main() -> Result<(), Box<dyn Error>> {
    if std::env::var_os("CARGO_CFG_WINDOWS").is_some() {
        let manifest = new_manifest(APP)
            .dpi_awareness(DpiAwareness::PerMonitorV2)
            .requested_execution_level(ExecutionLevel::AsInvoker);
        embed_manifest(manifest)?;
    }
    println!("cargo:rerun-if-changed=build.rs");
    Ok(())
}
