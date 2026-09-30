//! Builds the QML module `KeyXtend.Ui`, its bridges and its fonts and icons when the `qt` feature is on (ADR-0013,
//! ADR-0016). MSVC reads the generated C++ as UTF-8 through `.cargo/config.toml`, so this script holds no unsafe call.

fn main() {
    #[cfg(feature = "qt")]
    qt::build();
    println!("cargo::rerun-if-changed=build.rs");
}

/// The Qt build, with its file tables.
#[cfg(feature = "qt")]
mod qt {
    use cxx_qt_build::{CxxQtBuilder, QResource, QResources, QmlModule};
    use kx_theme::assets::{Bundled, ICON_EXT, ICONS_AT, bundle};
    use std::path::Path;

    /// QML module URI; its files land under `qrc:/qt/qml/<uri as folders>/`.
    const URI: &str = "KeyXtend.Ui";
    /// Every QML file of the module, one row each; the first is the root the app loads.
    const QML: &[&str] = &[
        "qml/main.qml",
        "qml/KbWindow.qml",
        "qml/Plate.qml",
        "qml/TopBar.qml",
        "qml/Chip.qml",
        "qml/Key.qml",
        "qml/KeyText.qml",
        "qml/CapFace.qml",
        "qml/Inset.qml",
        "qml/Led.qml",
        "qml/DPad.qml",
        "qml/Corner.qml",
        "qml/Mover.qml",
        "qml/Catch.qml",
        "qml/Bubble.qml",
        "qml/Glow.qml",
        "qml/Tip.qml",
        "qml/Hover.qml",
        "qml/Label.qml",
        "qml/Icon.qml",
        "qml/Shade.qml",
        "qml/Tween.qml",
        "qml/ColourTween.qml",
        "qml/PopIn.qml",
        "qml/LangCycle.qml",
        "qml/LangList.qml",
        "qml/LangPanel.qml",
        "qml/LangRow.qml",
        "qml/Notice.qml",
        "qml/TrayMenu.qml",
        "qml/TrayPicture.qml",
    ];
    /// The Rust files holding cxx-qt bridges.
    const BRIDGES: &[&str] = &["src/bridges/ui.rs", "src/bridges/keyboard.rs"];
    /// The repo's `assets/` folder, from this crate.
    const ASSETS: &str = "../../assets";
    /// The variable that hands the root's resource URL to the crate.
    const MAIN_ENV: &str = "KX_UI_MAIN_QML";
    /// The variable that hands the icon folder, icon type and font files, as JSON, to the crate.
    const ASSETS_ENV: &str = "KX_UI_ASSETS";
    /// MSVC switches that stop this crate's test binaries writing an import library for Qt's exported QML
    /// registration, which the linker would announce on every link.
    const NO_IMPORT_LIB: &[&str] = &["/NOIMPLIB", "/NOEXP"];

    /// Compiles the bridges, the QML, the fonts and the icons into the crate, and names their URLs.
    pub fn build() {
        let (resources, fonts) = match resources() {
            Ok(done) => done,
            Err(e) => {
                println!("cargo::error={e}");
                return;
            }
        };
        let module = QmlModule::new(URI).qml_files(QML.iter().copied());
        CxxQtBuilder::new_qml_module(module)
            .files(BRIDGES.iter().copied())
            .qrc_resources(resources)
            .build();
        let base = format!("qrc:/qt/qml/{}", URI.replace('.', "/"));
        let root = QML.first().copied().unwrap_or_default();
        println!("cargo::rustc-env={MAIN_ENV}={base}/{root}");
        println!(
            "cargo::rustc-env={ASSETS_ENV}={}",
            assets_json(&base, &fonts)
        );
        println!("cargo::rerun-if-changed={ASSETS}");
        if std::env::var("CARGO_CFG_TARGET_ENV").as_deref() == Ok("msvc") {
            for arg in NO_IMPORT_LIB {
                println!("cargo::rustc-link-arg={arg}");
            }
        }
    }

    /// The icons and fonts `kx_theme::assets::bundle` names, as resources next to the QML files, and the fonts' names.
    fn resources() -> Result<(QResources, Vec<String>), String> {
        let b = bundle(Path::new(ASSETS))?;
        let names = b.fonts.iter().map(|f| f.name.clone()).collect();
        let files = b.icons.iter().chain(&b.fonts);
        let list: Option<Vec<QResource>> = files.map(aliased).collect();
        let list = list.ok_or("a resource file could not be named")?;
        Ok((QResources::new().resources(list), names))
    }

    /// Bundled file `b` as a resource under its bundle name. cxx-qt-build does not export its file type, so the file
    /// comes from a one-file resource.
    fn aliased(b: &Bundled) -> Option<QResource> {
        let one = QResource::new().file(&b.file);
        let file = one.get_files().next()?.clone().alias(b.name.clone());
        Some(QResource::new().file(file))
    }

    /// What QML reads from `Ui.assets()`: the icon folder URL, the icon type and each font's URL.
    fn assets_json(base: &str, fonts: &[String]) -> String {
        let fonts: Vec<String> = fonts.iter().map(|f| format!("{base}/{f}")).collect();
        let value = serde_json::json!({ "icons": format!("{base}/{ICONS_AT}"), "ext": ICON_EXT, "fonts": fonts });
        value.to_string()
    }
}
