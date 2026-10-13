//! The build script's choice: only a release build without the gate-only feature asks for uiAccess, so the gate-input
//! test build runs unsigned and can never hold it.

#[path = "../build/choice.rs"]
mod choice;

/// The app's manifest, whose features name the gate-only build.
const MANIFEST: &str = include_str!("../Cargo.toml");
/// The gate-only feature, as the manifest names it.
const GATE_INPUT: &str = "gate-input";
/// The prefix of the variable Cargo sets for a build script per turned-on feature.
const FEATURE_VAR: &str = "CARGO_FEATURE_";
/// A profile that is not the release one.
const DEBUG: &str = "debug";

#[test]
fn only_a_release_build_without_the_gate_feature_asks_for_ui_access() {
    let release = choice::UI_ACCESS_PROFILE;
    assert!(choice::ui_access(release, false));
    assert!(
        !choice::ui_access(release, true),
        "the test build runs unsigned"
    );
    assert!(!choice::ui_access(DEBUG, false));
    assert!(!choice::ui_access(DEBUG, true));
}

#[test]
fn the_variable_read_is_cargos_for_the_apps_gate_feature() {
    let manifest: toml::Table = MANIFEST.parse().expect("the manifest parses");
    let features = manifest["features"].as_table().expect("a features table");
    assert!(features.contains_key(GATE_INPUT), "{GATE_INPUT}");
    let var = format!(
        "{FEATURE_VAR}{}",
        GATE_INPUT.to_uppercase().replace('-', "_")
    );
    assert_eq!(choice::GATE_INPUT_VAR, var);
}
