//! Embeds the Windows manifest: uiAccess in release, so the harness can click a uiAccess keyboard.

use embed_manifest::manifest::{DpiAwareness, ExecutionLevel};
use embed_manifest::{embed_manifest, new_manifest};

/// Application name inside the manifest.
const APP: &str = "KeyXtend.Spike.Harness";
/// Cargo profile that gets uiAccess; debug builds run unsigned.
const UI_ACCESS_PROFILE: &str = "release";

fn main() {
    if std::env::var_os("CARGO_CFG_WINDOWS").is_some() {
        let ui_access = std::env::var("PROFILE").is_ok_and(|p| p == UI_ACCESS_PROFILE);
        let manifest = new_manifest(APP)
            .dpi_awareness(DpiAwareness::PerMonitorV2)
            .requested_execution_level(ExecutionLevel::AsInvoker)
            .ui_access(ui_access);
        embed_manifest(manifest).expect("manifest should embed");
    }
    println!("cargo:rerun-if-changed=build.rs");
}
