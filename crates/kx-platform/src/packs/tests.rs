//! The pack rules over a fake disk: the code check, the size limit and the folder order.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use super::*;

/// The language every folder test looks up.
const ISO: &str = "en";

/// A fake pack view: the tag of the file it maps, as its one byte.
struct Tagged(u8);

impl PackView for Tagged {
    fn bytes(&self) -> &[u8] {
        std::slice::from_ref(&self.0)
    }
}

/// A fake disk: each pack file's tag and length, or its error, and every path and length it was asked for.
#[derive(Default)]
struct Disk {
    files: HashMap<PathBuf, Result<(u8, u64), PackError>>,
    asked: Mutex<Vec<PathBuf>>,
    mapped: Mutex<Vec<usize>>,
}

impl Disk {
    /// A disk holding `ISO`'s pack of one byte in each folder of `with`, each tagged with its place in that list.
    fn with(with: &[&PathBuf]) -> Self {
        let files = with
            .iter()
            .zip(0..)
            .map(|(dir, n)| (pack(dir), Ok((n, 1))))
            .collect();
        Self {
            files,
            ..Self::default()
        }
    }

    /// The paths asked so far.
    fn asked(&self) -> Vec<PathBuf> {
        self.asked.lock().unwrap().clone()
    }
}

impl PackDisk for Disk {
    type File = u8;
    type View = Tagged;

    fn open(&self, path: &Path) -> Result<Option<(u8, u64)>, PackError> {
        self.asked.lock().unwrap().push(path.to_path_buf());
        self.files.get(path).copied().transpose()
    }

    fn map(&self, file: u8, len: usize) -> Result<Tagged, PackError> {
        self.mapped.lock().unwrap().push(len);
        Ok(Tagged(file))
    }
}

/// An absolute folder on every system.
fn folder(name: &str) -> PathBuf {
    std::env::temp_dir().join(name)
}

/// `ISO`'s pack file in `dir`.
fn pack(dir: &Path) -> PathBuf {
    dir.join(pack_file(ISO).unwrap())
}

/// The tag of the pack `find` gives, or its error.
fn tag(disk: &Disk, dirs: &[PathBuf], iso: &str) -> Result<Option<u8>, PackError> {
    find(disk, dirs, iso).map(|view| view.map(|v| v.0))
}

#[test]
fn a_bad_code_never_reaches_the_disk() {
    let disk = Disk::default();
    for iso in ["../x", "EN", "e", "engl", "con", "nul"] {
        assert_eq!(tag(&disk, &[folder("a")], iso), Ok(None));
    }
    assert!(disk.asked().is_empty());
}

#[test]
fn the_first_folder_with_the_pack_wins() {
    let (a, b, c) = (folder("a"), folder("b"), folder("c"));
    let disk = Disk::with(&[&b, &c]);
    let dirs = [a.clone(), b.clone(), c];
    assert_eq!(tag(&disk, &dirs, ISO), Ok(Some(0)));
    assert_eq!(disk.asked(), [pack(&a), pack(&b)]);
}

#[test]
fn a_missing_language_is_none_after_every_folder() {
    let disk = Disk::default();
    let dirs = [folder("a"), folder("b")];
    assert_eq!(tag(&disk, &dirs, ISO), Ok(None));
    assert_eq!(disk.asked(), [pack(&dirs[0]), pack(&dirs[1])]);
}

#[test]
fn an_unreadable_pack_stands_over_a_later_folder() {
    let (a, b) = (folder("a"), folder("b"));
    let mut disk = Disk::with(&[&b]);
    disk.files.insert(pack(&a), Err(PackError::Unreadable));
    assert_eq!(tag(&disk, &[a, b], ISO), Err(PackError::Unreadable));
}

#[test]
fn a_relative_folder_is_never_tried() {
    let abs = folder("a");
    let disk = Disk::with(&[&PathBuf::from("packs"), &abs]);
    let dirs = [PathBuf::from("packs"), abs.clone()];
    assert_eq!(tag(&disk, &dirs, ISO), Ok(Some(1)));
    assert_eq!(disk.asked(), [pack(&abs)]);
}

#[test]
fn a_pack_is_one_byte_to_max_pack_bytes() {
    let max = usize::try_from(MAX_PACK_BYTES).unwrap();
    assert_eq!(view_len(0), Err(PackError::Unreadable));
    assert_eq!(view_len(1), Ok(1));
    assert_eq!(view_len(MAX_PACK_BYTES), Ok(max));
    assert_eq!(view_len(MAX_PACK_BYTES + 1), Err(PackError::Unreadable));
    assert_eq!(view_len(u64::MAX), Err(PackError::Unreadable));
}

#[test]
fn an_empty_or_oversized_file_is_unreadable_and_never_mapped() {
    let dir = folder("a");
    for len in [0, MAX_PACK_BYTES + 1] {
        let mut disk = Disk::default();
        disk.files.insert(pack(&dir), Ok((0, len)));
        let dirs = std::slice::from_ref(&dir);
        assert_eq!(tag(&disk, dirs, ISO), Err(PackError::Unreadable));
        assert!(disk.mapped.lock().unwrap().is_empty(), "{len}");
    }
}

#[test]
fn the_largest_pack_is_mapped_whole() {
    let dir = folder("a");
    let mut disk = Disk::default();
    disk.files.insert(pack(&dir), Ok((0, MAX_PACK_BYTES)));
    assert_eq!(tag(&disk, &[dir], ISO), Ok(Some(0)));
    let max = usize::try_from(MAX_PACK_BYTES).unwrap();
    assert_eq!(*disk.mapped.lock().unwrap(), [max]);
}

#[test]
fn the_packs_serve_as_the_port_from_another_thread() {
    let dir = folder("a");
    let packs: Arc<dyn WordPacks> = Arc::new(DiskPacks::new(Disk::with(&[&dir]), vec![dir]));
    let read = std::thread::spawn(move || packs.open(ISO).map(|v| v.map(|v| v.bytes().to_vec())));
    assert_eq!(read.join().unwrap(), Ok(Some(vec![0])));
}

mod index;
