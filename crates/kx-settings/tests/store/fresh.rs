//! A fresh store, for a run that keeps nothing (ADR-0037): the defaults, no file read, and saves that write nothing.

#![allow(
    clippy::unwrap_used,
    reason = "the snapshot helper runs only in tests, where a failed read should fail the test"
)]

use crate::spec::{DEFAULTS, HOLD, SPEC, defaults_with, folder, table, with_file};
use kx_settings::Store;
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use toml::Value;

/// A file with a change and a bad value, which a read would apply and repair.
const FILE: &str = "[hold]\nversion = 3\nmax_ms = 900\nsound = \"loud\"\n";

/// Every file under `dir` with its bytes.
fn snapshot(dir: &Path) -> BTreeMap<PathBuf, Vec<u8>> {
    let entries = fs::read_dir(dir).unwrap().flatten();
    let files = entries.map(|e| e.path()).filter(|p| p.is_file());
    files.map(|p| (p.clone(), fs::read(&p).unwrap())).collect()
}

#[test]
fn a_fresh_store_holds_the_defaults_and_reads_no_file_even_when_one_exists() {
    let (_dir, _files) = with_file("fresh-read", FILE);
    let (store, report) = Store::fresh(&[(HOLD, &SPEC)]);
    assert_eq!(store.get(HOLD), Some(&table(DEFAULTS)));
    assert!(
        report.notices.is_empty(),
        "the file's bad value is never seen"
    );
    assert!(report.broken.is_empty() && !store.needs_save());
}

#[test]
fn a_fresh_store_keeps_a_change_in_memory_and_its_save_writes_nothing() {
    let (dir, files) = with_file("fresh-save", FILE);
    let before = snapshot(dir.path());
    let (mut store, _) = Store::fresh(&[(HOLD, &SPEC)]);
    assert!(store.set(HOLD, "max_ms", Value::Integer(800)).unwrap());
    assert_eq!(store.get(HOLD), Some(&defaults_with("max_ms = 800")));
    assert!(!store.needs_save(), "nothing is due on disk");
    store.save().unwrap();
    assert_eq!(snapshot(dir.path()), before, "the folder stays as it was");
    assert_eq!(fs::read_to_string(files.settings).unwrap(), FILE);
}

#[test]
fn a_fresh_store_makes_no_file_in_an_empty_folder() {
    let (dir, _files) = folder("fresh-empty");
    let (mut store, _) = Store::fresh(&[(HOLD, &SPEC)]);
    store.set(HOLD, "sound", Value::Boolean(false)).unwrap();
    store.save().unwrap();
    assert_eq!(fs::read_dir(dir.path()).unwrap().count(), 0);
}
