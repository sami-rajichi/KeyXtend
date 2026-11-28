//! `write_kept` leaves our markers off, so the system's history keeps what it writes (ADR-0056 decision 2): of the
//! modules, the ui and the apps, only Snip calls it, so no other module's text can reach that history this way.

use kx_test_support::source::rust_files;
use std::fs;
use std::path::{Path, PathBuf};

/// The call this guard pins.
const CALL: &str = ".write_kept(";
/// The one crate that makes it.
const ONLY: &str = "kx-mod-snip";

/// The source folders of each module, the ui and each app under the workspace `root`.
fn callers(root: &Path) -> Vec<PathBuf> {
    let crates = fs::read_dir(root.join("crates")).into_iter().flatten();
    let apps = fs::read_dir(root.join("apps")).into_iter().flatten();
    let named = |name: &str| name.starts_with("kx-mod-") || name == "kx-ui";
    let modules = crates
        .flatten()
        .filter(|e| named(&e.file_name().to_string_lossy()));
    modules
        .chain(apps.flatten())
        .map(|e| e.path().join("src"))
        .collect()
}

#[test]
fn only_snip_writes_to_the_clipboard_without_our_markers() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let mut seen = false;
    for src in callers(&root) {
        let from_snip = src.parent().is_some_and(|c| c.ends_with(ONLY));
        for file in rust_files(&src).unwrap() {
            let calls = fs::read_to_string(&file).unwrap().contains(CALL);
            assert!(!calls || from_snip, "{} calls write_kept", file.display());
            seen |= calls && from_snip;
        }
    }
    assert!(seen, "the walk reached Snip's call");
}
