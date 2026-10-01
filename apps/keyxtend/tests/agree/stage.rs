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
