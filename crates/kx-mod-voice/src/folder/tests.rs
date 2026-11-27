//! The models folder (plan task 19): a file is known by its fingerprint, the folder offered at the first download and
//! the refused paths (edge 38), and a missing drive or too little room (edge 21).

use super::*;
use kx_module_api::ports::{ModelFacts, ModelFile, SpeechError};
use kx_test_support::paths::{native, native_path};

mod supply;

/// The folder the tests use, in this system's form.
fn dir() -> String {
    native(r"D:\KeyXtend\models")
}

/// The row of `id`.
fn row(id: &str) -> &'static Row {
    table().unwrap().get(id).unwrap()
}

/// A fixed drive at `root`, written for Windows, with `free` bytes free.
fn drive(root: &str, free: u64) -> Drive {
    Drive {
        root: Redacted::new(native_path(root)),
        name: Redacted::new(root.to_owned()),
        free,
    }
}

/// A scan finding `files`, each a name and a size.
fn scan(files: &[(&str, u64)]) -> FolderScan {
    let file = |(name, bytes): &(&str, u64)| ModelFile {
        name: Redacted::new((*name).to_owned()),
        bytes: *bytes,
    };
    FolderScan {
        folders: Vec::new(),
        files: files.iter().map(file).collect(),
    }
}

/// The folder at `dir` after a scan finding `files`.
fn scanned(files: &[(&str, u64)]) -> Folder {
    let mut folder = Folder::new(&dir());
    folder.scanned(Ok(scan(files)));
    folder
}

/// File `name` in `dir`, as `scanned` gives it.
fn at(name: &str) -> PathBuf {
    Path::new(&dir()).join(name)
}

/// Another fingerprint than any row's.
fn stranger() -> Fingerprint {
    Fingerprint::new([9; Fingerprint::BYTES])
}

/// The facts an other file tells.
fn facts(langs: &[&str]) -> ModelFacts {
    ModelFacts {
        runner: "transcribe".into(),
        langs: langs.iter().map(|l| (*l).to_owned()).collect(),
    }
}

#[test]
fn a_file_already_in_the_folder_is_known_by_its_fingerprint() {
    let parakeet = row("parakeet-v3-q8");
    let mut folder = Folder::new(&dir());
    let read = folder.scanned(Ok(scan(&[("my copy.gguf", parakeet.bytes)])));
    assert_eq!(read, [(at("my copy.gguf"), parakeet.bytes)]);
    assert_eq!(folder.of(parakeet), None, "unread, and not under its name");
    let other = folder.printed(&at("my copy.gguf"), parakeet.bytes, Ok(parakeet.print));
    assert_eq!(other, None, "a row's file needs no facts");
    assert_eq!(
        folder.of(parakeet).map(|f| f.name.as_str()),
        Some("my copy.gguf")
    );
    assert_eq!(folder.others().count(), 0);
}

#[test]
fn a_rows_name_with_another_fingerprint_is_an_other_file() {
    let parakeet = row("parakeet-v3-q8");
    let mut folder = scanned(&[(&parakeet.file, parakeet.bytes)]);
    let other = folder.printed(&at(&parakeet.file), parakeet.bytes, Ok(stranger()));
    assert_eq!(other.map(|o| o.name), Some(parakeet.file.clone()));
    assert_eq!(folder.of(parakeet), None);
    let names: Vec<&str> = folder.others().map(|f| f.name.as_str()).collect();
    assert_eq!(names, [parakeet.file.as_str()]);
    let taken = folder.taken(parakeet).map(|f| f.name.as_str());
    assert_eq!(
        taken,
        Some(parakeet.file.as_str()),
        "its download waits for Delete file"
    );
    let own = scanned(&[(&parakeet.file, parakeet.bytes)]);
    assert_eq!(own.taken(parakeet), None, "its own file, being read");
}

#[test]
fn a_rows_file_by_name_and_size_serves_until_its_fingerprint_is_read() {
    let parakeet = row("parakeet-v3-q8");
    let folder = scanned(&[(&parakeet.file.to_uppercase(), parakeet.bytes)]);
    assert!(folder.reading());
    assert_eq!(folder.of(parakeet).map(|f| f.print), Some(Print::Pending));
    let cut = scanned(&[(&parakeet.file, parakeet.bytes - 1)]);
    assert_eq!(cut.of(parakeet), None, "another size is not its file");
}

#[test]
fn a_new_scan_reads_only_files_it_has_not_read() {
    let small = row("whisper-small-q5km");
    let mut folder = scanned(&[(&small.file, small.bytes)]);
    folder.printed(&at(&small.file), small.bytes, Ok(small.print));
    let again = folder.scanned(Ok(scan(&[(&small.file, small.bytes), ("new.bin", 9)])));
    assert_eq!(again, [(at("new.bin"), 9)]);
    assert!(folder.of(small).is_some());
}

#[test]
fn a_late_fingerprint_for_a_file_no_longer_held_is_dropped() {
    let small = row("whisper-small-q5km");
    let mut folder = scanned(&[(&small.file, small.bytes)]);
    let old = native_path(r"E:\old").join(&small.file);
    assert_eq!(folder.printed(&old, small.bytes, Ok(stranger())), None);
    folder.printed(&at(&small.file), small.bytes - 1, Ok(stranger()));
    assert_eq!(folder.of(small).map(|f| f.print), Some(Print::Pending));
    assert!(folder.reading());
}

#[test]
fn a_file_whose_fingerprint_cannot_be_read_is_left_out() {
    let mut folder = scanned(&[("broken.gguf", 9)]);
    folder.printed(&at("broken.gguf"), 9, Err(ModelError::Io));
    assert!(!folder.reading());
    assert_eq!(folder.others().count(), 0);
}

#[test]
fn a_folder_that_cannot_be_read_holds_nothing() {
    let small = row("whisper-small-q5km");
    let mut folder = scanned(&[(&small.file, small.bytes)]);
    assert!(folder.scanned(Err(ModelError::Missing)).is_empty());
    assert_eq!(folder.of(small), None);
}

#[test]
fn an_other_files_facts_are_kept_by_its_fingerprint() {
    let mut folder = scanned(&[("ggml-medium.bin", 1_530 * MB)]);
    folder.printed(&at("ggml-medium.bin"), 1_530 * MB, Ok(stranger()));
    assert_eq!(folder.facts(&stranger()), None);
    folder.facts_read(stranger(), Ok(facts(&["fr", "en"])));
    assert_eq!(folder.facts(&stranger()), Some(&facts(&["fr", "en"])));
    let mut bad = scanned(&[("odd.bin", 9)]);
    bad.printed(&at("odd.bin"), 9, Ok(stranger()));
    bad.facts_read(stranger(), Err(SpeechError::BadFile));
    assert_eq!(bad.facts(&stranger()), None);
    assert!(
        !bad.wants_facts(&stranger()),
        "an unreadable file is not asked again"
    );
}

#[test]
fn a_finished_download_is_known_at_once() {
    let parakeet = row("parakeet-v3-q8");
    let mut folder = Folder::new(&dir());
    folder.fetched(parakeet);
    assert_eq!(
        folder.of(parakeet).map(|f| f.print),
        Some(Print::Known(parakeet.print))
    );
}

#[test]
fn the_offer_is_keyxtend_models_on_the_drive_with_the_most_room() {
    assert_eq!(Folder::new("").dir(), None);
    assert_eq!(offer(&[]), None, "no drive, no offer");
    let drives = [
        drive(r"C:\", 50 * MB),
        drive(r"E:\", 900 * MB),
        drive(r"D:\", 300 * MB),
    ];
    let (dir, shown) = offer(&drives).unwrap();
    assert_eq!(dir, native_path(r"E:\").join(OFFER[0]).join(OFFER[1]));
    assert_eq!(shown.free, 900 * MB);
    assert!(path_ok(&dir));
}

#[test]
fn a_relative_device_or_climbing_folder_is_refused() {
    let bads = [
        "models",
        r"\\.\C:\models",
        r"D:\KeyXtend\..\models",
        r"D:\NUL\models",
        r"\\?\D:\models",
        "D:\\mod\u{7}els",
    ];
    for bad in bads.map(native) {
        let mut folder = Folder::new(&bad);
        assert_eq!(folder.dir(), None, "{bad}");
        assert!(!folder.set_dir(Path::new(&bad)), "{bad}");
    }
    let mut folder = Folder::new("");
    assert!(folder.set_dir(Path::new(&dir())));
    assert_eq!(folder.dir(), Some(Path::new(&dir())));
}

#[test]
fn a_new_folder_forgets_the_old_ones_files() {
    let small = row("whisper-small-q5km");
    let mut folder = scanned(&[(&small.file, small.bytes)]);
    folder.printed(&at(&small.file), small.bytes, Ok(small.print));
    assert!(!folder.follow(&dir()), "the same folder");
    assert!(folder.follow(&native(r"E:\models")));
    assert_eq!(folder.of(small), None);
    assert!(folder.reading(), "the new folder is not read yet");
    let read = folder.scanned(Ok(scan(&[(&small.file, small.bytes)])));
    let file = native_path(r"E:\models").join(&small.file);
    assert_eq!(read, [(file, small.bytes)], "its fingerprint is read again");
}

#[test]
fn a_drive_holds_a_folder_in_any_case_on_windows_alone() {
    let at = |root: &str| Drive {
        root: Redacted::new(PathBuf::from(root)),
        name: Redacted::new(root.to_owned()),
        free: MB,
    };
    let windows = [at(r"D:\Data")];
    let held = drive_of(PathRoot::Drive, &windows, Path::new(r"d:\data\models"));
    assert!(held.is_some(), "Windows names ignore case");
    let unix = [at("/media/Data")];
    let other = drive_of(PathRoot::Slash, &unix, Path::new("/media/data/models"));
    assert!(other.is_none(), "Unix names keep their case");
    let same = drive_of(PathRoot::Slash, &unix, Path::new("/media/Data/models"));
    assert!(same.is_some());
}

#[test]
fn a_missing_drive_or_too_little_room_refuses_a_download() {
    let folder = Folder::new(&dir());
    let other = [drive(r"E:\", 100 * MB)];
    assert_eq!(
        folder.room(&other, MB, 1),
        Err(ModelError::Missing),
        "no drive holds it"
    );
    let drives = [drive(r"d:\", 100 * MB)];
    assert_eq!(folder.room(&drives, 60 * MB, 40), Ok(()));
    assert_eq!(
        folder.room(&drives, 60 * MB, 41),
        Err(ModelError::Full),
        "min_free_mb not met"
    );
    assert_eq!(folder.line(&drives).map(|l| l.free), Some(100 * MB));
    assert_eq!(
        Folder::new("").room(&drives, MB, 1),
        Err(ModelError::Missing),
        "no folder"
    );
}
