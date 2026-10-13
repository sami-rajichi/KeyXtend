//! Embeds the Windows manifest (uiAccess only in release without the gate-only feature, per-monitor DPI, UTF-8, no
//! elevation) and sets `kx_no_console` in every release build and `kx_ui_access` in the uiAccess one. On MSVC it also
//! stops the linker writing an import library for Qt's exported QML registration, as `kx-ui`'s own link arguments stop
//! here.

#[path = "build/choice.rs"]
mod choice;

use std::env;
use std::error::Error;

use embed_manifest::manifest::{ActiveCodePage, DpiAwareness, ExecutionLevel};
use embed_manifest::{embed_manifest, new_manifest};

/// Application name inside the manifest.
const APP: &str = "KeyXtend";
/// The cfg set in the build whose manifest asks for uiAccess, so the code follows the same choice.
const UI_ACCESS_CFG: &str = "kx_ui_access";
/// The cfg set in every build that opens no console window, a superset of the uiAccess one.
const NO_CONSOLE_CFG: &str = "kx_no_console";
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
    println!("cargo::rustc-check-cfg=cfg({NO_CONSOLE_CFG})");
    if env::var_os("CARGO_CFG_WINDOWS").is_some() {
        let profile = env::var("PROFILE").unwrap_or_default();
        let ui_access = choice::ui_access(&profile, env::var_os(choice::GATE_INPUT_VAR).is_some());
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
        if choice::no_console(&profile) {
            println!("cargo::rustc-cfg={NO_CONSOLE_CFG}");
        }
    }
    if env::var("CARGO_CFG_TARGET_ENV").as_deref() == Ok(MSVC) {
        for arg in NO_IMPORT_LIB {
            println!("cargo::rustc-link-arg={arg}");
        }
    }
    // Cargo rebuilds and reruns this script when `build/choice.rs` changes too, as the script's own module.
    println!("cargo::rerun-if-changed=build.rs");
    Ok(())
}
