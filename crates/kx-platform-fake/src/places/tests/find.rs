//! Finding boxes: names that hold `keyxtend` in any case with the box extension and the mark inside, in each
//! suggested place's top level and one level down, newest first.

use super::*;

#[test]
fn find_takes_box_names_with_the_mark_one_level_down_newest_first() {
    let places = places();
    let cloud = Path::new(ONEDRIVE);
    let ours = box_in(ONEDRIVE);
    let copy = cloud.join("KeyXtend").join("my KEYXTEND copy.kxbox");
    let stick = Path::new(USB).join("keyxtend backup (1).kxbox");
    places.plant(&ours, boxed(OLD), 1);
    places.plant(&copy, boxed(NEW), 3);
    places.plant(&stick, boxed(OLD), 2);
    let found = |place, path: &PathBuf, modified_ms| Found {
        place,
        path: Redacted::new(path.clone()),
        modified_ms,
    };
    let want = [
        found(onedrive(), &copy, 3),
        found(usb(), &stick, 2),
        found(onedrive(), &ours, 1),
    ];
    assert_eq!(places.find(), want);
}

#[test]
fn find_refuses_other_names_a_box_without_the_mark_and_boxes_too_deep_or_elsewhere() {
    let places = places();
    let cloud = Path::new(ONEDRIVE);
    places.plant(&cloud.join("keyxtend.txt"), boxed(OLD), 1);
    places.plant(&cloud.join("Backup.kxbox"), boxed(OLD), 1);
    places.plant(&box_in(ONEDRIVE), b"no mark".to_vec(), 1);
    places.plant(&cloud.join("a/b").join(box_file()), boxed(OLD), 1);
    places.plant(&box_in(PLAIN), boxed(OLD), 1);
    assert!(places.find().is_empty(), "{:?}", places.find());
}

#[test]
fn find_skips_a_vanished_subfolder_and_a_file_deleted_by_hand() {
    let places = places();
    let sub = Path::new(ONEDRIVE).join("KeyXtend");
    places.plant(&sub.join(box_file()), boxed(OLD), 1);
    places.plant(&box_in(USB), boxed(OLD), 2);
    places.vanish(&sub);
    places.delete_by_hand(&box_in(USB));
    assert!(places.find().is_empty());
    places.come_back(&sub);
    assert_eq!(places.find().len(), 1);
}

#[test]
fn modified_tells_when_a_file_was_last_written_and_none_once_it_is_gone() {
    let places = places();
    let (file, stick) = (box_in(USB), Path::new(USB));
    assert_eq!(places.modified(&file), None, "no such file");
    places.plant(&file, boxed(OLD), 7);
    assert_eq!(places.modified(&file), Some(7));
    places.set_now(9);
    places.write(stick, &boxed(NEW)).unwrap();
    assert_eq!(places.modified(&file), Some(9), "a write stamps it");
    places.vanish(stick);
    assert_eq!(places.modified(&file), None, "its place is gone");
    places.come_back(stick);
    places.fail(PlaceCall::Modified, PlaceError::Io);
    assert_eq!(places.modified(&file), None, "the system did not say");
    assert_eq!(places.modified(&file), Some(9), "only that once");
}
