//! A scan of the crate's sources: no file but `uia/handlers.rs` calls a UI Automation method that adds or removes an
//! event handler, each such call there holds the lock, and no file names the method that removes every handler.

use std::fs;
use std::path::{Path, PathBuf};

/// The part every UI Automation handler method's name holds.
const MARK: &str = "EventHandler";
/// The starts of the names of the methods that add or remove one.
const VERBS: [&str; 2] = ["Add", "Remove"];
/// The method that removes every handler in the process, the field watch's too; built in two, so this file never
/// names it.
const EVERY: &str = concat!("RemoveAll", "EventHandlers");
/// Where the locked helpers live, from the crate's `src`.
const HELPERS: [&str; 2] = ["uia", "handlers.rs"];
/// How a call under the lock begins.
const LOCKED: &str = "alone(||";

/// Every Rust file under `dir`.
fn sources(dir: &Path, out: &mut Vec<PathBuf>) {
    for entry in fs::read_dir(dir).unwrap().map(Result::unwrap) {
        let path = entry.path();
        if path.is_dir() {
            sources(&path, out);
        } else if path.extension().is_some_and(|e| e == "rs") {
            out.push(path);
        }
    }
}

/// Whether `c` can be part of a Rust name.
fn word(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '_'
}

/// Every name in `text` that holds `MARK`, and whether a call follows it.
fn names(text: &str) -> Vec<(&str, bool)> {
    let mut found = Vec::new();
    for (at, _) in text.match_indices(MARK) {
        let before = text[..at].char_indices().rev().find(|(_, c)| !word(*c));
        let start = before.map_or(0, |(i, c)| i + c.len_utf8());
        let end = text[at..].find(|c| !word(c)).map_or(text.len(), |i| at + i);
        found.push((&text[start..end], text[end..].starts_with('(')));
    }
    found
}

/// How many calls in `text` add or remove a handler, after failing on any name of `EVERY`.
fn calls(text: &str, path: &Path) -> usize {
    let mut made = 0;
    for (name, call) in names(text) {
        assert_ne!(name, EVERY, "{} removes every handler", path.display());
        made += usize::from(call && VERBS.iter().any(|v| name.starts_with(v)));
    }
    made
}

#[test]
fn the_name_scan_finds_calls_whole_names_and_nothing_else() {
    // Built with `MARK`, so the scan of this file finds no call in it.
    let text = format!("a.AddFocusChanged{MARK}(x); IUIAutomation{MARK}; éRemoveX{MARK}Y (");
    let want = [
        ("AddFocusChangedEventHandler", true),
        ("IUIAutomationEventHandler", false),
        ("RemoveXEventHandlerY", false),
    ];
    assert_eq!(names(&text), want);
    assert_eq!(calls(&text, Path::new("x")), 1);
}

#[test]
fn only_the_locked_helpers_add_or_remove_a_handler_and_none_removes_every_one() {
    let mut files = Vec::new();
    sources(
        &Path::new(env!("CARGO_MANIFEST_DIR")).join("src"),
        &mut files,
    );
    let helpers: PathBuf = HELPERS.iter().collect();
    let mut theirs = 0;
    for path in &files {
        let text = fs::read_to_string(path).unwrap();
        let made = calls(&text, path);
        if path.ends_with(&helpers) {
            let locked = text.matches(LOCKED).count();
            assert!(made > 0 && made <= locked, "{made} calls, {locked} locked");
            theirs = made;
        } else {
            assert_eq!(made, 0, "{} adds or removes a handler", path.display());
        }
    }
    assert!(theirs > 0, "the helpers are found");
}
