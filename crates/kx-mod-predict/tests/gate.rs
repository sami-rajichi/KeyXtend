//! Prediction holds `Vault` only to hear the notebook's emails (ADR-0038 decision 3, ADR-0039): its code never names
//! another event that gate lets through, so it can never hear a fill or the panel.

#![allow(
    clippy::unwrap_used,
    reason = "a source walk in a test, where a failed read should fail the test"
)]

use std::fs;
use std::path::Path;

use kx_test_support::source::rust_files;

/// The one event prediction hears through `Vault`.
const ONLY: &str = "VaultEmails";

/// What else `Vault` gates, which prediction must never name.
const NEVER: [&str; 10] = [
    "FillType",
    "FillTyped",
    "FillKey",
    "FillView",
    "FillAct",
    "FillLive",
    "FillPanel",
    "FillNames",
    "FillLine",
    "ClipLine",
];

/// The one event that brings prediction a panel's text (ADR-0040 decision 6).
const LINE: &str = "AimedLine";

/// What carries a panel's keys or view without the `Vault` gate, which prediction must never name.
const NO_PANEL: [&str; 4] = ["ClipKey", "ClipView", "ClipPanel", "PanelKey"];

#[test]
fn only_the_aimed_line_brings_a_panels_text() {
    let src = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let lib = fs::read_to_string(src.join("lib.rs")).unwrap();
    assert!(lib.contains(LINE), "the walk reads the code that hears it");
    for file in rust_files(&src).unwrap() {
        let code = fs::read_to_string(&file).unwrap();
        for name in NO_PANEL {
            assert!(!code.contains(name), "{} names {name}", file.display());
        }
    }
}

#[test]
fn prediction_names_only_the_vaults_emails() {
    let src = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let lib = fs::read_to_string(src.join("lib.rs")).unwrap();
    assert!(lib.contains(ONLY), "the walk reads the code that hears it");
    for file in rust_files(&src).unwrap() {
        let code = fs::read_to_string(&file).unwrap();
        for name in NEVER {
            assert!(!code.contains(name), "{} names {name}", file.display());
        }
    }
}
