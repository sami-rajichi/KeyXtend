//! Builds the Quick tests' QML plugin over kx-ui's icon provider and the soft shadows' adopter, with the provider's
//! name, the icons' folder and their file type from `kx_theme::assets`, so the tests draw as the app does. Like kx-ui,
//! it refuses a Qt other than the pinned one, as the adopter leans on how that Qt makes its shadows.
#![allow(
    unsafe_code,
    reason = "cxx-qt-build sets C++ macros only through an unsafe call; a test tool may hold it"
)]

use cxx_qt_build::CxxQtBuilder;
use kx_theme::assets::{ICON_EXT, ICON_PROVIDER, ICONS_AT};
use std::path::PathBuf;

/// kx-ui's Qt pin check and build-kind header, as its own build script uses them.
#[path = "../../crates/kx-ui/qtbuild.rs"]
mod qtbuild;

/// kx-ui's hand-written C++, which holds the provider.
const KX_UI_CPP: &str = "../../crates/kx-ui/cpp";
/// The provider's header there.
const TINT_H: &str = "tint.h";
/// The soft shadows' adopter there, a `QObject` that moc reads.
const SHADOWS_H: &str = "shadows.h";
/// The plugin's header, which moc reads.
const PLUGIN: &str = "plugin.h";
/// The generated header that names the provider, its folder and file type, in its own folder under `OUT_DIR`.
const NAMES_DIR: &str = "kx-test-icons";
const NAMES_H: &str = "kx-test-icons.h";
/// Cargo's variable for the build's output folder.
const OUT_ENV: &str = "OUT_DIR";
/// Qt modules the plugin links: Gui for images, Qml for the plugin, Quick for the provider.
const QT_MODULES: [&str; 3] = ["Gui", "Qml", "Quick"];
/// Macros a plugin over Qt's release libraries needs: export its entry points, and say it is a release build.
const DEFINES: [&str; 2] = ["QT_PLUGIN", "QT_NO_DEBUG"];
/// Links all of the C++, so the entry points moc writes reach the DLL.
const WHOLE: &str = "+whole-archive";
/// MSVC switches that stop the DLL's link, and its test binary's, writing an import library, which nothing links and
/// the linker announces.
const NO_IMPORT_LIB: [&str; 2] = ["/NOIMPLIB", "/NOEXP"];

fn main() {
    let names = qtbuild::pinned_version(&qtbuild::qmake()).and_then(|_| names_header());
    let names = match names {
        Ok(dir) => dir,
        Err(e) => {
            println!("cargo::error={e}");
            return;
        }
    };
    let builder = QT_MODULES
        .iter()
        .fold(CxxQtBuilder::new(), |b, m| b.qt_module(m))
        .include_dir(KX_UI_CPP)
        .include_dir(names)
        .cpp_file(PLUGIN)
        .cpp_file(format!("{KX_UI_CPP}/{SHADOWS_H}"));
    // SAFETY: the closure only adds macros and a link modifier to cxx-qt-build's C++ build, which keeps its own
    // files and flags.
    let builder = unsafe {
        builder.cc_builder(|cc| {
            for name in DEFINES {
                cc.define(name, None);
            }
            cc.link_lib_modifier(WHOLE);
        })
    };
    builder.build();
    if std::env::var("CARGO_CFG_TARGET_ENV").as_deref() == Ok("msvc") {
        for arg in NO_IMPORT_LIB {
            println!("cargo::rustc-link-arg={arg}");
        }
    }
    println!("cargo::rerun-if-changed={PLUGIN}");
    println!("cargo::rerun-if-changed={KX_UI_CPP}/{TINT_H}");
    println!("cargo::rerun-if-changed={KX_UI_CPP}/{SHADOWS_H}");
}

/// Writes the header that names the provider, the icons' folder and their file type, and kx-ui's build kind without
/// its probes, and gives their folder.
fn names_header() -> Result<PathBuf, String> {
    let out = std::env::var_os(OUT_ENV).ok_or("cargo set no OUT_DIR")?;
    let dir = PathBuf::from(out).join(NAMES_DIR);
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let text = format!(
        "#pragma once\n#define KX_ICON_PROVIDER \"{ICON_PROVIDER}\"\n#define KX_ICONS_AT \"{ICONS_AT}\"\n\
         #define KX_ICON_EXT \"{ICON_EXT}\"\n"
    );
    std::fs::write(dir.join(NAMES_H), text).map_err(|e| e.to_string())?;
    qtbuild::kind_header(&dir, None)?;
    Ok(dir)
}
