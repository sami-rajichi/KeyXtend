//! The build script's choice: only a release build without a gate-only feature asks for uiAccess, so the gate test
//! builds run unsigned and can never hold it; every release build opens no console.

#[path = "../build/choice.rs"]
mod choice;

/// The app's manifest, whose features name the gate-only builds.
const MANIFEST: &str = include_str!("../Cargo.toml");
/// The gate-only features, as the manifest names them.
const GATES: [&str; 2] = ["gate-input", "gate-voice"];
/// The prefix of the variable Cargo sets for a build script per turned-on feature.
const FEATURE_VAR: &str = "CARGO_FEATURE_";
/// A profile that is not the release one.
const DEBUG: &str = "debug";

#[test]
fn only_a_release_build_without_a_gate_feature_asks_for_ui_access() {
    let release = choice::RELEASE;
    assert!(choice::ui_access(release, false));
    assert!(
        !choice::ui_access(release, true),
        "the test builds run unsigned"
    );
    assert!(!choice::ui_access(DEBUG, false));
    assert!(!choice::ui_access(DEBUG, true));
}

#[test]
fn every_release_build_opens_no_console_and_ui_access_needs_that() {
    assert!(choice::no_console(choice::RELEASE));
    assert!(!choice::no_console(DEBUG));
    for profile in [choice::RELEASE, DEBUG] {
        for gate in [false, true] {
            let asks = choice::ui_access(profile, gate);
            assert!(!asks || choice::no_console(profile), "{profile} {gate}");
        }
    }
}

#[test]
fn the_variables_read_are_cargos_for_the_apps_gate_features() {
    let manifest: toml::Table = MANIFEST.parse().expect("the manifest parses");
    let features = manifest["features"].as_table().expect("a features table");
    let vars: Vec<String> = GATES
        .iter()
        .map(|g| format!("{FEATURE_VAR}{}", g.to_uppercase().replace('-', "_")))
        .collect();
    for gate in GATES {
        assert!(features.contains_key(gate), "{gate}");
    }
    assert_eq!(choice::GATE_VARS.to_vec(), vars);
}
