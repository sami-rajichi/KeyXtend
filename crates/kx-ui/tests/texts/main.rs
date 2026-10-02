//! The shipped English, French and Arabic files: the same keys, every text the UI names, plurals and punctuation, the
//! QML test fixtures made from them, the window titles and flags the guard relies on, and plain-text QML.

#![allow(
    clippy::expect_used,
    reason = "these helpers run only in tests, where a failed setup should fail the test"
)]

#[path = "../shared/board.rs"]
mod board;
mod compose;
mod files;
mod markup;
mod plural;
mod qml;
mod windows;
mod words;

use kx_ui::{I18n, LANGS};
use std::path::Path;

/// The QML folder, under this crate.
const QML: &str = "qml";

/// Every QML file of the UI, by name, with its text.
fn qml_files() -> Vec<(String, String)> {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join(QML);
    let files = std::fs::read_dir(dir).expect("the QML folder reads");
    files
        .flatten()
        .filter(|f| f.path().extension().is_some_and(|e| e == QML))
        .map(|f| {
            let text = std::fs::read_to_string(f.path()).expect("a QML file reads");
            (f.file_name().to_string_lossy().into_owned(), text)
        })
        .collect()
}

/// The keyboard module's `[keyboard]` defaults, whose sizes the top bar and the QML fixtures follow.
const KEYBOARD_DEFAULTS: &str = "../kx-mod-keyboard/defaults.toml";

/// The keyboard module's `[keyboard]` defaults.
fn keyboard_defaults() -> toml::Table {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(KEYBOARD_DEFAULTS);
    let text = std::fs::read_to_string(path).expect("the keyboard defaults read");
    toml::from_str(&text).expect("the keyboard defaults parse")
}

/// Fluent's marks around each argument, which keep Latin words in order inside Arabic text.
const ISOLATE: [char; 2] = ['\u{2068}', '\u{2069}'];

/// `text` without Fluent's isolation marks.
fn plain(text: &str) -> String {
    text.chars().filter(|c| !ISOLATE.contains(c)).collect()
}

/// The texts of every shipped language.
fn all() -> impl Iterator<Item = I18n> {
    LANGS.iter().map(I18n::new)
}

/// The texts of shipped language `tag`.
fn lang(tag: &str) -> I18n {
    I18n::new(kx_ui::find(tag).expect("a shipped language"))
}

/// `names` as a notice list.
fn items(names: &[&str]) -> Vec<String> {
    names.iter().map(|n| (*n).to_owned()).collect()
}
