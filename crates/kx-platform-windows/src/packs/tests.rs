//! The Win32 pack disk on real files: whole bytes through a read-only file map, a missing file, unreadable files,
//! writers and deleters kept out while mapped, and a view that moves between threads.

use std::fs::{self, OpenOptions};
use std::path::PathBuf;

use kx_module_api::ports::{MAX_PACK_BYTES, WordPacks, pack_file};
use kx_test_support::tempdir::TempDir;
use windows::Win32::System::Memory::{MEM_MAPPED, MEMORY_BASIC_INFORMATION, VirtualQuery};

use super::*;

/// A pack's length in the tests: more than one page, and not a whole number of them.
const LEN: usize = 10_000;

/// A temp folder for packs, removed at the end.
fn packs_dir() -> (TempDir, PathBuf) {
    let dir = TempDir::new("packs").unwrap();
    let path = dir.path().to_path_buf();
    (dir, path)
}

/// The packs in `dir` alone.
fn packs(dir: &Path) -> WinWordPacks {
    WinWordPacks::new(WinPackDisk, vec![dir.to_path_buf()])
}

/// Writes `bytes` as `iso`'s pack in `dir`, and gives its path.
fn write(dir: &Path, iso: &str, bytes: &[u8]) -> PathBuf {
    let path = dir.join(pack_file(iso).unwrap());
    fs::write(&path, bytes).unwrap();
    path
}

/// Bytes that differ from one place to the next.
fn sample() -> Vec<u8> {
    (0..LEN).map(|i| u8::try_from(i % 251).unwrap()).collect()
}

#[test]
fn a_packs_bytes_come_back_whole_through_the_view() {
    let (_dir, path) = packs_dir();
    write(&path, "en", &sample());
    let view = packs(&path).open("en").unwrap().unwrap();
    assert_eq!(view.bytes(), sample().as_slice());
}

#[test]
fn a_missing_file_or_folder_is_none() {
    let (_dir, path) = packs_dir();
    write(&path, "en", &sample());
    let packs = WinWordPacks::new(WinPackDisk, vec![path.join("missing"), path]);
    assert!(packs.open("fr").unwrap().is_none());
}

/// What Windows says of the memory region that holds `at`.
#[allow(unsafe_code, reason = "Asks Windows about our own memory.")]
fn region_of(at: *const u8) -> MEMORY_BASIC_INFORMATION {
    let mut info = MEMORY_BASIC_INFORMATION::default();
    let size = size_of_val(&info);
    // SAFETY: `info` is ours and its exact size goes with it; the query only reads the page tables.
    let wrote = unsafe { VirtualQuery(Some(at.cast()), std::ptr::from_mut(&mut info), size) };
    assert_eq!(wrote, size, "Windows describes the region");
    info
}

#[test]
fn the_view_is_a_read_only_file_map_never_copy_on_write() {
    let (_dir, path) = packs_dir();
    write(&path, "en", &sample());
    let view = packs(&path).open("en").unwrap().unwrap();
    let region = region_of(view.bytes().as_ptr());
    assert_eq!(region.Protect, PAGE_READONLY, "not PAGE_WRITECOPY");
    assert_eq!(region.Type, MEM_MAPPED);
    assert!(region.RegionSize >= LEN, "the whole pack is one region");
}

#[test]
fn an_empty_pack_a_folder_and_an_oversized_pack_are_unreadable() {
    let (_dir, path) = packs_dir();
    write(&path, "en", &[]);
    fs::create_dir(path.join(pack_file("fr").unwrap())).unwrap();
    let big = write(&path, "ar", &[]);
    let file = OpenOptions::new().write(true).open(big).unwrap();
    file.set_len(MAX_PACK_BYTES + 1).unwrap();
    drop(file);
    let packs = packs(&path);
    for iso in ["en", "fr", "ar"] {
        assert_eq!(packs.open(iso).err(), Some(PackError::Unreadable), "{iso}");
    }
}

#[test]
fn a_mapped_pack_cannot_be_written_or_deleted_until_its_view_drops() {
    let (_dir, path) = packs_dir();
    let file = write(&path, "en", &sample());
    let view = packs(&path).open("en").unwrap().unwrap();
    assert!(
        OpenOptions::new().write(true).open(&file).is_err(),
        "no writer while mapped"
    );
    assert!(fs::remove_file(&file).is_err(), "no delete while mapped");
    drop(view);
    fs::remove_file(&file).unwrap();
}

#[test]
fn the_view_moves_to_another_thread_and_reads_there() {
    let (_dir, path) = packs_dir();
    write(&path, "en", &sample());
    let view = packs(&path).open("en").unwrap().unwrap();
    let len = std::thread::spawn(move || view.bytes().len()).join();
    assert_eq!(len.ok(), Some(LEN));
}
