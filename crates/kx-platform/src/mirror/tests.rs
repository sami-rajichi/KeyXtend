//! The hidden-run rules over a fake disk: a probe found in a mirror, none found, a folder outside the app-data folder,
//! and the probe's clean-up in every case.

use super::*;
use crate::APP_DIR_NAME;

mod copies;
mod disk;

use disk::{Disk, Kind, Op};

/// The probe file's name in every test.
const NAME: &str = "kx-probe-1";
/// The package that hides the run's saves.
const PKG: &str = "Claude_pzs8sxrjxfjjc";
/// A package that hides nothing.
const OTHER: &str = "Other_8wekyb3d8bbwe";

/// The folders every test uses: the local folder, a packages root beside it, and the mirror inside each package.
fn at() -> Mirrors {
    Mirrors {
        local: PathBuf::from("local"),
        root: PathBuf::from("packages"),
        mirror: ["LocalCache", "Local"].iter().collect(),
        app_end: "_".into(),
    }
}

/// The installed data folder.
fn data() -> PathBuf {
    at().local.join(APP_DIR_NAME)
}

/// `pkg`'s private copy of the local folder.
fn mirror_of(pkg: &str) -> PathBuf {
    let at = at();
    at.root.join(pkg).join(at.mirror)
}

/// The probe of the data folder on `disk`.
fn probed(disk: &Disk) -> Result<Probe, HiddenError> {
    probe(disk, &at(), &data(), NAME)
}

#[test]
fn a_probe_found_in_a_package_mirror_is_a_hidden_run() {
    let disk = Disk::with(&[data()]).hiding_in(PKG);
    let want = Probe::Hidden {
        package: PKG.into(),
        app: "Claude".into(),
    };
    assert_eq!(probed(&disk), Ok(want));
    assert!(disk.files().is_empty(), "the probe file is gone");
}

#[test]
fn no_probe_in_any_mirror_is_a_normal_run() {
    let disk = Disk::with(&[data()]);
    assert_eq!(probed(&disk), Ok(Probe::Normal));
    assert!(disk.files().is_empty(), "the probe file is gone");
    assert!(disk.has(&data()), "the data folder stays");
}

#[test]
fn a_data_folder_outside_app_data_is_normal_without_any_disk_work() {
    let disk = Disk::with(&[]).hiding_in(PKG);
    let outside = PathBuf::from("elsewhere").join(APP_DIR_NAME);
    let climbs = at().local.join("..").join(APP_DIR_NAME);
    let sibling = PathBuf::from("localLow").join(APP_DIR_NAME);
    let dirs = [outside, climbs, sibling, at().local, APP_DIR_NAME.into()];
    for dir in dirs {
        assert_eq!(probe(&disk, &at(), &dir, NAME), Ok(Probe::Normal));
    }
    assert_eq!(disk.calls(), 0);
}

#[test]
fn a_data_folder_under_app_data_in_another_case_is_probed() {
    let dir = PathBuf::from("LOCAL").join(APP_DIR_NAME);
    let disk = Disk::with(std::slice::from_ref(&dir));
    assert_eq!(probe(&disk, &at(), &dir, NAME), Ok(Probe::Normal));
    assert!(disk.calls() > 0, "the probe ran");
}

#[test]
fn a_data_folder_the_probe_made_is_removed_again() {
    for disk in [Disk::with(&[]), Disk::with(&[]).hiding_in(PKG)] {
        assert!(probed(&disk).is_ok());
        assert!(!disk.has(&data()), "no data folder");
        assert!(
            !disk.has(&mirror_of(PKG).join(APP_DIR_NAME)),
            "no hidden one"
        );
    }
}

#[test]
fn a_missing_packages_root_is_a_normal_run() {
    let disk = Disk::with(&[data()]);
    disk.forget(&at().root);
    assert_eq!(probed(&disk), Ok(Probe::Normal));
    assert!(disk.files().is_empty());
}

#[test]
fn a_scan_that_fails_is_unreadable_and_still_deletes_the_probe() {
    for made in [false, true] {
        let dirs = if made { vec![] } else { vec![data()] };
        let disk = Disk::with(&dirs).failing(Op::List);
        assert_eq!(probed(&disk), Err(HiddenError::Unreadable));
        assert!(disk.files().is_empty(), "the probe file is gone");
        assert_eq!(
            disk.has(&data()),
            !made,
            "only a folder the probe made goes"
        );
    }
}

#[test]
fn a_probe_that_cannot_be_written_is_unreadable_and_leaves_nothing() {
    let disk = Disk::with(&[]).failing(Op::Write);
    assert_eq!(probed(&disk), Err(HiddenError::Unreadable));
    assert!(!disk.has(&data()), "the folder it made is gone");
}

#[test]
fn a_file_already_named_like_the_probe_is_never_deleted() {
    let disk = Disk::with(&[data()]);
    disk.put(&data().join(NAME), Kind::File);
    assert_eq!(probed(&disk), Err(HiddenError::Unreadable));
    assert_eq!(disk.files(), [data().join(NAME)]);
}

#[test]
fn an_undeleted_probe_is_unreadable_unless_the_run_is_hidden() {
    let normal = Disk::with(&[data()]).failing(Op::RemoveFile);
    assert_eq!(probed(&normal), Err(HiddenError::Unreadable));
    // A hidden run's probe file sits in the hidden copy, which a normal run offers to delete.
    let hidden = Disk::with(&[data()]).hiding_in(PKG).failing(Op::RemoveFile);
    let want = Probe::Hidden {
        package: PKG.into(),
        app: "Claude".into(),
    };
    assert_eq!(probed(&hidden), Ok(want));
}

#[test]
fn a_run_hidden_in_another_package_names_that_package() {
    let disk = Disk::with(&[data()]).hiding_in(OTHER);
    let want = Probe::Hidden {
        package: OTHER.into(),
        app: "Other".into(),
    };
    assert_eq!(probed(&disk), Ok(want));
    assert!(disk.files().is_empty());
}

#[test]
fn a_probe_seen_through_a_link_on_the_way_is_a_normal_run() {
    // A junction from a mirror back to the real folder shows the probe there, yet hides nothing.
    let cache = at().root.join(PKG).join("LocalCache");
    for link in [cache, mirror_of(PKG).join(APP_DIR_NAME)] {
        let disk = Disk::with(&[data()]).hiding_in(PKG);
        disk.put(&link, Kind::Link);
        assert_eq!(probed(&disk), Ok(Probe::Normal));
        assert!(disk.files().is_empty(), "the probe file is gone");
        assert!(disk.has(&link), "the link stays");
    }
}

#[test]
fn a_hidden_run_leaves_no_empty_copy_behind() {
    let disk = Disk::with(&[data()]).hiding_in(PKG);
    assert!(matches!(probed(&disk), Ok(Probe::Hidden { .. })));
    let made = mirror_of(PKG).join(APP_DIR_NAME);
    assert!(
        !disk.has(&made),
        "the folder Windows made for the probe is gone"
    );
    assert!(disk.has(&mirror_of(PKG)) && disk.has(&data()));
}

#[test]
fn a_hidden_run_keeps_a_copy_that_holds_anything() {
    let copy = mirror_of(PKG).join(APP_DIR_NAME);
    let disk = Disk::with(&[data(), copy.clone()]).hiding_in(PKG);
    disk.put(&copy.join("settings.toml"), Kind::File);
    assert!(matches!(probed(&disk), Ok(Probe::Hidden { .. })));
    assert_eq!(disk.files(), [copy.join("settings.toml")]);
}
