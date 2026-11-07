//! The suggested places: the table's cloud folders that exist, in its order and each once, then the removable drives;
//! and the drives that lead the folder list.

use super::*;

#[test]
fn only_existing_cloud_folders_are_offered_in_table_order_each_once_then_removable_drives() {
    let mut tree = Tree::new("places-suggest");
    let profile = tree.make("profile");
    let personal = tree.make("profile/OneDrive");
    let work = tree.make("profile/OneDrive - Contoso");
    let dropbox = tree.make("profile/Dropbox");
    tree.set_env("OneDriveConsumer", &personal);
    tree.set_env("OneDrive", &personal);
    tree.set_env("OneDriveCommercial", &work);
    tree.set_env("USERPROFILE", &profile);
    let google = tree.add_drive(PlaceKind::Drive, "gdrive", "Google Drive");
    let mine = google.join("My Drive");
    fs::create_dir(&mine).unwrap();
    let usb = tree.add_drive(PlaceKind::Removable, "usb", "STICK");
    tree.add_drive(PlaceKind::Drive, "fixed", "");
    let want = [
        place(PlaceKind::Cloud, "OneDrive", &personal),
        place(PlaceKind::Cloud, "OneDrive - Contoso", &work),
        place(PlaceKind::Cloud, "Google Drive", &mine),
        place(PlaceKind::Cloud, "Dropbox", &dropbox),
        drive(PlaceKind::Removable, &usb, "STICK"),
    ];
    assert_eq!(tree.places().suggested(), want);
}

#[test]
fn a_relative_or_missing_folder_and_a_gone_drive_are_not_offered() {
    let mut tree = Tree::new("places-unoffered");
    // The crate's own `src` folder, which a relative path finds from the test's working folder.
    tree.set_env("OneDriveConsumer", Path::new("src"));
    tree.set_env("OneDrive", &tree.top.join("never-made"));
    let usb = tree.add_drive(PlaceKind::Removable, "usb", "STICK");
    fs::remove_dir(&usb).unwrap();
    let places = tree.places();
    assert_eq!(places.suggested(), []);
    assert_eq!(places.folders(None), Ok(Vec::new()), "no drive left");
}

#[test]
fn the_drives_lead_the_folder_list_in_windows_order_whatever_their_kind() {
    let mut tree = Tree::new("places-drives");
    let fixed = tree.add_drive(PlaceKind::Drive, "c", "Windows");
    let usb = tree.add_drive(PlaceKind::Removable, "e", "");
    let want = vec![
        drive(PlaceKind::Drive, &fixed, "Windows"),
        drive(PlaceKind::Removable, &usb, ""),
    ];
    assert_eq!(tree.places().folders(None), Ok(want));
}

#[test]
fn a_network_drive_leads_the_folder_list_but_is_never_suggested_searched_or_a_cloud_base() {
    let mut tree = Tree::new("places-share");
    let share = tree.add_share("share", "Google Drive");
    let mine = share.join("My Drive");
    fs::create_dir(&mine).unwrap();
    fs::write(mine.join(box_file()), tree::boxed(b"box")).unwrap();
    let places = tree.places();
    let listed = vec![drive(PlaceKind::Drive, &share, "Google Drive")];
    assert_eq!(places.folders(None), Ok(listed));
    assert_eq!(places.suggested(), []);
    assert_eq!(places.find(), []);
}
