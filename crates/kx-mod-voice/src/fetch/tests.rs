//! Downloads (plan task 19): progress, Pause, Resume and Cancel on the row, a bad fingerprint's note (edge 20), and no
//! download with a note for a full or missing drive, a read-only folder or `min_free_mb` not met (edge 21).

use super::*;
use crate::models::table;
use kx_module_api::Redacted;
use kx_module_api::ports::{Fingerprint, FolderScan, MB, ModelFile};
use std::path::{Path, PathBuf};

/// The folder the tests use.
const DIR: &str = r"D:\KeyXtend\models";

fn parakeet() -> &'static Row {
    table().unwrap().get("parakeet-v3-q8").unwrap()
}

fn settings() -> VoiceSettings {
    VoiceSettings::from_defaults().unwrap()
}

/// The folder's drive with `free` bytes free.
fn drive(free: u64) -> [Drive; 1] {
    [Drive {
        root: Redacted::new(PathBuf::from(r"D:\")),
        name: Redacted::new("D:".into()),
        free,
    }]
}

/// Room for Parakeet beside `min_free_mb`.
fn roomy() -> [Drive; 1] {
    drive(parakeet().bytes + u64::from(settings().min_free_mb) * MB)
}

/// A download of Parakeet that started.
fn started() -> Fetches {
    let mut fetches = Fetches::default();
    fetches
        .start(parakeet(), &Folder::new(DIR), &roomy(), &settings())
        .unwrap();
    fetches
}

#[test]
fn the_ask_fetches_the_rows_file_into_the_folder() {
    let (row, s) = (parakeet(), settings());
    let ask = Fetches::default()
        .start(row, &Folder::new(DIR), &roomy(), &s)
        .unwrap();
    assert_eq!(ask.file.expose(), &Path::new(DIR).join(&row.file));
    assert_eq!(
        (ask.bytes, ask.print, &ask.hosts),
        (row.bytes, row.print, &row.hosts)
    );
    assert_eq!(
        (ask.min_free_mb, ask.stall_s, ask.job),
        (s.min_free_mb, s.stall_s, s.job())
    );
    assert_eq!(ask.links.len(), row.links.len());
}

#[test]
fn a_download_shows_its_progress_and_pauses_resumes_and_cancels() {
    let mut fetches = started();
    let id = &parakeet().id;
    assert!(fetches.busy());
    let total = parakeet().bytes;
    let moved = fetches.note(id, FetchNote::Progress { done: 9, total });
    assert_eq!(moved, Fetched::Moved);
    let shown = |f: &Fetches| f.state(id).map(|s| (s.done, s.total, s.paused));
    assert_eq!(shown(&fetches), Some((9, total, false)));
    assert_eq!(
        fetches.press(id, FetchButton::Resume),
        None,
        "it is not paused"
    );
    assert_eq!(
        fetches.press(id, FetchButton::Pause),
        Some(FetchButton::Pause)
    );
    assert_eq!(shown(&fetches), Some((9, total, true)));
    assert_eq!(
        fetches.press(id, FetchButton::Resume),
        Some(FetchButton::Resume)
    );
    assert_eq!(
        fetches.press(id, FetchButton::Cancel),
        Some(FetchButton::Cancel)
    );
    assert_eq!((fetches.state(id), fetches.busy()), (None, false));
    let late = FetchNote::Fault(ModelError::Cancelled);
    assert_eq!(
        fetches.note(id, late),
        Fetched::Stale,
        "a late note is dropped"
    );
}

#[test]
fn a_finished_download_ends_its_run() {
    let mut fetches = started();
    assert_eq!(fetches.note(&parakeet().id, FetchNote::Done), Fetched::Done);
    assert!(!fetches.busy());
    assert_eq!(fetches.note("other", FetchNote::Done), Fetched::Stale);
}

#[test]
fn a_bad_fingerprint_or_size_tells_a_note() {
    for error in [ModelError::Print, ModelError::Size] {
        let mut fetches = started();
        let got = fetches.note(&parakeet().id, FetchNote::Fault(error));
        assert_eq!(got, Fetched::Failed(BarHint::Print), "{error:?}");
        assert!(!fetches.busy());
    }
    let mut cancelled = started();
    let got = cancelled.note(&parakeet().id, FetchNote::Fault(ModelError::Cancelled));
    assert_eq!(got, Fetched::Stopped, "a cancel needs no note");
}

#[test]
fn a_missing_drive_or_too_little_room_starts_no_download() {
    let (row, s, folder) = (parakeet(), settings(), Folder::new(DIR));
    let mut fetches = Fetches::default();
    assert_eq!(
        fetches.start(row, &folder, &[], &s),
        Err(BarHint::Folder),
        "a missing drive"
    );
    let short = drive(row.bytes + u64::from(s.min_free_mb) * MB - 1);
    assert_eq!(
        fetches.start(row, &folder, &short, &s),
        Err(BarHint::Full),
        "min_free_mb not met"
    );
    let none = Folder::new("");
    assert_eq!(
        fetches.start(row, &none, &roomy(), &s),
        Err(BarHint::Folder),
        "no folder"
    );
    assert!(!fetches.busy());
}

#[test]
fn a_name_another_file_holds_starts_no_download() {
    let row = parakeet();
    let mut folder = Folder::new(DIR);
    let file = ModelFile {
        name: Redacted::new(row.file.clone()),
        bytes: row.bytes,
    };
    folder.scanned(Ok(FolderScan {
        folders: Vec::new(),
        files: vec![file],
    }));
    folder.printed(
        &Path::new(DIR).join(&row.file),
        row.bytes,
        Ok(Fingerprint::new([7; Fingerprint::BYTES])),
    );
    let mut fetches = Fetches::default();
    assert_eq!(
        fetches.start(row, &folder, &roomy(), &settings()),
        Err(BarHint::Taken),
        "the file in the way has its own note"
    );
    assert!(!fetches.busy());
}

#[test]
fn a_full_drive_or_a_read_only_folder_from_the_job_tells_its_note() {
    let cases = [
        (ModelError::Full, BarHint::Full),
        (ModelError::ReadOnly, BarHint::Folder),
        (ModelError::Missing, BarHint::Folder),
        (ModelError::Host, BarHint::Net),
        (ModelError::Stalled, BarHint::Net),
    ];
    for (error, want) in cases {
        assert_eq!(hint(error), Some(want.clone()), "{error:?}");
        let mut fetches = started();
        let got = fetches.note(&parakeet().id, FetchNote::Fault(error));
        assert_eq!(got, Fetched::Failed(want));
    }
    assert_eq!(hint(ModelError::Cancelled), None);
}
