//! Embeds the Windows manifest: uiAccess only in release, per-monitor DPI and no elevation. On MSVC it also stops the
//! linker writing an import library for Qt's exported QML registration, as `kx-ui`'s own link arguments stop here.

use std::env;
use std::error::Error;

use embed_manifest::manifest::{DpiAwareness, ExecutionLevel};
use embed_manifest::{embed_manifest, new_manifest};

/// Application name inside the manifest.
const APP: &str = "KeyXtend";
/// Cargo profile that gets uiAccess; debug builds run unsigned, which Windows would refuse with it.
const UI_ACCESS_PROFILE: &str = "release";
/// MSVC switches that stop the linker writing an import library and its exports file.
const NO_IMPORT_LIB: &[&str] = &["/NOIMPLIB", "/NOEXP"];
/// The target environment that takes them.
const MSVC: &str = "msvc";

#[allow(
    clippy::print_stdout,
    reason = "Cargo reads build-script directives from stdout."
)]
fn main() -> Result<(), Box<dyn Error>> {
    if env::var_os("CARGO_CFG_WINDOWS").is_some() {
        let ui_access = env::var("PROFILE").is_ok_and(|p| p == UI_ACCESS_PROFILE);
        let manifest = new_manifest(APP)
            .dpi_awareness(DpiAwareness::PerMonitorV2)
            .requested_execution_level(ExecutionLevel::AsInvoker)
            .ui_access(ui_access);
        embed_manifest(manifest)?;
    }
    if env::var("CARGO_CFG_TARGET_ENV").as_deref() == Ok(MSVC) {
        for arg in NO_IMPORT_LIB {
            println!("cargo::rustc-link-arg={arg}");
        }
    }
    println!("cargo::rerun-if-changed=build.rs");
    Ok(())
}
