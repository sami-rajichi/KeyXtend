//! The backup blob over the fake sealed store: a round trip, an empty start, a damaged blob set aside with its own
//! notice, a newer version left alone, and its pinned bytes.

use std::path::PathBuf;
use std::sync::Arc;

use super::codec::{MARK, decode, encode};
use super::*;
use crate::entry::Stamp;
use crate::store::{Bad, StoreError};
use kx_module_api::Redacted;
use kx_module_api::ports::{BlobName, PlaceKind, SealedError};
use kx_platform_fake::FakeSealedStore;

/// The backup blob's name, and the name of its set-aside copy.
fn names() -> (BlobName, BlobName) {
    (BlobName::new(BLOB).unwrap(), BlobName::new(ASIDE).unwrap())
}

/// A place of `kind` named `name` at `path`.
pub(super) fn place(kind: PlaceKind, name: &str, path: &str) -> Place {
    Place {
        kind,
        name: Redacted::new(name.to_owned()),
        path: Redacted::new(PathBuf::from(path)),
    }
}

/// A padlock of plain test bytes.
fn padlock() -> Padlock {
    Padlock {
        recipient: Redacted::new("age1test".to_owned()),
        locked: Redacted::new(vec![1, 2, 3]),
    }
}

/// A blob with two places, one written and one waiting, a padlock, a left mark, and checks under way.
fn sample() -> Kept {
    let cloud = Site {
        place: place(PlaceKind::Cloud, "OneDrive", "C:\\Users\\a\\OneDrive"),
        written: Some(Stamp::from_unix_ms(1_762_000_000_000)),
        mark: Some(0xfeed),
        len: 4096,
        waiting: false,
    };
    let stick = Site {
        place: place(PlaceKind::Removable, "E: مفتاح", "E:\\"),
        written: None,
        mark: None,
        len: 0,
        waiting: true,
    };
    Kept {
        sites: vec![cloud, stick],
        padlock: Some(padlock()),
        left: vec![7, 9],
        offered: true,
        behind: true,
        held: true,
        checks: Checks {
            due: Some(Stamp::from_unix_ms(5)),
            step: 2,
            missed: 1,
        },
    }
}

/// A store over `sealed`.
fn store(sealed: &FakeSealedStore) -> BackupStore {
    BackupStore::new(Arc::new(sealed.clone()))
}

#[test]
fn the_blob_comes_back_as_it_was_saved() {
    let sealed = FakeSealedStore::new();
    store(&sealed).save(&sample()).unwrap();
    let loaded = store(&sealed).load().unwrap();
    assert_eq!((loaded.kept, loaded.damaged), (sample(), false));
}

#[test]
fn nothing_saved_starts_clean_and_says_nothing() {
    let loaded = store(&FakeSealedStore::new()).load().unwrap();
    assert_eq!((loaded.kept, loaded.damaged), (Kept::default(), false));
}

#[test]
fn a_damaged_blob_is_set_aside_and_starts_clean() {
    let (sealed, (blob, aside)) = (FakeSealedStore::new(), names());
    sealed.plant(&blob, b"not a backup".to_vec());
    let mut store = store(&sealed);
    let loaded = store.load().unwrap();
    assert_eq!((loaded.kept, loaded.damaged), (Kept::default(), true));
    assert_eq!(sealed.blob(&aside), Some(b"not a backup".to_vec()));
    store.save(&sample()).unwrap();
}

#[test]
fn a_blob_the_sealed_store_set_aside_starts_clean() {
    let sealed = FakeSealedStore::new();
    store(&sealed).save(&sample()).unwrap();
    sealed.fail_load(SealedError::Damaged);
    assert!(store(&sealed).load().unwrap().damaged);
}

#[test]
fn a_newer_versions_blob_is_refused_and_left_alone() {
    let (sealed, (blob, _)) = (FakeSealedStore::new(), names());
    let mut newer = encode(&sample()).to_vec();
    newer[MARK.len()] += 1;
    sealed.plant(&blob, newer.clone());
    assert_eq!(store(&sealed).load().err(), Some(StoreError::Newer));
    assert_eq!(sealed.blob(&blob), Some(newer));
}

#[test]
fn the_blob_and_the_notebook_keep_their_own_names() {
    assert_eq!((BLOB, ASIDE), ("backup", "backup-aside"));
}

#[test]
fn the_format_refuses_what_it_never_writes() {
    let good = encode(&sample()).to_vec();
    let cut = &good[..good.len() - 1];
    let longer = [good.as_slice(), &[0]].concat();
    let mut flag = good.clone();
    flag[MARK.len() + 1] = 2;
    for bad in [cut, longer.as_slice(), flag.as_slice(), b"KXBK", b""] {
        assert_eq!(decode(bad), Err(Bad::Damaged), "{} bytes", bad.len());
    }
}

/// A blob with one written folder, held, no padlock and nothing else, with fixed values.
fn golden() -> Kept {
    Kept {
        sites: vec![Site {
            place: place(PlaceKind::Folder, "B", "D:\\B"),
            written: Some(Stamp::from_unix_ms(3)),
            mark: Some(9),
            len: 700,
            waiting: false,
        }],
        offered: true,
        held: true,
        ..Kept::default()
    }
}

/// The bytes `golden` saves as; a change here is a change of format, which needs a new version.
const GOLDEN: &[u8] = b"KXBK\x01\x01\x00\x01\x00\x00\x00\x00\
\x01\0\0\0\x03\x01\0\0\0B\x04\0\0\0D:\\B\x01\x03\0\0\0\0\0\0\0\x01\x09\0\0\0\0\0\0\0\xbc\x02\0\0\0\0\0\0\
\x00\0\0\0\0";

#[test]
fn the_format_is_pinned_byte_for_byte() {
    assert_eq!(encode(&golden()).as_slice(), GOLDEN);
    assert_eq!(decode(GOLDEN), Ok(golden()));
}
