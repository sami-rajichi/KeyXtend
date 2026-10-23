//! The shortcuts the tests' platform knows, as the Windows table lists them: each step's key names, the last tapped.

#![allow(
    clippy::unwrap_used,
    reason = "these fixtures run only in tests, where a failed setup should fail the test"
)]

use kx_module_api::ports::{KeyName, ShortcutId, ShortcutInfo, Step};

/// One shortcut: its id and its steps.
type Entry = (&'static str, &'static [&'static [&'static str]]);

/// Every shortcut of the owner's three pages, with the steps Windows sends.
const TABLE: [Entry; 37] = [
    ("copy", &[&["ctrl", "c"]]),
    ("paste", &[&["ctrl", "v"]]),
    ("cut", &[&["ctrl", "x"]]),
    ("select-all", &[&["ctrl", "a"]]),
    ("undo", &[&["ctrl", "z"]]),
    ("redo", &[&["ctrl", "y"]]),
    ("copy-all", &[&["ctrl", "a"], &["ctrl", "c"]]),
    ("cut-all", &[&["ctrl", "a"], &["ctrl", "x"]]),
    ("delete-all", &[&["ctrl", "a"], &["del"]]),
    ("find", &[&["ctrl", "f"]]),
    ("save", &[&["ctrl", "s"]]),
    ("print", &[&["ctrl", "p"]]),
    ("emoji", &[&["win", "period"]]),
    ("switch-app", &[&["ctrl", "alt", "tab"]]),
    ("show-desktop", &[&["win", "d"]]),
    ("file-explorer", &[&["win", "e"]]),
    ("lock-pc", &[&["win", "l"]]),
    ("task-manager", &[&["ctrl", "shift", "esc"]]),
    ("close-window", &[&["alt", "f4"]]),
    ("new-tab", &[&["ctrl", "t"]]),
    ("close-tab", &[&["ctrl", "w"]]),
    ("reopen-tab", &[&["ctrl", "shift", "t"]]),
    ("refresh", &[&["f5"]]),
    ("zoom-in", &[&["ctrl", "numplus"]]),
    ("zoom-out", &[&["ctrl", "numminus"]]),
    ("f1", &[&["f1"]]),
    ("f2", &[&["f2"]]),
    ("f3", &[&["f3"]]),
    ("f4", &[&["f4"]]),
    ("f5", &[&["f5"]]),
    ("f6", &[&["f6"]]),
    ("f7", &[&["f7"]]),
    ("f8", &[&["f8"]]),
    ("f9", &[&["f9"]]),
    ("f10", &[&["f10"]]),
    ("f11", &[&["f11"]]),
    ("f12", &[&["f12"]]),
];

/// Shortcut `id`.
pub fn sid(id: &str) -> ShortcutId {
    ShortcutId::new(id).unwrap()
}

/// One step from its key names, the last tapped.
fn step(names: &[&str]) -> Step {
    let mut keys: Vec<KeyName> = names.iter().map(|n| KeyName::new(n).unwrap()).collect();
    let key = keys.pop().unwrap();
    Step { held: keys, key }
}

/// Every shortcut in the table, in its order.
pub fn all() -> Vec<ShortcutInfo> {
    only(&TABLE.map(|(id, _)| id))
}

/// The shortcuts `ids` of the table, in the table's order.
pub fn only(ids: &[&str]) -> Vec<ShortcutInfo> {
    let kept = TABLE.iter().filter(|(id, _)| ids.contains(id));
    let info = |(id, steps): &Entry| ShortcutInfo {
        id: sid(id),
        steps: steps.iter().map(|s| step(s)).collect(),
    };
    kept.map(info).collect()
}
