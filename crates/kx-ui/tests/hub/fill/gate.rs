//! The ui holds `Vault` as an event gate only (ADR-0038 decision 3), and never `VaultData` (ADR-0039): its code never
//! types a fill, hears one typed, sends the panel's keys, reads the notebook's emails, or asks for the notebook store,
//! the boxes or the places.

use std::fs;
use std::path::Path;

use kx_test_support::source::rust_files;

/// What `Vault` gates that the ui must never name: the keyboard's fill, its answer and its panel keys, prediction's
/// emails, the lines the panels tell the keyboard, and the vault module's three services, which `VaultData` gates.
const NEVER: [&str; 9] = [
    "FillType",
    "FillTyped",
    "FillKey",
    "VaultEmails",
    "VaultStoreKey",
    "BoxesKey",
    "PlacesKey",
    "FillLine",
    "ClipLine",
];

#[test]
fn the_ui_never_names_a_fill_the_vaults_emails_or_its_services() {
    let src = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let files = rust_files(&src).unwrap();
    assert!(
        files.iter().any(|f| f.ends_with("hub/fill.rs")),
        "the walk reached the hub"
    );
    for file in files {
        let code = fs::read_to_string(&file).unwrap();
        for name in NEVER {
            assert!(!code.contains(name), "{} names {name}", file.display());
        }
    }
}
