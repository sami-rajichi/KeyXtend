//! Builds the QML module `KeyXtend.Ui` and its bridges when the `qt` feature is on (ADR-0013).
//! MSVC reads the generated C++ as UTF-8 through `.cargo/config.toml`, so this script holds no unsafe call.

fn main() {
    #[cfg(feature = "qt")]
    qt::build();
    println!("cargo::rerun-if-changed=build.rs");
}

/// The Qt build, with its file tables.
#[cfg(feature = "qt")]
mod qt {
    use cxx_qt_build::{CxxQtBuilder, QmlModule};

    /// QML module URI; its files land under `qrc:/qt/qml/<uri as folders>/`.
    const URI: &str = "KeyXtend.Ui";
    /// Every QML file of the module, one row each; the first is the root the app loads.
    const QML: &[&str] = &["qml/main.qml"];
    /// The Rust files holding cxx-qt bridges.
    const BRIDGES: &[&str] = &["src/bridges/ui.rs", "src/bridges/keyboard.rs"];
    /// The variable that hands the root's resource URL to the crate.
    const MAIN_ENV: &str = "KX_UI_MAIN_QML";
    /// MSVC switches that stop this crate's test binaries writing an import library for Qt's exported QML
    /// registration, which the linker would announce on every link.
    const NO_IMPORT_LIB: &[&str] = &["/NOIMPLIB", "/NOEXP"];

    /// Compiles the bridges and the QML into the crate, and names the root's URL.
    pub fn build() {
        let module = QmlModule::new(URI).qml_files(QML.iter().copied());
        CxxQtBuilder::new_qml_module(module)
            .files(BRIDGES.iter().copied())
            .build();
        let root = QML.first().copied().unwrap_or_default();
        let url = format!("qrc:/qt/qml/{}/{root}", URI.replace('.', "/"));
        println!("cargo::rustc-env={MAIN_ENV}={url}");
        if std::env::var("CARGO_CFG_TARGET_ENV").as_deref() == Ok("msvc") {
            for arg in NO_IMPORT_LIB {
                println!("cargo::rustc-link-arg={arg}");
            }
        }
    }
}
