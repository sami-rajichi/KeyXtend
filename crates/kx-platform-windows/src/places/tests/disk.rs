//! Reads, writes and states on the real file system: a read over the cap refused before the file opens, a write that
//! replaces the box in one step or leaves it whole, missing and full places, and the data folder refused.

use super::*;
use crate::testlinks::junction;
use kx_module_api::ports::{PlaceError, PlaceState};
use kx_platform::APP_DIR_NAME;
use std::fs::OpenOptions;
use std::os::windows::fs::OpenOptionsExt;
use tree::{boxed, names, plant};

/// The bytes' length as the port counts it.
fn len(bytes: &[u8]) -> u64 {
    u64::try_from(bytes.len()).unwrap()
}

#[test]
fn a_read_over_the_cap_is_refused_before_the_file_is_opened() {
    let tree = Tree::new("places-read");
    let file = tree.make("OneDrive").join(box_file());
    let bytes = boxed(b"box");
    plant(&file, &bytes, 1);
    let places = tree.places();
    let cap = len(&bytes);
    assert_eq!(places.read(&file, cap), Ok(Redacted::new(bytes)));
    // Held with no sharing, the file cannot be opened, so only a size check can refuse it.
    let held = OpenOptions::new().read(true).share_mode(0).open(&file);
    let _held = held.unwrap();
    assert_eq!(places.read(&file, cap - 1), Err(PlaceError::TooLarge));
    assert_eq!(
        places.read(&file, cap),
        Err(PlaceError::Io),
        "it was opened"
    );
}

#[test]
fn a_read_of_a_gone_file_is_missing() {
    let tree = Tree::new("places-gone-file");
    let file = tree.make("OneDrive").join(box_file());
    let places = tree.places();
    assert_eq!(places.read(&file, 1), Err(PlaceError::Missing));
    let gone = tree.top.join("gone").join(box_file());
    assert_eq!(places.read(&gone, 1), Err(PlaceError::Missing));
}

#[test]
fn a_write_replaces_the_box_in_one_step_and_leaves_no_temp_file() {
    let tree = Tree::new("places-write");
    let folder = tree.make("OneDrive");
    let places = tree.places();
    assert_eq!(places.write(&folder, &boxed(b"first")), Ok(()));
    assert_eq!(places.write(&folder, &boxed(b"second")), Ok(()));
    assert_eq!(fs::read(folder.join(box_file())).unwrap(), boxed(b"second"));
    assert_eq!(names(&folder), [box_file()]);
    // An old temp file another run's crash left is swept, and the box is still replaced in one step.
    let other = std::process::id().wrapping_add(1);
    let crashed = folder.join(shipped().temp_name(other, 0));
    plant(&crashed, b"half a box", 1);
    assert_eq!(places.write(&folder, &boxed(b"third")), Ok(()));
    assert_eq!(fs::read(folder.join(box_file())).unwrap(), boxed(b"third"));
    assert_eq!(names(&folder), [box_file()]);
}

#[test]
fn a_failed_write_keeps_the_old_box_whole_and_removes_its_temp_file() {
    let tree = Tree::new("places-failed");
    let folder = tree.make("OneDrive");
    let file = folder.join(box_file());
    plant(&file, &boxed(b"old"), 1);
    // A read-only box refuses the replacing rename, after the temp file is written.
    let before = fs::metadata(&file).unwrap().permissions();
    let mut lock = before.clone();
    lock.set_readonly(true);
    fs::set_permissions(&file, lock).unwrap();
    let wrote = tree.places().write(&folder, &boxed(b"new"));
    assert_eq!(wrote, Err(PlaceError::ReadOnly));
    assert_eq!(fs::read(&file).unwrap(), boxed(b"old"), "whole");
    assert_eq!(names(&folder), [box_file()], "no temp file");
    fs::set_permissions(&file, before).unwrap();
}

#[test]
fn a_full_place_reports_full_and_refuses_a_write_keeping_the_old_box() {
    let tree = Tree::new("places-full");
    let folder = tree.make("usb");
    plant(&folder.join(box_file()), &boxed(b"old"), 1);
    let (places, need) = (tree.places(), len(&boxed(b"new")));
    tree.room.set_free(need - 1);
    assert_eq!(places.state(&folder, need), PlaceState::Full);
    assert_eq!(places.state(&folder, need - 1), PlaceState::Ready);
    assert_eq!(places.write(&folder, &boxed(b"new")), Err(PlaceError::Full));
    assert_eq!(fs::read(folder.join(box_file())).unwrap(), boxed(b"old"));
    tree.room.set_free(need);
    assert_eq!(places.write(&folder, &boxed(b"new")), Ok(()));
}

#[test]
fn a_removed_folder_is_missing_everywhere() {
    let tree = Tree::new("places-removed");
    let folder = tree.make("usb");
    let file = folder.join(box_file());
    plant(&file, &boxed(b"old"), 1);
    let places = tree.places();
    assert_eq!(places.state(&folder, 1), PlaceState::Ready);
    fs::remove_dir_all(&folder).unwrap();
    assert_eq!(places.state(&folder, 1), PlaceState::Missing);
    assert_eq!(places.write(&folder, b"box"), Err(PlaceError::Missing));
    assert_eq!(places.read(&file, 1), Err(PlaceError::Missing));
    assert_eq!(places.folders(Some(&folder)), Err(PlaceError::Missing));
    assert_eq!(places.files(&folder, "kdbx"), Err(PlaceError::Missing));
}

#[test]
fn the_data_folder_its_parents_and_children_are_refused_however_spelt_or_reached() {
    let tree = Tree::new("places-data");
    let data = tree.data();
    let child = data.join("quickfill");
    fs::create_dir(&child).unwrap();
    let linked = tree.top.join("Linked");
    junction(&linked, &child);
    let places = tree.places();
    let refused = [
        data.clone(),
        data.parent().unwrap().to_path_buf(),
        child.clone(),
        data.to_string_lossy().to_uppercase().into(),
        format!("{}\\", data.display()).into(),
        child.join(".."),
        tree.top
            .join("Local")
            .join("never-made")
            .join("..")
            .join(APP_DIR_NAME),
        linked,
    ];
    for folder in &refused {
        assert_eq!(places.write(folder, b"box"), Err(PlaceError::Refused));
    }
    assert!(
        names(&data) == ["quickfill"] && names(&child).is_empty(),
        "nothing written"
    );
    let sibling = tree.make("Local/OneDrive");
    assert_eq!(places.write(&sibling, b"box"), Ok(()), "a sibling");
}

#[test]
fn the_data_folder_its_parents_and_children_have_the_refused_state() {
    let tree = Tree::new("places-data-state");
    let data = tree.data();
    let child = data.join("quickfill");
    fs::create_dir(&child).unwrap();
    let places = tree.places();
    for folder in [&data, &data.parent().unwrap().to_path_buf(), &child] {
        assert_eq!(places.state(folder, 0), PlaceState::Refused, "{folder:?}");
    }
    let sibling = tree.make("Local/OneDrive");
    assert_eq!(places.state(&sibling, 0), PlaceState::Ready, "a sibling");
}
