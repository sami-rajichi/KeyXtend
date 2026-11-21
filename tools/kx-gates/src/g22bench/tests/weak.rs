//! The weak worker's folder: only the plain processor parts, never the card part, and no path in a refusal.

use kx_test_support::tempdir::TempDir;

use super::super::spawn::{only, plain_parts};

/// A card part's name, which no processor part shares.
const CARD: &str = "engine-card.dll";
/// The processor parts' name prefix.
const PREFIX: &str = "part-";

/// `list` as owned names.
fn names(list: &[&str]) -> Vec<String> {
    list.iter().map(|n| (*n).to_owned()).collect()
}

#[test]
fn a_weak_workers_folder_holds_only_the_plain_parts() {
    let weak = names(&["part-x64.dll", "part-sse42.dll"]);
    let plain = names(&["keyxtend-worker.exe", "engine.dll", "PART-SSE42.DLL"]);
    assert_eq!(only(&plain, PREFIX, &weak, CARD), Ok(()));
    let fast = names(&["keyxtend-worker.exe", "part-x64.dll", "part-haswell.dll"]);
    assert!(only(&fast, PREFIX, &weak, CARD).is_err(), "a part with AVX");
    let none = names(&["keyxtend-worker.exe", "engine.dll"]);
    assert!(only(&none, PREFIX, &weak, CARD).is_err(), "no part at all");
}

#[test]
fn a_weak_workers_folder_made_from_the_card_build_leaves_the_card_part_out() {
    let weak = names(&["part-x64.dll"]);
    let carded = names(&["keyxtend-worker.exe", "part-x64.dll", "ENGINE-CARD.DLL"]);
    let e = only(&carded, PREFIX, &weak, CARD).expect_err("the card part");
    assert!(e.contains(CARD), "{e}");
    let left_out = names(&["keyxtend-worker.exe", "part-x64.dll"]);
    assert_eq!(only(&left_out, PREFIX, &weak, CARD), Ok(()));
}

#[test]
fn a_weak_folder_that_holds_a_wrong_part_or_cannot_be_read_names_no_path() {
    let dir = TempDir::new("kx-gates-weak").expect("a folder");
    std::fs::write(dir.path().join("part-haswell.dll"), b"stand-in").expect("written");
    let weak = names(&["part-x64.dll"]);
    let shown = dir.path().display().to_string();
    let workers = [
        dir.path().join("w.exe"),
        dir.path().join("gone").join("w.exe"),
    ];
    for worker in workers {
        let e = plain_parts(&worker, PREFIX, &weak, CARD).expect_err("refused");
        assert!(!e.contains(&shown), "{e}");
    }
}
