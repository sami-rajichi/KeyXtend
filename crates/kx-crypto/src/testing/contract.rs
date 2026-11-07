//! The port contract the file stores and the memory stores both keep, so they cannot drift. Each check takes an empty
//! store, and a shelf on a default `Scripted` drive.

use super::{DAY, DRIVE, FREE, PINNED, name};
use kx_module_api::ports::{BlobName, MAX_BLOB_BYTES, Room, SealedError, SealedShelf, SealedStore};

/// The bytes a first save keeps.
const OLD: &[u8] = b"old words";
/// The bytes that replace them, in Arabic so no store may change the text.
const NEW: &[u8] = "كلمات جديدة".as_bytes();
/// Device names' lookalikes, which every store keeps.
const LOOKALIKES: [&str; 2] = ["con-1", "lpt10"];
/// The day before `DAY`, which sorts first.
const EARLIER: &str = "2026-10-30";
/// A day no check ever saves to.
const NEVER: &str = "2026-01-01";

/// A folder's blobs, each with its bytes, as a shelf lists and loads them.
pub(crate) type Seen = Vec<(BlobName, Vec<u8>)>;

/// Exactly the cap, which every store keeps.
fn at_cap() -> Vec<u8> {
    vec![0; MAX_BLOB_BYTES]
}

/// One byte over the cap.
fn oversized() -> Vec<u8> {
    vec![0; MAX_BLOB_BYTES + 1]
}

/// Checks an empty `store` against the `SealedStore` port: loads, replacing saves, the size cap and every name.
pub(crate) fn store(store: &dyn SealedStore) {
    let load = |name: &BlobName| store.load(name).unwrap();
    let (en, fr, emails) = (name("predict-en"), name("predict-fr"), name("emails"));
    assert_eq!(load(&en), None, "nothing saved yet");
    store.save(&en, OLD).unwrap();
    store.save(&fr, NEW).unwrap();
    assert_eq!(load(&en).as_deref(), Some(OLD));
    assert_eq!(load(&fr).as_deref(), Some(NEW), "kept apart");
    store.save(&en, NEW).unwrap();
    assert_eq!(load(&en).as_deref(), Some(NEW), "a save replaces");
    assert_eq!(store.save(&en, &oversized()), Err(SealedError::Io));
    assert_eq!(load(&en).as_deref(), Some(NEW), "the old blob stays");
    assert_eq!(store.save(&emails, &oversized()), Err(SealedError::Io));
    assert_eq!(load(&emails), None, "a refused save keeps nothing");
    assert_eq!(store.save(&emails, &at_cap()), Ok(()), "at the cap");
    assert_eq!(load(&emails).map(|b| b.len()), Some(MAX_BLOB_BYTES));
    store.save(&emails, b"").unwrap();
    assert_eq!(load(&emails), Some(Vec::new()), "empty, not missing");
    for text in LOOKALIKES {
        store.save(&name(text), text.as_bytes()).unwrap();
        assert_eq!(load(&name(text)).as_deref(), Some(text.as_bytes()));
    }
}

/// Checks an empty `shelf` against the `SealedShelf` port; each blob takes its length and `overhead` of the room.
pub(crate) fn shelf(shelf: &dyn SealedShelf, overhead: usize) {
    assert_eq!(shelf.folders(), Ok(vec![]));
    assert_eq!(shelf.names(&name(DAY)), Ok(vec![]));
    assert_eq!(shelf.load(&name(DAY), &name("e1")), Ok(None));
    room(shelf, overhead);
    saves(shelf);
    lists(shelf);
    room(shelf, overhead);
    deletes(shelf);
    room(shelf, overhead);
}

/// Checks a `shelf` whose drive cannot be read: its room and its free bytes are `Io`.
pub(crate) fn blind(shelf: &dyn SealedShelf) {
    assert_eq!(shelf.room(), Err(SealedError::Io));
    assert_eq!(shelf.free(), Err(SealedError::Io));
}

/// Every folder of `shelf` with its blobs, as it lists and loads them.
pub(crate) fn seen(shelf: &dyn SealedShelf) -> Vec<(BlobName, Seen)> {
    let blobs = |folder: &BlobName| -> Seen {
        let names = shelf.names(folder).unwrap().into_iter();
        names
            .map(|n| (n.clone(), shelf.load(folder, &n).unwrap().unwrap()))
            .collect()
    };
    let folders = shelf.folders().unwrap().into_iter();
    folders.map(|f| (f.clone(), blobs(&f))).collect()
}

/// The room counts each listed blob's length and `overhead`; the free bytes and the drive's name come from the drive.
fn room(shelf: &dyn SealedShelf, overhead: usize) {
    let blobs = seen(shelf).into_iter().flat_map(|(_, blobs)| blobs);
    let used: usize = blobs.map(|(_, bytes)| bytes.len() + overhead).sum();
    let want = Room {
        used: u64::try_from(used).unwrap(),
        free: FREE,
        drive: DRIVE.to_owned(),
    };
    assert_eq!(shelf.room(), Ok(want));
    assert_eq!(shelf.free(), Ok(FREE));
}

/// Saves load back, kept apart by folder; a save replaces, and an oversized one is `Io` and changes nothing.
fn saves(shelf: &dyn SealedShelf) {
    let load = |folder: &BlobName, name: &BlobName| shelf.load(folder, name).unwrap();
    let (day, pinned, e1) = (name(DAY), name(PINNED), name("e1"));
    shelf.save(&day, &e1, OLD).unwrap();
    shelf.save(&pinned, &e1, NEW).unwrap();
    assert_eq!(load(&day, &e1).as_deref(), Some(OLD));
    assert_eq!(load(&pinned, &e1).as_deref(), Some(NEW), "kept apart");
    assert_eq!(shelf.save(&day, &e1, &at_cap()), Ok(()), "at the cap");
    assert_eq!(load(&day, &e1).map(|b| b.len()), Some(MAX_BLOB_BYTES));
    shelf.save(&day, &e1, NEW).unwrap();
    assert_eq!(load(&day, &e1).as_deref(), Some(NEW), "a save replaces");
    assert_eq!(shelf.save(&day, &e1, &oversized()), Err(SealedError::Io));
    assert_eq!(load(&day, &e1).as_deref(), Some(NEW), "the old blob stays");
    let refused = shelf.save(&name(EARLIER), &e1, &oversized());
    assert_eq!(refused, Err(SealedError::Io));
    let folders = shelf.folders().unwrap();
    assert_eq!(folders, [day, pinned], "and makes no folder");
}

/// Folders and names list sorted, whatever order they were saved in, device names' lookalikes too.
fn lists(shelf: &dyn SealedShelf) {
    let day = name(DAY);
    shelf.save(&day, &name("d1"), OLD).unwrap();
    shelf.save(&name(EARLIER), &name("e1"), OLD).unwrap();
    for text in LOOKALIKES {
        let both = name(text);
        shelf.save(&both, &both, text.as_bytes()).unwrap();
    }
    let folders = [EARLIER, DAY, LOOKALIKES[0], LOOKALIKES[1], PINNED].map(name);
    assert_eq!(shelf.folders(), Ok(folders.to_vec()));
    assert_eq!(shelf.names(&day), Ok(vec![name("d1"), name("e1")]));
    let (lpt, bytes) = (name(LOOKALIKES[1]), LOOKALIKES[1].as_bytes());
    assert_eq!(shelf.load(&lpt, &lpt), Ok(Some(bytes.to_vec())));
}

/// Remove and drop delete at once and leave the rest; a missing blob or folder is no error.
fn deletes(shelf: &dyn SealedShelf) {
    let (day, earlier, e1, d1) = (name(DAY), name(EARLIER), name("e1"), name("d1"));
    shelf.remove(&day, &e1).unwrap();
    assert_eq!(shelf.load(&day, &e1), Ok(None));
    assert_eq!(shelf.names(&day), Ok(vec![d1.clone()]));
    assert_eq!(shelf.remove(&day, &e1), Ok(()), "already gone");
    assert_eq!(shelf.remove(&name(EARLIER), &d1), Ok(()), "never there");
    shelf.remove(&day, &d1).unwrap();
    assert_eq!(shelf.names(&day), Ok(vec![]));
    let kept = shelf.folders().unwrap().contains(&day);
    assert!(kept, "an emptied folder stays until it is dropped");
    shelf.drop_folder(&earlier).unwrap();
    assert_eq!(shelf.load(&earlier, &e1), Ok(None));
    assert_eq!(shelf.drop_folder(&earlier), Ok(()), "already gone");
    let missing = name(NEVER);
    assert_eq!(shelf.remove(&missing, &e1), Ok(()), "no such folder");
    let left = [DAY, LOOKALIKES[0], LOOKALIKES[1], PINNED].map(name);
    assert_eq!(shelf.folders(), Ok(left.to_vec()));
    let pinned = shelf.load(&name(PINNED), &e1).unwrap();
    assert_eq!(pinned.as_deref(), Some(NEW), "other folders stay");
}
