use super::*;
use crate::SealedFiles;
use crate::consts::{DAMAGED_EXT, MAX_BLOB_BYTES, SEALED_EXT, TEMP_EXT};
use crate::testing::{Mirror, listing};
use kx_module_api::ports::SealedStore;
use kx_test_support::tempdir::TempDir;
use std::fs;
use std::io;
use std::path::Path;
use std::sync::{Arc, Mutex};

mod agree;
mod damage;
mod keys;
mod lists;
mod props;
mod room;

/// A day folder, as the history names one.
const DAY: &str = "2026-10-31";
/// The pinned folder.
const PINNED: &str = "pinned";
/// The free bytes `Scripted` answers.
const FREE: u64 = 7_000_000;
/// The drive name `Scripted` answers.
const DRIVE: &str = "D:";

/// A test drive that answers `FREE` and `DRIVE`, or fails on cue, and records the folders it is asked about.
#[derive(Clone, Default)]
struct Scripted {
    fail: bool,
    asked: Arc<Mutex<Vec<PathBuf>>>,
}

impl Scripted {
    fn answer<T>(&self, path: &Path, value: T) -> io::Result<T> {
        self.asked.lock().unwrap().push(path.to_owned());
        if self.fail {
            return Err(io::ErrorKind::PermissionDenied.into());
        }
        Ok(value)
    }

    fn asked(&self) -> Vec<PathBuf> {
        self.asked.lock().unwrap().clone()
    }
}

impl DiskRoom for Scripted {
    fn free(&self, path: &Path) -> io::Result<u64> {
        self.answer(path, FREE)
    }

    fn drive(&self, path: &Path) -> io::Result<String> {
        self.answer(path, DRIVE.to_owned())
    }
}

/// A shelf over a root that does not exist yet, inside a temp folder removed at the end.
struct Case {
    temp: TempDir,
    root: PathBuf,
    guard: Mirror,
    disk: Scripted,
    shelf: SealedShelfFiles,
}

fn case() -> Case {
    let temp = TempDir::new("crypto-shelf").unwrap();
    let root = temp.path().join("clipboard");
    let (guard, disk) = (Mirror::default(), Scripted::default());
    let shelf = SealedShelfFiles::new(
        root.clone(),
        Box::new(guard.clone()),
        Box::new(disk.clone()),
    );
    Case {
        temp,
        root,
        guard,
        disk,
        shelf,
    }
}

impl Case {
    /// A shelf for a later run on the same root, with `guard`.
    fn reopen(&self, guard: &Mirror) -> SealedShelfFiles {
        let disk = Box::new(self.disk.clone());
        SealedShelfFiles::new(self.root.clone(), Box::new(guard.clone()), disk)
    }

    fn file(&self, folder: &str, name: &str, ext: &str) -> PathBuf {
        self.root.join(folder).join(format!("{name}.{ext}"))
    }

    fn key_file(&self) -> PathBuf {
        self.root.join(KEY_FILE)
    }

    /// The file names in `folder`, or in the root for `""`, sorted.
    fn listing(&self, folder: &str) -> Vec<String> {
        listing(&self.root.join(folder))
    }
}

fn name(text: &str) -> BlobName {
    BlobName::new(text).unwrap()
}

/// `texts`, sorted, as `listing` gives them.
fn sorted(texts: &[&str]) -> Vec<String> {
    let mut texts: Vec<_> = texts.iter().map(|t| (*t).to_owned()).collect();
    texts.sort();
    texts
}

#[test]
fn save_then_load_round_trips_per_folder() {
    let c = case();
    c.shelf
        .save(&name(DAY), &name("e1"), b"copied today")
        .unwrap();
    c.shelf.save(&name(PINNED), &name("e1"), b"kept").unwrap();
    let day = c.shelf.load(&name(DAY), &name("e1")).unwrap();
    assert_eq!(day.as_deref(), Some(&b"copied today"[..]));
    let pinned = c.shelf.load(&name(PINNED), &name("e1")).unwrap();
    assert_eq!(pinned.as_deref(), Some(&b"kept"[..]));
    let file = fs::read(c.file(DAY, "e1", SEALED_EXT)).unwrap();
    assert!(!file.windows(6).any(|w| w == b"copied"));
}

#[test]
fn the_root_holds_one_key_and_a_folder_of_blobs_each() {
    let c = case();
    for (folder, blob) in [(DAY, "e1"), (DAY, "e2"), (PINNED, "e1")] {
        c.shelf.save(&name(folder), &name(blob), b"x").unwrap();
    }
    assert_eq!(c.listing(""), sorted(&[DAY, PINNED, KEY_FILE]));
    assert_eq!(c.listing(DAY), ["e1.sealed", "e2.sealed"]);
}

#[test]
fn the_aad_names_the_folder_then_the_blob() {
    assert_eq!(aad(&name(DAY), &name("e1")), b"2026-10-31/e1");
}

#[test]
fn a_missing_blob_or_folder_loads_as_none_without_the_guard() {
    let c = case();
    assert_eq!(c.shelf.load(&name(DAY), &name("e1")), Ok(None));
    assert_eq!(c.guard.calls(), 0);
    assert!(!c.root.exists());
}

#[test]
fn a_save_replaces_the_whole_blob() {
    let c = case();
    c.shelf.save(&name(DAY), &name("e1"), b"first").unwrap();
    c.shelf.save(&name(DAY), &name("e1"), b"second").unwrap();
    let got = c.shelf.load(&name(DAY), &name("e1")).unwrap();
    assert_eq!(got.as_deref(), Some(&b"second"[..]));
    assert_eq!(c.listing(DAY), ["e1.sealed"]);
}

#[test]
fn a_blob_moved_to_another_folder_will_not_open() {
    let c = case();
    c.shelf.save(&name(DAY), &name("e1"), b"words").unwrap();
    fs::create_dir_all(c.root.join(PINNED)).unwrap();
    let from = c.file(DAY, "e1", SEALED_EXT);
    fs::copy(&from, c.file(PINNED, "e1", SEALED_EXT)).unwrap();
    assert_eq!(
        c.shelf.load(&name(PINNED), &name("e1")),
        Err(SealedError::Damaged)
    );
    assert!(c.shelf.load(&name(DAY), &name("e1")).unwrap().is_some());
}

#[test]
fn a_renamed_blob_will_not_open() {
    let c = case();
    c.shelf.save(&name(DAY), &name("e1"), b"words").unwrap();
    let from = c.file(DAY, "e1", SEALED_EXT);
    fs::rename(&from, c.file(DAY, "e2", SEALED_EXT)).unwrap();
    assert_eq!(
        c.shelf.load(&name(DAY), &name("e2")),
        Err(SealedError::Damaged)
    );
}

#[test]
fn blobs_never_cross_between_the_store_and_the_shelf() {
    let c = case();
    let store = SealedFiles::new(c.root.clone(), Box::new(c.guard.clone()));
    store.save(&name("e1"), b"store").unwrap();
    c.shelf.save(&name(DAY), &name("e2"), b"shelf").unwrap();
    let store_file = c.root.join(format!("e1.{SEALED_EXT}"));
    fs::copy(&store_file, c.file(DAY, "e1", SEALED_EXT)).unwrap();
    fs::copy(
        c.file(DAY, "e2", SEALED_EXT),
        c.root.join(format!("e2.{SEALED_EXT}")),
    )
    .unwrap();
    assert_eq!(
        c.shelf.load(&name(DAY), &name("e1")),
        Err(SealedError::Damaged)
    );
    assert_eq!(store.load(&name("e2")), Err(SealedError::Damaged));
}

#[test]
fn remove_deletes_the_blob_with_its_leftovers_for_good() {
    let c = case();
    c.shelf.save(&name(DAY), &name("e1"), b"one").unwrap();
    c.shelf.save(&name(DAY), &name("e2"), b"two").unwrap();
    fs::write(c.file(DAY, "e1", DAMAGED_EXT), b"older").unwrap();
    fs::write(
        c.file(DAY, "e1", &format!("{SEALED_EXT}.{TEMP_EXT}")),
        b"half",
    )
    .unwrap();
    c.shelf.remove(&name(DAY), &name("e1")).unwrap();
    assert_eq!(c.listing(DAY), ["e2.sealed"]);
    assert_eq!(c.shelf.remove(&name(DAY), &name("e1")), Ok(()));
    assert_eq!(c.shelf.remove(&name("2026-01-01"), &name("e1")), Ok(()));
}

#[test]
fn drop_folder_deletes_it_and_everything_in_it() {
    let c = case();
    c.shelf.save(&name(DAY), &name("e1"), b"one").unwrap();
    c.shelf.save(&name(PINNED), &name("e1"), b"kept").unwrap();
    fs::write(c.root.join(DAY).join("notes.txt"), b"stray").unwrap();
    fs::create_dir_all(c.root.join(DAY).join("inner")).unwrap();
    fs::write(c.root.join(DAY).join("inner").join("x"), b"deep").unwrap();
    c.shelf.drop_folder(&name(DAY)).unwrap();
    assert_eq!(c.listing(""), sorted(&[PINNED, KEY_FILE]));
    assert_eq!(c.shelf.drop_folder(&name(DAY)), Ok(()));
    assert!(c.shelf.load(&name(PINNED), &name("e1")).unwrap().is_some());
}

#[test]
fn an_oversized_blob_is_io_and_writes_nothing() {
    let c = case();
    let big = vec![0; MAX_BLOB_BYTES + 1];
    assert_eq!(
        c.shelf.save(&name(DAY), &name("e1"), &big),
        Err(SealedError::Io)
    );
    assert!(!c.root.exists());
}

#[test]
fn a_refused_wrap_makes_no_folder_and_no_key() {
    let c = case();
    let shelf = c.reopen(&Mirror::refusing());
    assert_eq!(
        shelf.save(&name(DAY), &name("e1"), b"x"),
        Err(SealedError::Locked)
    );
    assert!(!c.root.join(DAY).exists());
    assert!(!c.key_file().exists());
}

#[test]
fn the_debug_text_shows_no_folder() {
    let c = case();
    assert!(!format!("{:?}", c.shelf).contains("clipboard"));
}
