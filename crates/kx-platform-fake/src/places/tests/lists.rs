//! The folder list: drives first in the order added, then a folder's subfolders and its files of one extension, A to
//! Z ignoring case; a folder never made is missing.

use super::*;

#[test]
fn drives_lead_the_folder_list_in_the_order_added_whatever_their_kind() {
    let places = FakePlaces::new();
    let drives = [
        (PlaceKind::Drive, "D:", "drive-d"),
        (PlaceKind::Removable, "E:", USB),
        (PlaceKind::Drive, "C:", DRIVE),
    ];
    for (kind, name, path) in drives {
        places.add(kind, name, Path::new(path));
    }
    let want = drives.map(|(kind, name, path)| place(kind, name, path));
    assert_eq!(places.folders(None), Ok(want.to_vec()));
}

#[test]
fn a_folder_never_made_is_missing() {
    let places = places();
    let nowhere = Path::new("Users/someone/nowhere");
    assert_eq!(places.folders(Some(nowhere)), Err(PlaceError::Missing));
    assert_eq!(places.files(nowhere, BOX_EXT), Err(PlaceError::Missing));
    assert_eq!(places.write(nowhere, OLD), Err(PlaceError::Missing));
    assert_eq!(places.state(nowhere, 0), PlaceState::Missing);
}

#[test]
fn folders_and_files_list_a_to_z_ignoring_case() {
    let places = FakePlaces::new();
    let drive = Path::new(DRIVE);
    places.add(PlaceKind::Drive, "C:", drive);
    for name in ["b", "A", "c/d"] {
        places.add_folder(&drive.join(name));
    }
    let folder = |name| place(PlaceKind::Folder, name, drive.join(name));
    let want = vec![folder("A"), folder("b"), folder("c")];
    assert_eq!(places.folders(Some(drive)), Ok(want));
    for name in ["x.kdbx", "B.KDBX", "a.txt", "kdbx"] {
        places.plant(&drive.join(name), Vec::new(), 0);
    }
    let file = |name| place(PlaceKind::File, name, drive.join(name));
    let want = vec![file("B.KDBX"), file("x.kdbx")];
    assert_eq!(places.files(drive, "kdbx"), Ok(want.clone()));
    assert_eq!(places.files(drive, "KDBX"), Ok(want), "any case");
}
