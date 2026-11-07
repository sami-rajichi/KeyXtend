//! A scan of the crate's sources: no call removes every handler in the process, and each handler added or removed
//! goes through `alone`, so two watches never do it at once.

use std::fs;
use std::path::{Path, PathBuf};

/// The calls that add or remove a UI Automation handler of ours.
const CALLS: [&str; 4] = [
    "AddFocusChangedEventHandler(",
    "RemoveFocusChangedEventHandler(",
    "AddAutomationEventHandler(",
    "RemoveAutomationEventHandler(",
];
/// The call that removes every handler in the process, the field watch's too.
const EVERY: &str = "RemoveAllEventHandlers";
/// The process-wide lock's call.
const ALONE: &str = "alone(";

/// Every Rust file under `dir`, but this one, which names the calls it looks for.
fn sources(dir: &Path, out: &mut Vec<PathBuf>) {
    for entry in fs::read_dir(dir).unwrap().map(Result::unwrap) {
        let path = entry.path();
        if path.is_dir() {
            sources(&path, out);
        } else if path.extension().is_some_and(|e| e == "rs") && !path.ends_with(here()) {
            out.push(path);
        }
    }
}

/// This file, from the crate's `src`.
fn here() -> PathBuf {
    ["uia", "tests", "handlers.rs"].iter().collect()
}

#[test]
fn no_source_removes_every_handler_and_each_add_or_remove_takes_the_lock() {
    let mut files = Vec::new();
    sources(
        &Path::new(env!("CARGO_MANIFEST_DIR")).join("src"),
        &mut files,
    );
    let mut calls = 0;
    for path in &files {
        let text = fs::read_to_string(path).unwrap();
        assert!(
            !text.contains(EVERY),
            "{} removes every handler",
            path.display()
        );
        let made = CALLS.iter().map(|c| text.matches(c).count()).sum::<usize>();
        let locked = text.matches(ALONE).count();
        assert!(
            made <= locked,
            "{}: {made} calls, {locked} locked",
            path.display()
        );
        calls += made;
    }
    assert!(
        calls >= CALLS.len(),
        "the scan finds the watches' own calls"
    );
}
