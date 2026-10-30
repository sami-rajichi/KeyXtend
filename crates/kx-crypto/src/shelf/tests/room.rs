//! The room: the sealed blobs' bytes, and the drive's free space and name from the platform.

use super::*;

/// The size of the file at `path`.
fn size(path: &Path) -> u64 {
    fs::metadata(path).unwrap().len()
}

#[test]
fn room_counts_the_sealed_blobs_and_asks_the_drive() {
    let c = case();
    c.shelf.save(&name(DAY), &name("e1"), b"words").unwrap();
    c.shelf.save(&name(PINNED), &name("e2"), b"kept").unwrap();
    fs::write(c.file(DAY, "e3", DAMAGED_EXT), b"set aside").unwrap();
    fs::write(c.root.join(DAY).join("notes.txt"), b"stray").unwrap();
    fs::create_dir_all(c.root.join("Upper")).unwrap();
    fs::write(c.file("Upper", "e4", SEALED_EXT), b"not on the shelf").unwrap();
    let used = size(&c.file(DAY, "e1", SEALED_EXT)) + size(&c.file(PINNED, "e2", SEALED_EXT));
    let want = Room {
        used,
        free: FREE,
        drive: DRIVE.to_owned(),
    };
    assert_eq!(c.shelf.room(), Ok(want));
    assert_eq!(c.disk.asked(), [c.root.clone(), c.root.clone()]);
}

#[test]
fn a_missing_root_uses_nothing_and_asks_about_its_nearest_folder() {
    let c = case();
    assert_eq!(c.shelf.room().map(|room| room.used), Ok(0));
    let parent = c.temp.path().to_owned();
    assert_eq!(c.disk.asked(), [parent.clone(), parent]);
}

#[test]
fn a_drive_that_cannot_be_read_is_io() {
    let c = case();
    let disk = Scripted {
        fail: true,
        ..Scripted::default()
    };
    let guard = Box::new(c.guard.clone());
    let shelf = SealedShelfFiles::new(c.root.clone(), guard, Box::new(disk));
    assert_eq!(shelf.room(), Err(SealedError::Io));
}
