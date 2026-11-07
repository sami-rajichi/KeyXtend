//! The memory stores keep the port contract the file stores keep, write no file, let go of what they replace or
//! delete, and show nothing they hold.

use super::*;
use crate::testing::{DAY, PINNED, Scripted, contract, listing, name};
use kx_module_api::ports::SealedShelf;
use kx_test_support::tempdir::TempDir;

/// The bytes a first save keeps.
const OLD: &[u8] = b"old words";
/// Shorter bytes that replace them.
const NEW: &[u8] = b"new";

/// A shelf told about an empty temp folder, on a default `Scripted` drive.
struct Case {
    temp: TempDir,
    disk: Scripted,
    shelf: MemoryShelf,
}

fn case() -> Case {
    let temp = TempDir::new("crypto-memory").unwrap();
    let disk = Scripted::default();
    let shelf = MemoryShelf::new(temp.path().to_owned(), Box::new(disk.clone()));
    Case { temp, disk, shelf }
}

/// The bytes the shelf's room says it holds.
fn used(shelf: &MemoryShelf) -> u64 {
    shelf.room().unwrap().used
}

#[test]
fn the_store_keeps_the_port_contract() {
    contract::store(&MemoryStore::new());
}

#[test]
fn the_shelf_keeps_the_port_contract_and_writes_no_file() {
    let c = case();
    contract::shelf(&c.shelf, 0);
    assert_eq!(listing(c.temp.path()), Vec::<String>::new());
}

#[test]
fn the_room_and_the_free_bytes_ask_the_drive_about_the_place() {
    let c = case();
    c.shelf.room().unwrap();
    c.shelf.free().unwrap();
    let at = c.temp.path().to_owned();
    let asked = [at.clone(), at.clone(), at];
    assert_eq!(c.disk.asked(), asked, "free asks for the free bytes alone");
}

#[test]
fn a_missing_place_asks_about_its_nearest_folder_and_stays_missing() {
    let c = case();
    let at = c.temp.path().join("clipboard");
    let shelf = MemoryShelf::new(at.clone(), Box::new(c.disk.clone()));
    shelf.save(&name(DAY), &name("e1"), OLD).unwrap();
    shelf.room().unwrap();
    let parent = c.temp.path().to_owned();
    assert_eq!(c.disk.asked(), [parent.clone(), parent]);
    assert!(!at.exists());
}

#[test]
fn a_drive_that_cannot_be_read_is_io() {
    let c = case();
    let disk = Box::new(Scripted::failing());
    contract::blind(&MemoryShelf::new(c.temp.path().to_owned(), disk));
}

#[test]
fn a_replaced_removed_or_dropped_blob_is_held_no_more() {
    let c = case();
    let (day, pinned, e1) = (name(DAY), name(PINNED), name("e1"));
    c.shelf.save(&day, &e1, OLD).unwrap();
    c.shelf.save(&pinned, &e1, OLD).unwrap();
    c.shelf.save(&day, &e1, NEW).unwrap();
    let sizes = [OLD.len(), NEW.len()].map(|n| u64::try_from(n).unwrap());
    assert_eq!(used(&c.shelf), sizes[0] + sizes[1]);
    c.shelf.remove(&day, &e1).unwrap();
    assert_eq!(used(&c.shelf), sizes[0]);
    c.shelf.drop_folder(&pinned).unwrap();
    assert_eq!(used(&c.shelf), 0);
}

/// Each blob is held as `Zeroizing`, so it is wiped when replaced, removed or dropped; a plain `Vec` would not build.
#[test]
fn only_the_new_blob_is_held_as_zeroizing() {
    let store = MemoryStore::new();
    store.save(&name("emails"), OLD).unwrap();
    store.save(&name("emails"), NEW).unwrap();
    let blobs: &Blobs = &lock(&store.blobs);
    let held: Vec<&Zeroizing<Vec<u8>>> = blobs.values().collect();
    assert_eq!(held, [&Zeroizing::new(NEW.to_vec())]);
}

#[test]
fn the_debug_text_shows_nothing_held_and_no_place() {
    let c = case();
    c.shelf.save(&name(DAY), &name("e1"), OLD).unwrap();
    let store = MemoryStore::new();
    store.save(&name("emails"), OLD).unwrap();
    assert_eq!(format!("{store:?}"), "MemoryStore { .. }");
    assert_eq!(format!("{:?}", c.shelf), "MemoryShelf { .. }");
}
