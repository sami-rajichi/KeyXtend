//! Builds the QML module and the bridge, and embeds the Windows manifest.

use cxx_qt_build::{CxxQtBuilder, QmlModule};
use embed_manifest::manifest::{DpiAwareness, ExecutionLevel};
use embed_manifest::{embed_manifest, new_manifest};

/// QML module URI; its files land under `qrc:/qt/qml/<uri as folders>/`.
const URI: &str = "KeyXtend.Spike";
/// The window, as a file of the QML module.
const MAIN_QML: &str = "qml/main.qml";
/// The Rust file holding the cxx-qt bridge.
const BRIDGE: &str = "src/bridge.rs";
/// Identity name in the manifest.
const MANIFEST_NAME: &str = "KeyXtend.Spike.Qt";
/// Only this profile asks for uiAccess; unsigned debug builds would be refused.
const UIACCESS_PROFILE: &str = "release";

/// Reads C++ sources as UTF-8; without it MSVC uses the ANSI code page and garbles QML text.
const UTF8_FLAG: &str = "/utf-8";

fn main() {
    let builder = CxxQtBuilder::new_qml_module(QmlModule::new(URI).qml_file(MAIN_QML)).file(BRIDGE);
    // SAFETY: only adds one compiler flag; the cc::Build is otherwise left as cxx-qt set it.
    let builder = unsafe {
        builder.cc_builder(|cc| {
            cc.flag_if_supported(UTF8_FLAG);
        })
    };
    builder.build();
    // One source for the resource URL that `main.rs` loads.
    let url = format!("qrc:/qt/qml/{}/{MAIN_QML}", URI.replace('.', "/"));
    println!("cargo::rustc-env=QT_KB_MAIN_QML={url}");
    if std::env::var_os("CARGO_CFG_WINDOWS").is_some() {
        let ui_access = std::env::var("PROFILE").is_ok_and(|p| p == UIACCESS_PROFILE);
        let manifest = new_manifest(MANIFEST_NAME)
            .dpi_awareness(DpiAwareness::PerMonitorV2)
            .requested_execution_level(ExecutionLevel::AsInvoker)
            .ui_access(ui_access);
        embed_manifest(manifest).expect("embed the Windows manifest");
    }
    println!("cargo::rerun-if-changed=build.rs");
}
