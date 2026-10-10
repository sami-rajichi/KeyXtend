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
    use std::path::{Path, PathBuf};
    use std::process::Command;

    /// QML module URI; its files land under `qrc:/qt/qml/<uri as folders>/`.
    const URI: &str = "KeyXtend.Ui";
    /// Every QML file of the module, one row each; the first is the root the app loads.
    const QML: &[&str] = &[
        "qml/main.qml",
        "qml/App.qml",
        "qml/KbWindow.qml",
        "qml/Spots.qml",
        "qml/Rescale.qml",
        "qml/Placing.qml",
        "qml/Guard.qml",
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
        "qml/PressWatch.qml",
    ];
    /// The Rust files holding cxx-qt bridges.
    const BRIDGES: &[&str] = &[
        "src/bridges/ui.rs",
        "src/bridges/keyboard.rs",
        "src/bridges/harden.rs",
        "src/bridges/device.rs",
        "src/bridges/fonts.rs",
    ];
    /// Qt modules the hand-written C++ links beside those cxx-qt-lib brings: Quick, for the windows' graphics device.
    const QT_MODULES: &[&str] = &["Quick"];
    /// The variable naming Qt's qmake, as cxx-qt-build reads it, and the program it runs when that is unset.
    const QMAKE_ENV: &str = "QMAKE";
    const QMAKE: &str = "qmake";
    /// qmake's keys for Qt's header folder and version: `rhi/qrhi.h` sits under `<headers>/QtGui/<version>/QtGui`.
    const HEADERS_KEY: &str = "QT_INSTALL_HEADERS";
    const VERSION_KEY: &str = "QT_VERSION";
    const GUI: &str = "QtGui";
    /// The workspace manifest and the path of the one Qt version it pins, which the graphics device glue is checked for.
    const WORKSPACE: &str = "../../Cargo.toml";
    const PIN_PATH: [&str; 4] = ["workspace", "metadata", "stage", "qt_version"];
    /// The hand-written C++ the bridges include.
    const CPP: &str = "cpp";
    /// The header that tells that C++ the build kind, in its own folder under `OUT_DIR`, and the macro a debug build
    /// defines in it to turn on the test-only device loss and window counts (ADR-0021).
    const KIND_DIR: &str = "kx-ui-build";
    const KIND_H: &str = "kx-ui-build.h";
    const PROBES: &str = "KX_UI_PROBES";
    /// Cargo's variables for the build's output folder, and for debug assertions, set only when they are on.
    const OUT_ENV: &str = "OUT_DIR";
    const DEBUG_ENV: &str = "CARGO_CFG_DEBUG_ASSERTIONS";
    /// The resource root of compiled-in QML modules, as cxx-qt-build lays them out.
    const QML_ROOT: &str = "qrc:/qt/qml";
    /// The variable that hands that root to the crate.
    const ROOT_ENV: &str = "KX_UI_QML_ROOT";
    /// The repo's `assets/` folder, from this crate.
    const ASSETS: &str = "../../assets";
    /// The variable that hands the root's resource URL to the crate.
    const MAIN_ENV: &str = "KX_UI_MAIN_QML";
    /// The variable that hands the module's URI to the crate.
    const URI_ENV: &str = "KX_UI_QML_URI";
    /// The variable that hands the module's resource folder, as a file path Qt opens, to the crate.
    const FILES_ENV: &str = "KX_UI_FILES";
    /// The scheme a resource URL has and a resource file path drops.
    const QRC_SCHEME: &str = "qrc";
    /// The variable that hands the icon folder, icon type and font files, as JSON, to the crate.
    const ASSETS_ENV: &str = "KX_UI_ASSETS";
    /// MSVC switches that stop this crate's test binaries writing an import library for Qt's exported QML
    /// registration, which the linker would announce on every link.
    const NO_IMPORT_LIB: &[&str] = &["/NOIMPLIB", "/NOEXP"];

    /// Compiles the bridges, the QML, the fonts and the icons into the crate, and names their URLs.
    pub fn build() {
        let parts = resources().and_then(|(r, f)| Ok((r, f, rhi_headers()?, kind_header()?)));
        let (resources, fonts, rhi, kind) = match parts {
            Ok(done) => done,
            Err(e) => {
                println!("cargo::error={e}");
                return;
            }
        };
        let module = QmlModule::new(URI).qml_files(QML.iter().copied());
        let builder = CxxQtBuilder::new_qml_module(module)
            .files(BRIDGES.iter().copied())
            .qrc_resources(resources)
            .include_dir(rhi)
            .include_dir(kind);
        QT_MODULES
            .iter()
            .fold(builder, |b, m| b.qt_module(m))
            .build();
        hand_over(&fonts);
        println!("cargo::rerun-if-changed={ASSETS}");
        println!("cargo::rerun-if-changed={CPP}");
        if std::env::var("CARGO_CFG_TARGET_ENV").as_deref() == Ok("msvc") {
            for arg in NO_IMPORT_LIB {
                println!("cargo::rustc-link-arg={arg}");
            }
        }
    }

    /// Hands the crate the root's URL, the module's URI, the assets as JSON and the resource roots.
    fn hand_over(fonts: &[String]) {
        let base = format!("{QML_ROOT}/{}", URI.replace('.', "/"));
        let root = QML.first().copied().unwrap_or_default();
        println!("cargo::rustc-env={MAIN_ENV}={base}/{root}");
        println!("cargo::rustc-env={URI_ENV}={URI}");
        println!(
            "cargo::rustc-env={ASSETS_ENV}={}",
            assets_json(&base, fonts)
        );
        println!("cargo::rustc-env={ROOT_ENV}={QML_ROOT}");
        println!(
            "cargo::rustc-env={FILES_ENV}={}",
            base.replacen(QRC_SCHEME, "", 1)
        );
    }

    /// Qt's versioned `QtGui` header folder, which holds the graphics device's `rhi/qrhi.h` (ADR-0021); an error when
    /// Qt is not the version the workspace pins, as that header keeps few promises across versions.
    fn rhi_headers() -> Result<PathBuf, String> {
        println!("cargo::rerun-if-env-changed={QMAKE_ENV}");
        let qmake = std::env::var_os(QMAKE_ENV).unwrap_or_else(|| QMAKE.into());
        let query = |key: &str| -> Result<String, String> {
            let out = Command::new(&qmake).args(["-query", key]).output();
            let out = out.map_err(|e| format!("qmake -query {key}: {e}"))?;
            let text = String::from_utf8(out.stdout).map_err(|e| e.to_string())?;
            Ok(text.trim().to_owned())
        };
        let (version, pinned) = (query(VERSION_KEY)?, pinned_qt()?);
        if version != pinned {
            return Err(format!(
                "Qt {version} is not the pinned {pinned}; check the graphics device glue first (ADR-0021)"
            ));
        }
        let headers = PathBuf::from(query(HEADERS_KEY)?);
        Ok(headers.join(GUI).join(version).join(GUI))
    }

    /// The Qt version the workspace manifest pins.
    fn pinned_qt() -> Result<String, String> {
        println!("cargo::rerun-if-changed={WORKSPACE}");
        let text = std::fs::read_to_string(WORKSPACE).map_err(|e| format!("{WORKSPACE}: {e}"))?;
        let table: toml::Table = text.parse().map_err(|e| format!("{WORKSPACE}: {e}"))?;
        let root = toml::Value::Table(table);
        let pin = PIN_PATH.iter().try_fold(&root, |v, key| v.get(key));
        pin.and_then(toml::Value::as_str)
            .map(str::to_owned)
            .ok_or_else(|| format!("{WORKSPACE} pins no Qt version at {}", PIN_PATH.join(".")))
    }

    /// Writes the build-kind header, which defines `PROBES` only with debug assertions, and gives its folder.
    fn kind_header() -> Result<PathBuf, String> {
        let out = std::env::var_os(OUT_ENV).ok_or("cargo set no OUT_DIR")?;
        let dir = PathBuf::from(out).join(KIND_DIR);
        std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
        let debug = std::env::var_os(DEBUG_ENV).is_some();
        let probes = if debug {
            format!("#define {PROBES} 1\n")
        } else {
            String::new()
        };
        let text = format!("#pragma once\n{probes}");
        std::fs::write(dir.join(KIND_H), text).map_err(|e| e.to_string())?;
        Ok(dir)
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
