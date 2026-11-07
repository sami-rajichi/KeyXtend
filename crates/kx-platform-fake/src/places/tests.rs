//! The fake places, held to the port: suggested places in a stable order, places that vanish and come back, fill up,
//! turn read-only or hold the data folder, writes in one step, capped reads, the folder list, failures on cue, counted
//! calls, and no path in `Debug`.

use super::*;
use kx_module_api::ports::{BOX_EXT, BOX_MARK, BOX_VERSION};
use std::path::PathBuf;
use std::thread;

mod find;
mod lists;

// The test places; "someone" stands for the user's name.

/// A cloud folder.
const ONEDRIVE: &str = "Users/someone/OneDrive";
/// A second cloud folder.
const DROPBOX: &str = "Users/someone/Dropbox";
/// A USB stick.
const USB: &str = "usb-e";
/// A fixed drive.
const DRIVE: &str = "drive-c";
/// A plain folder, which is not suggested.
const PLAIN: &str = "Users/someone/Documents";

/// The data folder, beside the cloud folders.
const DATA: &str = "Users/someone/KeyXtend";

/// A read cap larger than any test file.
const CAP: u64 = 1_000;

/// An old box's bytes after the head.
const OLD: &[u8] = b"old box";
/// A new box's bytes after the head ("new box").
const NEW: &[u8] = "صندوق جديد".as_bytes();

/// A box's bytes: its mark and version, then `tail`.
fn boxed(tail: &[u8]) -> Vec<u8> {
    [&BOX_MARK[..], &[BOX_VERSION], tail].concat()
}

/// The box's file in `folder`.
fn box_in(folder: &str) -> PathBuf {
    Path::new(folder).join(box_file())
}

/// A place of `kind` named `name` at `path`.
fn place(kind: PlaceKind, name: &str, path: impl Into<PathBuf>) -> Place {
    Place {
        kind,
        name: Redacted::new(name.to_owned()),
        path: Redacted::new(path.into()),
    }
}

/// The `OneDrive` place.
fn onedrive() -> Place {
    place(PlaceKind::Cloud, "OneDrive", ONEDRIVE)
}

/// The USB stick's place.
fn usb() -> Place {
    place(PlaceKind::Removable, "E:", USB)
}

/// Places with `OneDrive`, the USB stick and a plain folder.
fn places() -> FakePlaces {
    let places = FakePlaces::new();
    places.add(PlaceKind::Cloud, "OneDrive", Path::new(ONEDRIVE));
    places.add(PlaceKind::Removable, "E:", Path::new(USB));
    places.add_folder(Path::new(PLAIN));
    places
}

/// The bytes' length as the port counts it.
fn len(bytes: &[u8]) -> u64 {
    u64::try_from(bytes.len()).unwrap()
}

#[test]
fn suggested_lists_cloud_folders_in_order_then_removable_drives() {
    let places = FakePlaces::new();
    places.add(PlaceKind::Removable, "E:", Path::new(USB));
    places.add(PlaceKind::Cloud, "OneDrive", Path::new(ONEDRIVE));
    places.add(PlaceKind::Drive, "C:", Path::new(DRIVE));
    places.add(PlaceKind::Cloud, "Dropbox", Path::new(DROPBOX));
    let dropbox = place(PlaceKind::Cloud, "Dropbox", DROPBOX);
    let want = [onedrive(), dropbox, usb()];
    assert_eq!(places.suggested(), want);
    assert_eq!(places.suggested(), want, "stable");
    let drive = place(PlaceKind::Drive, "C:", DRIVE);
    assert_eq!(places.folders(None), Ok(vec![usb(), drive]), "drives lead");
}

#[test]
fn a_vanished_place_is_missing_everywhere_until_it_comes_back_whole() {
    let places = places();
    let (stick, file) = (Path::new(USB), box_in(USB));
    places.plant(&file, boxed(OLD), 1);
    places.vanish(stick);
    assert_eq!(places.suggested(), [onedrive()]);
    assert_eq!(places.state(stick, 1), PlaceState::Missing);
    assert_eq!(places.write(stick, &boxed(NEW)), Err(PlaceError::Missing));
    assert_eq!(places.read(&file, CAP), Err(PlaceError::Missing));
    assert_eq!(places.folders(Some(stick)), Err(PlaceError::Missing));
    assert_eq!(places.files(stick, BOX_EXT), Err(PlaceError::Missing));
    assert_eq!(places.folders(None), Ok(Vec::new()), "no drive left");
    assert!(places.find().is_empty());
    places.come_back(stick);
    assert_eq!(places.suggested(), [onedrive(), usb()], "in its place");
    let read = places.read(&file, CAP);
    assert_eq!(read, Ok(Redacted::new(boxed(OLD))), "whole");
    assert_eq!(places.state(stick, 1), PlaceState::Ready);
}

#[test]
fn a_full_place_refuses_a_write_and_keeps_the_old_box() {
    let places = places();
    let (stick, file) = (Path::new(USB), box_in(USB));
    places.plant(&file, boxed(OLD), 1);
    let need = len(&boxed(NEW));
    places.set_free(stick, need - 1);
    assert_eq!(places.state(stick, need), PlaceState::Full);
    assert_eq!(places.state(stick, need - 1), PlaceState::Ready);
    assert_eq!(places.write(stick, &boxed(NEW)), Err(PlaceError::Full));
    assert_eq!(places.file(&file), Some(boxed(OLD)), "kept");
    places.set_free(stick, need);
    assert_eq!(places.write(stick, &boxed(NEW)), Ok(()));
    assert_eq!(places.file(&file), Some(boxed(NEW)));
}

#[test]
fn a_write_that_fails_half_way_keeps_the_old_box_whole() {
    let places = places();
    let (cloud, file) = (Path::new(ONEDRIVE), box_in(ONEDRIVE));
    places.plant(&file, boxed(OLD), 1);
    places.fail(PlaceCall::Write, PlaceError::Io);
    assert_eq!(places.write(cloud, &boxed(NEW)), Err(PlaceError::Io));
    assert_eq!(places.file(&file), Some(boxed(OLD)), "whole");
    let listed = places.files(cloud, BOX_EXT);
    assert_eq!(listed, Ok(vec![place(PlaceKind::File, &box_file(), &file)]));
    assert_eq!(places.write(cloud, &boxed(NEW)), Ok(()), "once");
}

#[test]
fn a_write_replaces_the_box_in_one_step_and_stamps_it_with_the_time_set() {
    let places = places();
    let cloud = Path::new(ONEDRIVE);
    places.set_now(5);
    assert_eq!(places.write(cloud, &boxed(OLD)), Ok(()));
    places.set_now(9);
    assert_eq!(places.write(cloud, &boxed(NEW)), Ok(()));
    assert_eq!(places.file(&box_in(ONEDRIVE)), Some(boxed(NEW)));
    let found = Found {
        place: onedrive(),
        path: Redacted::new(box_in(ONEDRIVE)),
        modified_ms: 9,
    };
    assert_eq!(places.find(), [found]);
}

#[test]
fn a_read_only_place_and_the_data_folder_with_its_parents_and_children_refuse_writes() {
    let places = places();
    let stick = Path::new(USB);
    places.set_read_only(stick, true);
    assert_eq!(places.write(stick, OLD), Err(PlaceError::ReadOnly));
    places.add_folder(&stick.join("inside"));
    let inside = places.write(&stick.join("inside"), OLD);
    assert_eq!(inside, Err(PlaceError::ReadOnly), "below too");
    places.set_read_only(stick, false);
    assert_eq!(places.write(stick, OLD), Ok(()));
    let data = Path::new(DATA);
    let child = data.join("quickfill");
    places.add_folder(&child);
    places.set_data_dir(data);
    for folder in [data, data.parent().unwrap(), &child] {
        let refused = places.write(folder, OLD);
        assert_eq!(refused, Err(PlaceError::Refused), "{folder:?}");
    }
    assert_eq!(places.write(Path::new(ONEDRIVE), OLD), Ok(()), "a sibling");
}

#[test]
fn a_read_over_the_cap_is_refused_and_a_deleted_file_is_missing() {
    let places = places();
    let file = box_in(ONEDRIVE);
    let bytes = boxed(OLD);
    let cap = len(&bytes);
    places.plant(&file, bytes.clone(), 1);
    assert_eq!(places.read(&file, cap), Ok(Redacted::new(bytes)));
    assert_eq!(places.read(&file, cap - 1), Err(PlaceError::TooLarge));
    places.delete_by_hand(&file);
    assert_eq!(places.read(&file, cap), Err(PlaceError::Missing));
    assert_eq!(
        places.calls(PlaceCall::Read),
        3,
        "a delete by hand is no call"
    );
}

#[test]
fn each_scripted_failure_hits_its_own_call_once() {
    let places = places();
    let (cloud, file) = (Path::new(ONEDRIVE), box_in(ONEDRIVE));
    places.plant(&file, boxed(OLD), 1);
    places.fail(PlaceCall::Folders, PlaceError::Io);
    places.fail(PlaceCall::Files, PlaceError::Missing);
    places.fail(PlaceCall::Read, PlaceError::Io);
    assert_eq!(places.folders(Some(cloud)), Err(PlaceError::Io));
    assert_eq!(places.files(cloud, BOX_EXT), Err(PlaceError::Missing));
    assert_eq!(places.read(&file, CAP), Err(PlaceError::Io));
    for error in [
        PlaceError::Missing,
        PlaceError::Full,
        PlaceError::ReadOnly,
        PlaceError::Refused,
        PlaceError::Io,
    ] {
        places.fail(PlaceCall::Write, error);
        assert_eq!(places.write(cloud, &boxed(NEW)), Err(error));
    }
    assert_eq!(places.file(&file), Some(boxed(OLD)), "kept");
    assert!(places.folders(Some(cloud)).is_ok(), "once");
    assert!(places.files(cloud, BOX_EXT).is_ok());
    assert!(places.read(&file, CAP).is_ok());
    assert_eq!(places.write(cloud, &boxed(NEW)), Ok(()));
}

#[test]
fn calls_are_counted_by_kind_refused_ones_too() {
    let places = places();
    let file = box_in(ONEDRIVE);
    places.plant(&file, boxed(OLD), 1);
    places.fail(PlaceCall::Read, PlaceError::Io);
    let _ = places.read(&file, CAP);
    let _ = places.read(&file, CAP);
    let _ = places.modified(&file);
    let _ = places.suggested();
    let _ = places.find();
    let _ = places.state(Path::new(USB), 1);
    let kinds = [
        PlaceCall::Suggested,
        PlaceCall::Folders,
        PlaceCall::Files,
        PlaceCall::Find,
        PlaceCall::Read,
        PlaceCall::Write,
        PlaceCall::State,
        PlaceCall::Modified,
    ];
    assert_eq!(kinds.map(|c| places.calls(c)), [1, 0, 0, 1, 2, 0, 1, 1]);
}

#[test]
fn clones_share_the_tree_and_serve_as_the_port_from_another_thread() {
    let places = places();
    let port: Arc<dyn Places> = Arc::new(places.clone());
    let wrote = thread::spawn(move || port.write(Path::new(USB), &boxed(NEW)));
    assert_eq!(wrote.join().unwrap(), Ok(()));
    assert_eq!(places.file(&box_in(USB)), Some(boxed(NEW)));
}

#[test]
fn debug_counts_but_prints_no_path_name_or_bytes() {
    let places = places();
    places.add(PlaceKind::Cloud, "ملفاتي", Path::new(DROPBOX));
    places.plant(&box_in(ONEDRIVE), boxed(NEW), 1);
    let _ = places.find();
    let text = format!("{places:?}");
    assert!(text.starts_with("FakePlaces"), "{text}");
    assert!(text.contains("Find"), "{text}");
    for secret in ["someone", "ملفاتي", "OneDrive", "صندوق"] {
        assert!(!text.contains(secret), "{text} shows {secret}");
    }
}

#[cfg(debug_assertions)]
#[test]
#[should_panic(expected = "never returns")]
fn an_error_the_call_never_returns_is_a_test_mistake() {
    FakePlaces::new().fail(PlaceCall::Read, PlaceError::Full);
}

#[test]
fn the_data_folder_its_parents_and_children_have_the_refused_state() {
    let places = places();
    let data = Path::new(DATA);
    let child = data.join("quickfill");
    places.add_folder(&child);
    places.set_data_dir(data);
    for folder in [data, data.parent().unwrap(), &child] {
        assert_eq!(places.state(folder, 0), PlaceState::Refused, "{folder:?}");
    }
    assert_eq!(
        places.state(Path::new(ONEDRIVE), 0),
        PlaceState::Ready,
        "a sibling"
    );
}
