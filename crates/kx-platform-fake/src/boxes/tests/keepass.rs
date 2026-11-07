//! `KeePass` files as the worker reads them: a set file's entries and drops for its password only, bytes never set as
//! a file that is not `KeePass`, and a file that also needs a key file, which every password fails.

use super::*;
use kx_module_api::ports::Dropped;

/// Another file's bytes.
const OTHER: &[u8] = b"another kdbx file";

#[test]
fn keepass_hands_back_a_set_files_entries_for_its_password_only() {
    let (boxes, mut run) = session();
    let none = run.keepass(KDBX, &password(0));
    assert_eq!(none, Err(BoxError::Damaged), "nothing set: not KeePass");
    boxes.set_keepass(KDBX, PASSWORDS[1], entries());
    assert_eq!(run.keepass(KDBX, &password(1)), Ok(entries().into()));
    let wrong = run.keepass(KDBX, &password(0));
    assert_eq!(wrong, Err(BoxError::WrongPassword));
    let other = run.keepass(OTHER, &password(1));
    assert_eq!(other, Err(BoxError::Damaged), "other bytes are no file set");
}

#[test]
fn setting_a_file_again_replaces_its_password_and_entries() {
    let (boxes, mut run) = session();
    boxes.set_keepass(KDBX, PASSWORDS[0], entries());
    boxes.set_keepass(OTHER, PASSWORDS[0], entries());
    boxes.set_keepass(KDBX, PASSWORDS[1], Vec::new());
    assert_eq!(
        run.keepass(KDBX, &password(1)),
        Ok(KeepassImport::default())
    );
    let old = run.keepass(KDBX, &password(0));
    assert_eq!(old, Err(BoxError::WrongPassword));
    assert_eq!(
        run.keepass(OTHER, &password(0)),
        Ok(entries().into()),
        "kept"
    );
}

#[test]
fn an_import_set_with_drops_hands_them_back_until_the_file_is_set_again() {
    let (boxes, mut run) = session();
    let dropped = Dropped {
        entries: 2,
        fields: 3,
    };
    let import = KeepassImport {
        entries: entries(),
        dropped,
    };
    boxes.set_import(KDBX, PASSWORDS[0], import.clone());
    assert_eq!(run.keepass(KDBX, &password(0)), Ok(import));
    let wrong = run.keepass(KDBX, &password(1));
    assert_eq!(wrong, Err(BoxError::WrongPassword));
    boxes.set_keepass(KDBX, PASSWORDS[0], entries());
    let again = run.keepass(KDBX, &password(0));
    assert_eq!(again, Ok(entries().into()), "nothing dropped");
}

#[test]
fn a_file_that_needs_a_key_file_fails_every_password_as_wrong() {
    let (boxes, mut run) = session();
    boxes.set_keepass(KDBX, PASSWORDS[0], entries());
    boxes.set_key_file(KDBX);
    for n in 0..PASSWORDS.len() {
        let read = run.keepass(KDBX, &password(n));
        assert_eq!(read, Err(BoxError::WrongPassword), "{n}");
    }
}
