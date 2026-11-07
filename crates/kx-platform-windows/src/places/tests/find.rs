//! Finding boxes: names that hold `keyxtend` in any case with the box extension and the mark inside, cloud conflict
//! copies too, in each suggested place's top level and one level down, newest first.

use super::*;
use crate::places::table::Table;
use crate::places::{TABLE, list};
use crate::testlinks::junction;
use kx_module_api::ports::Found;
use std::path::PathBuf;
use tree::{boxed, plant};

/// A tree with `OneDrive` and a USB stick, and the two places.
fn two_places(label: &str) -> (Tree, Place, Place) {
    let mut tree = Tree::new(label);
    let cloud = tree.make("OneDrive");
    tree.set_env("OneDriveConsumer", &cloud);
    let usb = tree.add_drive(PlaceKind::Removable, "usb", "");
    let onedrive = place(PlaceKind::Cloud, "OneDrive", &cloud);
    (tree, onedrive, drive(PlaceKind::Removable, &usb, ""))
}

/// A box found in `place` as `file`, last written `secs` after the Unix epoch.
fn found(place: &Place, file: &Path, secs: i64) -> Found {
    Found {
        place: place.clone(),
        path: Redacted::new(file.to_path_buf()),
        modified_ms: secs * 1000,
    }
}

/// `place`'s folder.
fn at(place: &Place) -> PathBuf {
    place.path.expose().clone()
}

#[test]
fn find_takes_box_names_with_the_mark_and_conflict_copies_one_level_down_newest_first() {
    let (tree, onedrive, usb) = two_places("places-find");
    let ours = at(&onedrive).join(box_file());
    let conflict = at(&onedrive).join("keyxtend backup (1).kxbox");
    let made = at(&onedrive).join("KeyXtend").join(box_file());
    let stick = at(&usb).join("KEYXTEND BACKUP.KXBOX");
    plant(&ours, &boxed(b"old"), 1);
    plant(&conflict, &boxed(b"copy"), 3);
    plant(&made, &boxed(b"made"), 2);
    plant(&stick, &boxed(b"stick"), 4);
    let want = [
        found(&usb, &stick, 4),
        found(&onedrive, &conflict, 3),
        found(&onedrive, &made, 2),
        found(&onedrive, &ours, 1),
    ];
    assert_eq!(tree.places().find(), want);
}

#[test]
fn find_skips_other_names_unmarked_boxes_and_boxes_too_deep_elsewhere_or_linked() {
    let (tree, onedrive, _) = two_places("places-refuse");
    let cloud = at(&onedrive);
    plant(&cloud.join("keyxtend.txt"), &boxed(b"x"), 1);
    plant(&cloud.join("Backup.kxbox"), &boxed(b"x"), 1);
    plant(&cloud.join(box_file()), b"no mark", 1);
    plant(&cloud.join("a").join("b").join(box_file()), &boxed(b"x"), 1);
    plant(&tree.make("Documents").join(box_file()), &boxed(b"x"), 1);
    let behind = tree.make("Elsewhere");
    plant(&behind.join(box_file()), &boxed(b"x"), 1);
    junction(&cloud.join("Linked"), &behind);
    assert_eq!(tree.places().find(), []);
}

#[test]
fn find_looks_in_no_more_subfolders_than_its_own_limit() {
    let (_tree, onedrive, _) = two_places("places-find-cap");
    for sub in ["a", "b", "c"] {
        plant(&at(&onedrive).join(sub).join(box_file()), &boxed(b"x"), 1);
    }
    let mut table = Table::parse(TABLE).unwrap();
    table.max_subfolders = 2;
    // The folder list's limit plays no part in a find.
    table.max_listed = 1;
    assert_eq!(list::boxes(&onedrive, &table).len(), 2);
}

#[test]
fn modified_tells_when_a_file_was_last_written_anywhere_and_none_when_it_is_gone() {
    let tree = Tree::new("places-modified");
    let file = tree.top.join("Anywhere").join("Deep").join(box_file());
    let places = tree.places();
    assert_eq!(places.modified(&file), None, "no such file");
    plant(&file, &boxed(b"box"), 5);
    assert_eq!(places.modified(&file), Some(5000), "where no find reaches");
    fs::remove_file(&file).unwrap();
    assert_eq!(places.modified(&file), None);
}
