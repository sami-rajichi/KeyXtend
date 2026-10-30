//! The fake shelf, held to the port: folders of blobs that save, load, list and delete as the real shelf does, the
//! blob cap, failures on cue per call, the scripted room, and a folder deleted by hand.

use super::*;
use std::thread;

const OLD: &[u8] = b"old list";
const NEW: &[u8] = "قائمة جديدة".as_bytes();

/// Free bytes the tests set on the drive.
const FREE: u64 = 5_000;

/// A checked test name.
fn name(text: &str) -> BlobName {
    BlobName::new(text).unwrap()
}

/// A shelf holding one blob, `list`, in folder `day`.
fn planted(day: &str) -> FakeShelf {
    let shelf = FakeShelf::new();
    shelf.plant(&name(day), &name("list"), OLD.to_vec());
    shelf
}

#[test]
fn save_then_load_keeps_blobs_apart_by_folder_and_name() {
    let shelf = FakeShelf::new();
    let (day, pinned, list) = (name("2026-10-31"), name("pinned"), name("list"));
    assert_eq!(shelf.load(&day, &list), Ok(None), "nothing saved yet");
    assert_eq!(shelf.save(&day, &list, OLD), Ok(()));
    assert_eq!(shelf.save(&pinned, &list, NEW), Ok(()));
    assert_eq!(shelf.load(&day, &list), Ok(Some(OLD.to_vec())));
    assert_eq!(shelf.load(&pinned, &list), Ok(Some(NEW.to_vec())));
    assert_eq!(shelf.save(&day, &list, NEW), Ok(()));
    assert_eq!(
        shelf.blob(&day, &list),
        Some(NEW.to_vec()),
        "a save replaces"
    );
}

#[test]
fn folders_and_names_list_sorted_and_a_missing_folder_has_none() {
    let shelf = FakeShelf::new();
    for (folder, blob) in [
        ("pinned", "list"),
        ("2026-10-31", "7-0"),
        ("2026-10-30", "list"),
    ] {
        shelf.save(&name(folder), &name(blob), OLD).unwrap();
    }
    shelf.save(&name("2026-10-31"), &name("list"), OLD).unwrap();
    let folders = ["2026-10-30", "2026-10-31", "pinned"].map(name);
    assert_eq!(shelf.folders(), Ok(folders.to_vec()));
    let names = ["7-0", "list"].map(name);
    assert_eq!(shelf.names(&name("2026-10-31")), Ok(names.to_vec()));
    assert_eq!(shelf.names(&name("2026-10-01")), Ok(Vec::new()));
}

#[test]
fn remove_and_drop_delete_at_once_and_missing_ones_are_no_error() {
    let shelf = planted("2026-10-31");
    let (day, list) = (name("2026-10-31"), name("list"));
    shelf.plant(&day, &name("7-0"), NEW.to_vec());
    assert_eq!(shelf.remove(&day, &list), Ok(()));
    assert_eq!(shelf.remove(&day, &list), Ok(()), "already gone");
    assert_eq!(shelf.names(&day), Ok(vec![name("7-0")]));
    assert_eq!(shelf.drop_folder(&day), Ok(()));
    assert_eq!(shelf.drop_folder(&day), Ok(()), "already gone");
    assert_eq!(shelf.folders(), Ok(Vec::new()));
    assert_eq!(shelf.load(&day, &name("7-0")), Ok(None));
}

#[test]
fn a_blob_past_the_cap_is_refused_as_io_and_the_old_one_kept() {
    let shelf = FakeShelf::new();
    let (day, part) = (name("2026-10-31"), name("7-0"));
    assert_eq!(
        shelf.save(&day, &part, &vec![0; MAX_BLOB_BYTES]),
        Ok(()),
        "at the cap"
    );
    let big = vec![0; MAX_BLOB_BYTES + 1];
    assert_eq!(shelf.save(&day, &part, &big), Err(SealedError::Io));
    assert_eq!(
        shelf.blob(&day, &part).map(|b| b.len()),
        Some(MAX_BLOB_BYTES)
    );
}

#[test]
fn each_scripted_failure_hits_its_own_call_once() {
    let shelf = planted("2026-10-31");
    let (day, list) = (name("2026-10-31"), name("list"));
    shelf.fail(ShelfCall::Save, SealedError::Locked);
    assert_eq!(
        shelf.load(&day, &list),
        Ok(Some(OLD.to_vec())),
        "not the load"
    );
    assert_eq!(shelf.save(&day, &list, NEW), Err(SealedError::Locked));
    assert_eq!(shelf.blob(&day, &list), Some(OLD.to_vec()), "kept");
    assert_eq!(shelf.save(&day, &list, NEW), Ok(()), "once");
    let io = SealedError::Io;
    for call in [
        ShelfCall::Remove,
        ShelfCall::DropFolder,
        ShelfCall::Folders,
        ShelfCall::Names,
        ShelfCall::Room,
    ] {
        shelf.fail(call, io);
    }
    assert_eq!(shelf.remove(&day, &list), Err(io));
    assert_eq!(shelf.drop_folder(&day), Err(io));
    assert_eq!(shelf.folders(), Err(io));
    assert_eq!(shelf.names(&day), Err(io));
    assert_eq!(shelf.room(), Err(io));
    assert_eq!(
        shelf.blob(&day, &list),
        Some(NEW.to_vec()),
        "nothing deleted"
    );
    assert_eq!(shelf.folders(), Ok(vec![day]));
}

#[test]
fn a_scripted_locked_load_comes_back_once() {
    let shelf = planted("2026-10-31");
    let (day, list) = (name("2026-10-31"), name("list"));
    shelf.fail(ShelfCall::Load, SealedError::Locked);
    assert_eq!(shelf.load(&day, &list), Err(SealedError::Locked));
    assert_eq!(shelf.load(&day, &list), Ok(Some(OLD.to_vec())));
}

#[test]
fn a_scripted_damaged_load_comes_back_once_and_sets_the_blob_aside() {
    let shelf = planted("2026-10-31");
    let (day, list) = (name("2026-10-31"), name("list"));
    shelf.fail(ShelfCall::Load, SealedError::Damaged);
    assert_eq!(shelf.load(&day, &list), Err(SealedError::Damaged));
    assert_eq!(shelf.load(&day, &list), Ok(None), "set aside");
    assert_eq!(shelf.names(&day), Ok(Vec::new()));
}

#[test]
fn the_room_tells_the_bytes_held_and_the_free_bytes_set() {
    let shelf = FakeShelf::new();
    let roomy = shelf.room().unwrap();
    assert_eq!(roomy.used, 0);
    assert!(roomy.free > FREE, "a roomy drive by default");
    shelf.set_free(FREE);
    shelf.set_drive("E:");
    shelf.save(&name("2026-10-31"), &name("list"), OLD).unwrap();
    shelf.save(&name("pinned"), &name("list"), NEW).unwrap();
    let used = u64::try_from(OLD.len() + NEW.len()).unwrap();
    let room = Room {
        used,
        free: FREE,
        drive: "E:".to_owned(),
    };
    assert_eq!(shelf.room(), Ok(room));
}

#[test]
fn a_folder_deleted_by_hand_is_gone_without_a_call() {
    let shelf = planted("2026-10-30");
    shelf.plant(&name("2026-10-31"), &name("list"), NEW.to_vec());
    shelf.delete_by_hand(&name("2026-10-30"));
    assert_eq!(shelf.calls(ShelfCall::DropFolder), 0);
    assert_eq!(shelf.folders(), Ok(vec![name("2026-10-31")]));
    assert_eq!(shelf.load(&name("2026-10-30"), &name("list")), Ok(None));
}

#[test]
fn calls_are_counted_by_kind_refused_ones_too() {
    let shelf = planted("2026-10-31");
    let (day, list) = (name("2026-10-31"), name("list"));
    assert_eq!(shelf.calls(ShelfCall::Save), 0, "a plant is no save");
    shelf.fail(ShelfCall::Load, SealedError::Io);
    let _ = shelf.load(&day, &list);
    let _ = shelf.load(&day, &list);
    let _ = shelf.save(&day, &list, NEW);
    let _ = shelf.room();
    let counts = [
        ShelfCall::Load,
        ShelfCall::Save,
        ShelfCall::Room,
        ShelfCall::Names,
    ]
    .map(|c| shelf.calls(c));
    assert_eq!(counts, [2, 1, 1, 0]);
}

#[test]
fn clones_share_the_shelf_and_serve_as_the_port_from_another_thread() {
    let shelf = FakeShelf::new();
    let port: Arc<dyn SealedShelf> = Arc::new(shelf.clone());
    let saved = thread::spawn(move || port.save(&name("pinned"), &name("list"), OLD));
    assert_eq!(saved.join().unwrap(), Ok(()));
    assert_eq!(
        shelf.blob(&name("pinned"), &name("list")),
        Some(OLD.to_vec())
    );
}

#[test]
fn debug_names_folders_but_never_prints_the_bytes() {
    let shelf = planted("2026-10-31");
    let text = format!("{shelf:?}");
    assert!(text.contains("2026-10-31"));
    assert!(!text.contains(&*String::from_utf8_lossy(OLD)));
}

#[cfg(debug_assertions)]
#[test]
#[should_panic(expected = "never returns")]
fn an_error_the_call_never_returns_is_a_test_mistake() {
    FakeShelf::new().fail(ShelfCall::Remove, SealedError::Locked);
}
