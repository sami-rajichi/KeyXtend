//! The real shelf and the fake agree: `BlobName` gates both, and the same calls leave the same folders, names and
//! bytes, refuse the same oversized blob, and count the same room but for the seal's overhead.

use super::*;
use crate::consts::OVERHEAD;
use kx_module_api::ports::RESERVED_NAMES;
use kx_platform_fake::FakeShelf;

/// Names both shelves keep, a device name's lookalikes among them.
const GOOD: [&str; 4] = [DAY, PINNED, "con-1", "lpt10"];
/// Names of the wrong form.
const BAD: [&str; 5] = ["", "Pinned", "a/b", "..", "CON"];

/// A folder's blobs, each with its bytes.
type Blobs = Vec<(BlobName, Vec<u8>)>;

/// Every folder of `shelf` with its blobs, as it lists and loads them.
fn seen(shelf: &dyn SealedShelf) -> Vec<(BlobName, Blobs)> {
    let blobs = |folder: &BlobName| -> Blobs {
        let names = shelf.names(folder).unwrap().into_iter();
        names
            .map(|n| (n.clone(), shelf.load(folder, &n).unwrap().unwrap()))
            .collect()
    };
    let folders = shelf.folders().unwrap().into_iter();
    folders.map(|f| (f.clone(), blobs(&f))).collect()
}

/// The calls both shelves get: every folder and name pair, an oversized save, and removes and drops, missing ones too.
fn run(shelf: &dyn SealedShelf, names: &[BlobName]) {
    for folder in names {
        for blob in names {
            shelf.save(folder, blob, blob.as_str().as_bytes()).unwrap();
        }
    }
    let big = vec![0; MAX_BLOB_BYTES + 1];
    assert_eq!(
        shelf.save(&name(DAY), &name("e1"), &big),
        Err(SealedError::Io)
    );
    assert_eq!(shelf.load(&name(DAY), &name("e1")), Ok(None));
    shelf.remove(&name(DAY), &name(PINNED)).unwrap();
    shelf.remove(&name(DAY), &name("e1")).unwrap();
    shelf.drop_folder(&name("con-1")).unwrap();
    shelf.drop_folder(&name("2026-01-01")).unwrap();
    assert_eq!(shelf.names(&name("con-1")), Ok(vec![]));
}

#[test]
fn the_fake_and_the_real_shelf_agree() {
    let c = case();
    let fake = FakeShelf::new();
    let texts = GOOD.iter().chain(&BAD).chain(RESERVED_NAMES);
    let names: Vec<BlobName> = texts.filter_map(|t| BlobName::new(t).ok()).collect();
    assert_eq!(names.len(), GOOD.len(), "only the good names build");
    run(&c.shelf, &names);
    run(&fake, &names);
    let real = seen(&c.shelf);
    assert_eq!(real, seen(&fake));
    assert_eq!(real.len(), GOOD.len() - 1, "one folder was dropped");
    let blobs = real.iter().map(|(_, b)| b.len()).sum::<usize>();
    let overhead = u64::try_from(blobs * OVERHEAD).unwrap();
    let (mine, theirs) = (c.shelf.room().unwrap(), fake.room().unwrap());
    assert_eq!(mine.used, theirs.used + overhead, "the fake counts no seal");
}
