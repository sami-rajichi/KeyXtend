//! The stage settings in the root `Cargo.toml` name this app and the QML folder kx-ui pins, so `cargo xtask stage`
//! puts Qt's QML modules where the uiAccess build looks for them.

/// The root manifest, from this crate.
const ROOT_MANIFEST: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../Cargo.toml");

/// The `[workspace.metadata.stage]` section.
fn stage() -> toml::Table {
    let text = std::fs::read_to_string(ROOT_MANIFEST).unwrap();
    let root: toml::Table = text.parse().unwrap();
    let section = &root["workspace"]["metadata"]["stage"];
    section.as_table().unwrap().clone()
}

#[test]
fn the_stage_puts_qml_modules_where_the_app_pins_them() {
    assert_eq!(stage()["qml_dir"].as_str(), Some(kx_ui::QML_DIR));
}

#[test]
fn the_stage_builds_this_app() {
    assert_eq!(stage()["app"].as_str(), Some(env!("CARGO_PKG_NAME")));
}

/// QML modules that need no staged file because a library Qt Quick loads registers them (measured in P3 Task 20).
const FROM_LIBRARIES: &[&str] = &["QtQml.Models"];
/// kx-ui's shipped QML, from this crate.
const KX_UI_QML: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../crates/kx-ui/qml");
/// The keyword that starts an import line.
const IMPORT: &str = "import ";
/// The quote that starts a file or folder import, which names no module.
const QUOTE: char = '"';

/// Every module the shipped QML imports, by URI.
fn imports() -> std::collections::BTreeSet<String> {
    let mut found = std::collections::BTreeSet::new();
    for entry in std::fs::read_dir(KX_UI_QML).unwrap() {
        let text = std::fs::read_to_string(entry.unwrap().path()).unwrap();
        let uris = text
            .lines()
            .filter_map(|l| l.trim_start().strip_prefix(IMPORT));
        let modules = uris.filter_map(|rest| rest.split_whitespace().next());
        found.extend(
            modules
                .filter(|uri| !uri.starts_with(QUOTE))
                .map(str::to_string),
        );
    }
    found
}

/// True when `uri` is the app's own module, compiled into the program.
fn compiled_in(uri: &str) -> bool {
    kx_ui::MAIN_QML.contains(&format!("/{}/", uri.replace('.', "/")))
}

#[test]
fn every_qml_import_has_its_qmldir_staged() {
    let stage = stage();
    let staged: Vec<&str> = stage["qml"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(toml::Value::as_str)
        .collect();
    let imports = imports();
    assert!(
        imports.iter().any(|uri| compiled_in(uri)),
        "the app's own module is imported"
    );
    for uri in imports
        .iter()
        .filter(|uri| !compiled_in(uri) && !FROM_LIBRARIES.contains(&uri.as_str()))
    {
        let qmldir = format!("{}/qmldir", uri.replace('.', "/"));
        assert!(
            staged.contains(&qmldir.as_str()),
            "{uri} needs {qmldir} in the stage's qml list"
        );
    }
    for uri in FROM_LIBRARIES {
        assert!(
            imports.contains(*uri),
            "{uri} is no longer imported; drop it from FROM_LIBRARIES"
        );
    }
}
