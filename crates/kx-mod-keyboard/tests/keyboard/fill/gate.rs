//! The keyboard holds `Vault` as an event gate only (ADR-0038 decision 3, ADR-0039): it types a fill and hears the
//! panel say it opened, and its code never reads the panel's view, sends its clicks or hears the notebook's emails.

use std::fs;
use std::path::Path;

use kx_test_support::source::rust_files;

/// What `Vault` gates that the keyboard must never name: the panel's view, clicks and label names, and prediction's
/// emails.
const NEVER: [&str; 4] = ["VaultEmails", "FillView", "FillAct", "FillNames"];

#[test]
fn the_keyboard_never_names_the_panels_view_its_clicks_or_the_emails() {
    let src = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let files = rust_files(&src).unwrap();
    assert!(
        files.iter().any(|f| f.ends_with("face/fill.rs")),
        "the walk reached the fill"
    );
    for file in files {
        let code = fs::read_to_string(&file).unwrap();
        for name in NEVER {
            assert!(!code.contains(name), "{} names {name}", file.display());
        }
    }
}
