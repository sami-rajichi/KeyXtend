//! `peek` reads the settings file for a later copy of the app and never changes, repairs or sets it aside.

#![allow(clippy::unwrap_used, reason = "a failed setup should fail the test")]

use kx_settings::{Files, peek, save};
use kx_test_support::tempdir::TempDir;

const GOOD: &str = "[keyboard]\nversion = 1\nsize = 3\n";
const BAD_TOML: &[u8] = b"[keyboard\nsize = = 3\n";

/// A temp data folder and its settings files.
fn setup(label: &str) -> (TempDir, Files) {
    let dir = TempDir::new(label).unwrap();
    let files = Files::new(dir.path());
    (dir, files)
}

#[test]
fn a_good_file_is_read_and_nothing_is_written() {
    let (_dir, files) = setup("peek-good");
    save(&files, GOOD).unwrap();
    assert_eq!(peek(&files), Some(GOOD.parse().unwrap()));
    assert!(!files.previous.exists() && !files.broken.exists());
}

#[test]
fn a_missing_or_damaged_file_gives_nothing_and_is_left_as_it_was() {
    let (_dir, files) = setup("peek-bad");
    assert_eq!(peek(&files), None, "missing");
    std::fs::write(&files.settings, BAD_TOML).unwrap();
    assert_eq!(peek(&files), None, "damaged");
    assert_eq!(
        std::fs::read(&files.settings).unwrap(),
        BAD_TOML,
        "left as it was"
    );
    assert!(!files.broken.exists(), "not set aside");
}
