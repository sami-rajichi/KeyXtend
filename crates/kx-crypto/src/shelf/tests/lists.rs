//! Listing: only entries named by a `BlobName` are listed, sorted, and anything else on disk is left out.

use super::*;
use kx_module_api::ports::MAX_BLOB_LEN;

#[test]
fn a_missing_root_or_folder_lists_nothing() {
    let c = case();
    assert_eq!(c.shelf.folders(), Ok(vec![]));
    assert_eq!(c.shelf.names(&name(DAY)), Ok(vec![]));
    assert!(!c.root.exists());
}

#[test]
fn folders_lists_only_folders_with_blob_names() {
    let c = case();
    c.shelf.save(&name(PINNED), &name("e1"), b"kept").unwrap();
    let long = "a".repeat(MAX_BLOB_LEN + 1);
    for dir in [DAY, "2026-10-30", "Upper", "x.y", "a b", "école", &long] {
        fs::create_dir_all(c.root.join(dir)).unwrap();
    }
    fs::write(c.root.join("2026-10-29"), b"a file, not a folder").unwrap();
    let want = vec![name("2026-10-30"), name(DAY), name(PINNED)];
    assert_eq!(c.shelf.folders(), Ok(want));
}

#[test]
fn names_lists_only_sealed_blobs_with_blob_names() {
    let c = case();
    c.shelf.save(&name(DAY), &name("b"), b"two").unwrap();
    c.shelf.save(&name(DAY), &name("a"), b"one").unwrap();
    let dir = c.root.join(DAY);
    let strays = [
        "a.damaged",
        "a.sealed.tmp",
        "Bad.sealed",
        "x.y.sealed",
        "c.txt",
        "sealed",
    ];
    for file in strays {
        fs::write(dir.join(file), b"stray").unwrap();
    }
    fs::create_dir_all(dir.join("d.sealed")).unwrap();
    assert_eq!(c.shelf.names(&name(DAY)), Ok(vec![name("a"), name("b")]));
}

/// The shelf joins folder names to its root unchecked, so it relies on no path, drive, device or upper-case
/// lookalike ever building a `BlobName`.
#[test]
fn path_like_folder_names_never_build() {
    let paths = [
        "..",
        "../up",
        "a/b",
        "a\\b",
        "c:",
        "2026-10-31.",
        "con",
        "Pinned",
    ];
    for text in paths {
        assert_eq!(BlobName::new(text), Err(SealedError::BadName), "{text:?}");
    }
}
