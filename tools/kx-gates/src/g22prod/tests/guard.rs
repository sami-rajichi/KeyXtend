//! The gate copies a sealed key's bytes and never opens one: no source names the key guard or a shelf that unseals,
//! and it takes only the sealed files' names from kx-crypto. The needles are built in parts, so this file holds none.

use std::fs;
use std::path::Path;

/// Each Rust source under `dir`, as its path and text.
fn sources(dir: &Path, all: &mut Vec<(String, String)>) {
    for entry in fs::read_dir(dir).expect("a source folder") {
        let path = entry.expect("an entry").path();
        if path.is_dir() {
            sources(&path, all);
        } else if path.extension().is_some_and(|e| e == "rs") {
            let text = fs::read_to_string(&path).expect("a source");
            all.push((path.display().to_string(), text));
        }
    }
}

/// Every kx-gates source.
fn all() -> Vec<(String, String)> {
    let mut all = Vec::new();
    sources(&Path::new(env!("CARGO_MANIFEST_DIR")).join("src"), &mut all);
    assert!(all.len() > 100, "the sources were found");
    all
}

#[test]
fn no_gate_source_names_the_sealing_guard_or_a_shelf_that_unseals() {
    let needles = [
        ["Key", "Guard"].concat(),
        ["Sealed", "Shelf"].concat(),
        ["Crypt", "Unprotect"].concat(),
        ["guard", "_key"].concat(),
        ["key", "_guard"].concat(),
    ];
    for (file, text) in all() {
        for needle in &needles {
            assert!(!text.contains(needle.as_str()), "{file} names {needle}");
        }
    }
}

#[test]
fn the_gate_takes_only_the_sealed_files_names_from_kx_crypto() {
    let (path, names) = (["kx", "_crypto::"].concat(), "{KEY_FILE, SEALED_EXT}");
    let mut uses = 0;
    for (file, text) in all() {
        for (at, _) in text.match_indices(&path) {
            assert!(text[at + path.len()..].starts_with(names), "{file}");
            uses += 1;
        }
    }
    assert!(uses > 0, "the borrow names them");
}
