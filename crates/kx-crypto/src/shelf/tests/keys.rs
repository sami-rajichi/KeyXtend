//! The shelf's one data key: shared by every folder, lost, or locked.

use super::*;

#[test]
fn one_key_serves_every_folder_and_unwraps_once_per_run() {
    let c = case();
    c.shelf.save(&name(DAY), &name("e1"), b"day").unwrap();
    c.shelf.save(&name(PINNED), &name("e1"), b"pin").unwrap();
    let guard = Mirror::default();
    let later = c.reopen(&guard);
    for folder in [DAY, PINNED] {
        assert!(later.load(&name(folder), &name("e1")).unwrap().is_some());
    }
    later
        .save(&name("2026-11-01"), &name("e1"), b"new")
        .unwrap();
    assert_eq!(guard.calls(), 1);
    assert_eq!(c.listing("").iter().filter(|f| *f == KEY_FILE).count(), 1);
}

#[test]
fn a_lost_key_file_makes_a_new_key_and_old_blobs_damaged() {
    let c = case();
    c.shelf.save(&name(DAY), &name("e1"), b"one").unwrap();
    c.shelf.save(&name(PINNED), &name("e2"), b"two").unwrap();
    fs::remove_file(c.key_file()).unwrap();
    let later = c.reopen(&Mirror::default());
    assert_eq!(
        later.load(&name(DAY), &name("e1")),
        Err(SealedError::Damaged)
    );
    later.save(&name(DAY), &name("e3"), b"three").unwrap();
    assert!(c.key_file().is_file());
    assert_eq!(
        later.load(&name(PINNED), &name("e2")),
        Err(SealedError::Damaged)
    );
    assert!(later.load(&name(DAY), &name("e3")).unwrap().is_some());
}

#[test]
fn a_locked_guard_is_locked_and_keeps_every_file() {
    let c = case();
    c.shelf.save(&name(DAY), &name("e1"), b"words").unwrap();
    let blob = fs::read(c.file(DAY, "e1", SEALED_EXT)).unwrap();
    let later = c.reopen(&Mirror::refusing());
    assert_eq!(
        later.load(&name(DAY), &name("e1")),
        Err(SealedError::Locked)
    );
    assert_eq!(
        later.save(&name(DAY), &name("e1"), b"new"),
        Err(SealedError::Locked)
    );
    assert_eq!(fs::read(c.file(DAY, "e1", SEALED_EXT)).unwrap(), blob);
    assert_eq!(c.listing(DAY), ["e1.sealed"]);
}
