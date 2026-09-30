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
    use kx_theme::Themes;
    use kx_theme::assets::{FONT_DIR, ICON_EXT, font_files, icon_file};
    use std::collections::BTreeSet;
    use std::path::{Path, PathBuf};

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
    ];
    /// The Rust files holding cxx-qt bridges.
    const BRIDGES: &[&str] = &["src/bridges/ui.rs", "src/bridges/keyboard.rs"];
    /// The repo's `assets/` folder, from this crate.
    const ASSETS: &str = "../../assets";
    /// The resource folder the icons land in, next to the QML files.
    const ICONS: &str = "icons";
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

    /// The icons and fonts as resources next to the QML files, and the fonts' resource names.
    fn resources() -> Result<(QResources, Vec<String>), String> {
        let assets = Path::new(ASSETS);
        let (icons, fonts) = (icons(assets)?, fonts(assets)?);
        let names = fonts.iter().map(|(alias, _)| alias.clone()).collect();
        let files = icons.into_iter().chain(fonts);
        let list: Option<Vec<QResource>> =
            files.map(|(alias, path)| aliased(&path, alias)).collect();
        let list = list.ok_or("a resource file could not be named")?;
        Ok((QResources::new().resources(list), names))
    }

    /// File `path` as resource `alias`. cxx-qt-build does not export its file type, so the file comes from a one-file
    /// resource.
    fn aliased(path: &Path, alias: String) -> Option<QResource> {
        let one = QResource::new().file(path);
        let file = one.get_files().next()?.clone().alias(alias);
        Some(QResource::new().file(file))
    }

    /// Each icon the theme tokens name, as its resource name and file; a missing file is an error naming it.
    fn icons(assets: &Path) -> Result<Vec<(String, PathBuf)>, String> {
        let themes = Themes::shipped().map_err(|e| e.to_string())?;
        let names: BTreeSet<&str> = themes.icons.names().into_iter().collect();
        names
            .into_iter()
            .map(|name| {
                let file = icon_file(assets, name)
                    .ok_or(format!("no icon file for {name} under {ASSETS}"))?;
                Ok((format!("{ICONS}/{name}.{ICON_EXT}"), file))
            })
            .collect()
    }

    /// Each bundled font, as its resource name (family folder and a URL-safe file name) and file.
    fn fonts(assets: &Path) -> Result<Vec<(String, PathBuf)>, String> {
        let files = font_files(assets).map_err(|e| format!("{ASSETS}/{FONT_DIR}: {e}"))?;
        let named = files.into_iter().map(|path| {
            let family = path.parent().and_then(Path::file_name);
            let file = path.file_name();
            let name = |p: Option<&std::ffi::OsStr>| safe(&p.unwrap_or_default().to_string_lossy());
            let alias = format!("{FONT_DIR}/{}/{}", name(family), name(file));
            (alias, path)
        });
        Ok(named.collect())
    }

    /// `name` with every character a URL path would escape, such as `[` or `,`, made a `-`.
    fn safe(name: &str) -> String {
        let keep = |c: char| c.is_ascii_alphanumeric() || matches!(c, '.' | '-' | '_');
        name.chars()
            .map(|c| if keep(c) { c } else { '-' })
            .collect()
    }

    /// What QML reads from `Ui.assets()`: the icon folder URL, the icon type and each font's URL.
    fn assets_json(base: &str, fonts: &[String]) -> String {
        let fonts: Vec<String> = fonts.iter().map(|f| format!("{base}/{f}")).collect();
        let value = serde_json::json!({ "icons": format!("{base}/{ICONS}"), "ext": ICON_EXT, "fonts": fonts });
        value.to_string()
    }
}
