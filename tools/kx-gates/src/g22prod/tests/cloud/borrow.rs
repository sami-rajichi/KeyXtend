//! A borrowed key: a daily folder reads as a data folder, and a key is copied byte for byte and nothing else, or not
//! at all, with no path in an error.

use std::fs;
use std::path::Path;

use kx_crypto::{KEY_FILE, SEALED_EXT};
use kx_mod_voice::keys::KEYS;
use kx_platform::VOICE_DIR;
use kx_settings::Files;
use kx_test_support::tempdir::TempDir;

use super::super::super::borrow;
use super::{BLOB, GROQ, WRAPPED, daily};

/// A made-up service id the daily folder holds no key for.
const KEYLESS: &str = "keyless";

/// How many files lie under `dir`.
fn files(dir: &Path) -> usize {
    let entries = fs::read_dir(dir)
        .expect("a folder")
        .map(|e| e.expect("an entry").path());
    entries
        .map(|p| if p.is_dir() { files(&p) } else { 1 })
        .sum()
}

#[test]
fn a_daily_folder_is_a_data_folder_only_with_its_settings_or_its_voice_data_key() {
    let d = daily(&[]);
    assert!(borrow::is_data(d.path()));
    fs::remove_file(Files::new(d.path()).settings).expect("removed");
    assert!(borrow::is_data(d.path()), "the data key alone");
    fs::remove_file(d.path().join(VOICE_DIR).join(KEY_FILE)).expect("removed");
    assert!(!borrow::is_data(d.path()), "neither");
    let empty = TempDir::new("g22-mistyped").expect("a folder");
    assert!(!borrow::is_data(empty.path()), "an empty folder");
    assert!(!borrow::is_data(&empty.path().join("none")), "no folder");
}

#[test]
fn a_key_is_there_only_with_its_blob_and_the_data_key_by_their_names() {
    let d = daily(&[GROQ]);
    assert!(borrow::has(d.path(), GROQ));
    assert!(!borrow::has(d.path(), KEYLESS), "no blob");
    assert!(!borrow::has(d.path(), "a/b"), "no blob name");
    fs::remove_file(d.path().join(VOICE_DIR).join(KEY_FILE)).expect("removed");
    assert!(!borrow::has(d.path(), GROQ), "no data key");
}

#[test]
fn a_borrow_copies_that_blob_and_the_data_key_byte_for_byte_and_nothing_else() {
    let d = daily(&[GROQ, KEYLESS]);
    let copy = TempDir::new("g22-copy").expect("a folder");
    let made = borrow::copy(d.path(), copy.path(), GROQ).expect("copied");
    let voice = copy.path().join(VOICE_DIR);
    assert_eq!(fs::read(voice.join(KEY_FILE)).expect("copied"), WRAPPED);
    let blob = voice.join(KEYS).join(format!("{GROQ}.{SEALED_EXT}"));
    assert_eq!(fs::read(&blob).expect("copied"), BLOB);
    assert_eq!(files(copy.path()), 2, "only those two");
    assert_eq!(made, [voice.join(KEY_FILE), blob], "each named, to be lent");
}

#[test]
fn a_failed_borrow_names_no_path_and_no_folder_is_copied_onto_itself() {
    let d = daily(&[GROQ]);
    let copy = TempDir::new("g22-copy").expect("a folder");
    let e = borrow::copy(d.path(), copy.path(), KEYLESS).expect_err("no blob");
    for path in [d.path(), copy.path()] {
        assert!(!e.contains(&path.display().to_string()), "{e}");
    }
    assert!(
        borrow::copy(d.path(), d.path(), GROQ).is_err(),
        "its own folder"
    );
    assert_eq!(files(d.path()), 4, "the daily folder is as it was");
}

#[test]
fn a_borrow_missing_either_file_copies_nothing() {
    let d = daily(&[GROQ]);
    let copy = TempDir::new("g22-copy").expect("a folder");
    assert!(
        borrow::copy(d.path(), copy.path(), KEYLESS).is_err(),
        "no blob"
    );
    assert_eq!(files(copy.path()), 0, "not even the data key");
    fs::remove_file(d.path().join(VOICE_DIR).join(KEY_FILE)).expect("removed");
    assert!(
        borrow::copy(d.path(), copy.path(), GROQ).is_err(),
        "no data key"
    );
    assert_eq!(files(copy.path()), 0, "not even the blob");
}
