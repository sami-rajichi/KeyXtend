//! Damaged blobs: set aside once as `<name>.damaged`, then gone from loads and lists.

use super::*;
use crate::consts::OVERHEAD;

#[test]
fn a_damaged_blob_is_set_aside_once() {
    let c = case();
    c.shelf.save(&name(DAY), &name("e1"), b"words").unwrap();
    let path = c.file(DAY, "e1", SEALED_EXT);
    let mut bytes = fs::read(&path).unwrap();
    *bytes.last_mut().unwrap() ^= 1;
    fs::write(&path, &bytes).unwrap();
    assert_eq!(
        c.shelf.load(&name(DAY), &name("e1")),
        Err(SealedError::Damaged)
    );
    assert_eq!(c.listing(DAY), ["e1.damaged"]);
    assert_eq!(fs::read(c.file(DAY, "e1", DAMAGED_EXT)).unwrap(), bytes);
    assert_eq!(c.shelf.load(&name(DAY), &name("e1")), Ok(None));
    assert_eq!(c.shelf.names(&name(DAY)), Ok(vec![]));
}

#[test]
fn an_oversized_file_is_set_aside_on_load() {
    let c = case();
    c.shelf.save(&name(DAY), &name("e1"), b"words").unwrap();
    let big = vec![0; MAX_BLOB_BYTES + OVERHEAD + 1];
    fs::write(c.file(DAY, "e1", SEALED_EXT), &big).unwrap();
    assert_eq!(
        c.shelf.load(&name(DAY), &name("e1")),
        Err(SealedError::Damaged)
    );
    assert_eq!(c.listing(DAY), ["e1.damaged"]);
}

#[test]
fn a_set_aside_that_fails_is_io_and_keeps_the_blob() {
    let c = case();
    c.shelf.save(&name(DAY), &name("e1"), b"words").unwrap();
    fs::create_dir_all(c.file(DAY, "e1", DAMAGED_EXT).join("inside")).unwrap();
    fs::write(c.file(DAY, "e1", SEALED_EXT), b"junk").unwrap();
    assert_eq!(c.shelf.load(&name(DAY), &name("e1")), Err(SealedError::Io));
    assert!(c.file(DAY, "e1", SEALED_EXT).is_file());
}

#[test]
fn a_blob_that_cannot_be_read_is_io_and_stays() {
    let c = case();
    fs::create_dir_all(c.file(DAY, "e1", SEALED_EXT)).unwrap();
    assert_eq!(c.shelf.load(&name(DAY), &name("e1")), Err(SealedError::Io));
    assert!(c.file(DAY, "e1", SEALED_EXT).is_dir());
}
