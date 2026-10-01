//! Embeds the Windows manifest: uiAccess only in release, per-monitor DPI, UTF-8 and no elevation, and sets
//! `kx_ui_access` in the build that asks for uiAccess. On MSVC it also stops the linker writing an import library for
//! Qt's exported QML registration, as `kx-ui`'s own link arguments stop here.

use std::env;
use std::error::Error;

use embed_manifest::manifest::{ActiveCodePage, DpiAwareness, ExecutionLevel};
use embed_manifest::{embed_manifest, new_manifest};

/// Application name inside the manifest.
const APP: &str = "KeyXtend";
/// Cargo profile that gets uiAccess; debug builds run unsigned, which Windows would refuse with it.
const UI_ACCESS_PROFILE: &str = "release";
/// The cfg set in the build whose manifest asks for uiAccess, so the code follows the same choice.
const UI_ACCESS_CFG: &str = "kx_ui_access";
/// MSVC switches that stop the linker writing an import library and its exports file.
const NO_IMPORT_LIB: &[&str] = &["/NOIMPLIB", "/NOEXP"];
/// The target environment that takes them.
const MSVC: &str = "msvc";

#[allow(
    clippy::print_stdout,
    reason = "Cargo reads build-script directives from stdout."
)]
fn main() -> Result<(), Box<dyn Error>> {
    println!("cargo::rustc-check-cfg=cfg({UI_ACCESS_CFG})");
    if env::var_os("CARGO_CFG_WINDOWS").is_some() {
        let ui_access = env::var("PROFILE").is_ok_and(|p| p == UI_ACCESS_PROFILE);
        let manifest = new_manifest(APP)
            .dpi_awareness(DpiAwareness::PerMonitorV2)
            .requested_execution_level(ExecutionLevel::AsInvoker)
            // UTF-8 narrow text: the C runtime then narrows no variable's name to a Qt one by best fit.
            .active_code_page(ActiveCodePage::Utf8)
            .ui_access(ui_access);
        embed_manifest(manifest)?;
        if ui_access {
            println!("cargo::rustc-cfg={UI_ACCESS_CFG}");
        }
    }
    if env::var("CARGO_CFG_TARGET_ENV").as_deref() == Ok(MSVC) {
        for arg in NO_IMPORT_LIB {
            println!("cargo::rustc-link-arg={arg}");
        }
    }
    println!("cargo::rerun-if-changed=build.rs");
    Ok(())
}
