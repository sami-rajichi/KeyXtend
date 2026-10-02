//! Our windows as the guard finds them: each title names one kind of window in every language, and every window file
//! asks Qt to start on top.

use crate::all;
use kx_ui::i18n::keys::{
    CATCH, WINDOW_BUBBLE, WINDOW_KEYBOARD, WINDOW_LIST, WINDOW_MENU, WINDOW_NOTICE, WINDOW_TIP,
};
use std::collections::BTreeMap;
use std::path::Path;

/// Every window title the UI names; the guard, `shape`, the backdrops, a tip's lift and UI Automation find windows by them.
pub const TITLES: [&str; 7] = [
    WINDOW_KEYBOARD,
    WINDOW_BUBBLE,
    WINDOW_NOTICE,
    WINDOW_MENU,
    WINDOW_LIST,
    WINDOW_TIP,
    CATCH,
];
/// The QML folder, under this crate.
const QML: &str = "qml";
/// What may come before a file's root type: comments, pragmas and imports.
const HEAD: [&str; 3] = ["//", "pragma ", "import "];
/// A root type that makes a window.
const WINDOW_ROOT: &str = "Window {";
/// What starts a window's flags.
const FLAGS: &str = "flags:";
/// The flag the guard needs there (kx-platform-windows `window`).
const ON_TOP: &str = "Qt.WindowStaysOnTopHint";
/// A window file the check must find, so a broken root reader cannot pass it.
const KEYBOARD: &str = "KbWindow.qml";

/// The root type's line of `qml`: its first line that is not blank or a head line.
fn root(qml: &str) -> Option<&str> {
    let head = |l: &str| l.is_empty() || HEAD.iter().any(|h| l.starts_with(h));
    qml.lines().map(str::trim).find(|l| !head(l))
}

/// The tray menu speaks the system's language and the other windows the keyboard's, so titles differ across languages
/// too; a tip's title names both tips.
#[test]
fn each_window_title_names_one_kind_of_window_in_every_language() {
    let mut seen: BTreeMap<String, &str> = BTreeMap::new();
    for t in all() {
        for key in TITLES {
            let was = seen.insert(t.text(key), key);
            let tag = t.lang().tag;
            assert!(
                was.is_none_or(|k| k == key),
                "{tag}: {key} reads as {was:?}"
            );
        }
    }
}

/// The guard keeps a window topmost only when it starts so; the Settings window it spares will need its own rule here.
#[test]
fn every_window_file_starts_on_top() {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join(QML);
    let mut found = Vec::new();
    for f in std::fs::read_dir(dir)
        .expect("the QML folder reads")
        .flatten()
    {
        let text = std::fs::read_to_string(f.path()).expect("a QML file reads");
        if root(&text) != Some(WINDOW_ROOT) {
            continue;
        }
        let name = f.file_name().to_string_lossy().into_owned();
        let flags = text.lines().map(str::trim).find(|l| l.starts_with(FLAGS));
        assert!(
            flags.is_some_and(|l| l.contains(ON_TOP)),
            "{name} starts on top"
        );
        found.push(name);
    }
    assert!(found.iter().any(|n| n == KEYBOARD), "found {found:?}");
}
