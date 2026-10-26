//! Which languages ship: the index in each folder, read once at the first ask, and nothing from one that does not read.

use std::collections::HashMap;
use std::sync::Mutex;

use super::super::*;
use super::folder;

/// A fake disk of text files: each file's bytes, or its error, and every path it opened.
#[derive(Default)]
struct Texts {
    files: HashMap<PathBuf, Result<Vec<u8>, PackError>>,
    opened: Mutex<Vec<PathBuf>>,
}

/// One file's bytes as its view.
struct Bytes(Vec<u8>);

impl PackView for Bytes {
    fn bytes(&self) -> &[u8] {
        &self.0
    }
}

impl PackDisk for Texts {
    type File = Vec<u8>;
    type View = Bytes;

    fn open(&self, path: &Path) -> Result<Option<(Vec<u8>, u64)>, PackError> {
        self.opened.lock().unwrap().push(path.to_path_buf());
        let file = self.files.get(path).cloned().transpose()?;
        Ok(file.map(|bytes| (bytes.clone(), u64::try_from(bytes.len()).unwrap())))
    }

    fn map(&self, file: Vec<u8>, len: usize) -> Result<Bytes, PackError> {
        Ok(Bytes(file.get(..len).unwrap().to_vec()))
    }
}

/// Packs in `dirs` whose index files hold each of `indexes`, by folder.
fn packs(dirs: &[PathBuf], indexes: &[(&PathBuf, Result<&[u8], PackError>)]) -> DiskPacks<Texts> {
    let files = indexes
        .iter()
        .map(|(dir, text)| (dir.join(PACK_INDEX), text.map(<[u8]>::to_vec)))
        .collect();
    let disk = Texts {
        files,
        ..Texts::default()
    };
    DiskPacks::new(disk, dirs.to_vec())
}

/// True when `packs` ships a pack serving layout code `iso`.
fn ships(packs: &DiskPacks<Texts>, iso: &str) -> bool {
    packs.pack_of(iso).is_some()
}

#[test]
fn a_language_ships_when_the_index_lists_it() {
    let dir = folder("a");
    let packs = packs(std::slice::from_ref(&dir), &[(&dir, Ok(b"ar\nen\n"))]);
    assert!(ships(&packs, "en") && ships(&packs, "ar"));
    assert!(!ships(&packs, "fr"), "not listed");
    assert!(
        !ships(&packs, "EN") && !ships(&packs, "../en"),
        "no such code"
    );
}

#[test]
fn the_index_is_read_once_at_the_first_ask() {
    let dir = folder("a");
    let packs = packs(std::slice::from_ref(&dir), &[(&dir, Ok(b"en\n"))]);
    assert!(
        packs.disk.opened.lock().unwrap().is_empty(),
        "nothing read at start"
    );
    for _ in 0..3 {
        assert!(ships(&packs, "en"));
    }
    assert_eq!(*packs.disk.opened.lock().unwrap(), [dir.join(PACK_INDEX)]);
}

#[test]
fn every_absolute_folders_index_counts_and_a_relative_one_is_never_read() {
    let (a, b, rel) = (folder("a"), folder("b"), PathBuf::from("packs"));
    let dirs = [rel.clone(), a.clone(), b.clone()];
    let packs = packs(
        &dirs,
        &[(&rel, Ok(b"de\n")), (&a, Ok(b"en\n")), (&b, Ok(b"fr\n"))],
    );
    assert!(ships(&packs, "en") && ships(&packs, "fr"));
    assert!(
        !ships(&packs, "de"),
        "the working folder never lists a pack"
    );
}

#[test]
fn a_missing_damaged_or_unreadable_index_ships_nothing_from_its_folder() {
    let (a, b) = (folder("a"), folder("b"));
    let dirs = [a.clone(), b.clone()];
    assert!(!ships(&packs(&dirs, &[]), "en"), "no index anywhere");
    let damaged = packs(&dirs, &[(&a, Ok(b"en\nEN\n")), (&b, Ok(b"fr\n"))]);
    assert!(!ships(&damaged, "en") && ships(&damaged, "fr"));
    let unreadable = packs(
        &dirs,
        &[(&a, Err(PackError::Unreadable)), (&b, Ok(b"fr\n"))],
    );
    assert!(!ships(&unreadable, "en") && ships(&unreadable, "fr"));
}

#[test]
fn an_empty_index_file_is_refused_by_the_index_rule_and_ships_nothing() {
    let dir = folder("a");
    let packs = packs(std::slice::from_ref(&dir), &[(&dir, Ok(b""))]);
    assert_eq!(
        index(&packs.disk, &dir.join(PACK_INDEX)),
        Err(PackIndex::read(b"").unwrap_err())
    );
    assert!(!ships(&packs, "en"));
}

#[test]
fn an_index_file_past_the_index_cap_is_unreadable() {
    let dir = folder("a");
    let cap = usize::try_from(MAX_INDEX_BYTES).unwrap();
    let over = "en\n".repeat(cap).into_bytes();
    let packs = packs(std::slice::from_ref(&dir), &[(&dir, Ok(&over))]);
    assert_eq!(
        index(&packs.disk, &dir.join(PACK_INDEX)),
        Err(IndexError::Unreadable)
    );
    assert!(!ships(&packs, "en"));
}

#[test]
fn a_later_folder_whose_index_clashes_with_an_earlier_one_adds_nothing() {
    let (a, b) = (folder("a"), folder("b"));
    let dirs = [a.clone(), b.clone()];
    let packs = packs(&dirs, &[(&a, Ok(b"sh hr\n")), (&b, Ok(b"fr\nsl hr\n"))]);
    assert_eq!(packs.pack_of("hr").as_deref(), Some("sh"));
    assert!(!ships(&packs, "fr") && !ships(&packs, "sl"));
}

#[test]
fn a_layout_code_the_index_lists_after_a_pack_is_served_by_that_pack() {
    let dir = folder("a");
    let packs = packs(std::slice::from_ref(&dir), &[(&dir, Ok(b"en\nsh bs hr\n"))]);
    assert_eq!(packs.pack_of("hr").as_deref(), Some("sh"));
    assert_eq!(packs.pack_of("sh").as_deref(), Some("sh"));
    assert_eq!(packs.pack_of("en").as_deref(), Some("en"));
    assert_eq!(packs.pack_of("sr"), None);
}
