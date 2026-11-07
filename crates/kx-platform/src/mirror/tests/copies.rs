//! The hidden copies over a fake disk: only mirrors whose app folder holds something, with no link on the way, their
//! app names, and a delete that takes only a listed copy.

use super::*;

/// The file every filled copy holds.
const SETTINGS: &str = "settings.toml";

/// `pkg`'s hidden copy of our data folder.
fn copy_in(pkg: &str) -> PathBuf {
    mirror_of(pkg).join(APP_DIR_NAME)
}

/// The copy the list gives for `pkg`, whose app is `app`.
fn listed(pkg: &str, app: &str) -> HiddenCopy {
    HiddenCopy::new(copy_in(pkg), pkg.into(), app.into())
}

/// A disk with folders at `also`, and a copy holding a settings file in each of `pkgs`.
fn holding(pkgs: &[&str], also: &[PathBuf]) -> Disk {
    let copies = pkgs.iter().map(|p| copy_in(p));
    let disk = Disk::with(&also.iter().cloned().chain(copies).collect::<Vec<_>>());
    for pkg in pkgs {
        disk.put(&copy_in(pkg).join(SETTINGS), Kind::File);
    }
    disk
}

#[test]
fn copies_keep_only_packages_whose_mirror_holds_the_app_folder() {
    let disk = holding(&[PKG], &[data()]);
    disk.put(&mirror_of(OTHER).join("Else"), Kind::Dir);
    assert_eq!(copies(&disk, &at()), Ok(vec![listed(PKG, "Claude")]));
}

#[test]
fn copies_come_a_to_z() {
    let disk = holding(&[OTHER, PKG], &[]);
    let want = vec![listed(PKG, "Claude"), listed(OTHER, "Other")];
    assert_eq!(copies(&disk, &at()), Ok(want));
}

#[test]
fn an_app_folder_that_is_a_file_or_a_link_is_no_copy() {
    let disk = Disk::with(&[]);
    disk.put(&copy_in(PKG), Kind::File);
    disk.put(&copy_in(OTHER), Kind::Link);
    assert_eq!(copies(&disk, &at()), Ok(vec![]));
}

#[test]
fn a_copy_behind_a_link_on_the_way_is_never_listed() {
    let disk = holding(&[PKG], &[]);
    disk.put(&at().root.join(PKG).join("LocalCache"), Kind::Link);
    assert_eq!(copies(&disk, &at()), Ok(vec![]));
}

#[test]
fn an_empty_copy_is_never_listed_nor_removed() {
    let disk = Disk::with(&[copy_in(PKG)]);
    assert_eq!(copies(&disk, &at()), Ok(vec![]));
    let got = remove(&disk, &at(), &listed(PKG, "Claude"));
    assert_eq!(got, Err(HiddenError::Refused));
    assert!(disk.has(&copy_in(PKG)));
}

#[test]
fn a_sibling_of_the_app_folder_is_never_a_copy() {
    let sibling = |pkg: &str| mirror_of(pkg).join(format!("{APP_DIR_NAME}X"));
    let disk = holding(&[PKG], &[sibling(PKG), sibling(OTHER)]);
    for pkg in [PKG, OTHER] {
        disk.put(&sibling(pkg).join(SETTINGS), Kind::File);
    }
    assert_eq!(copies(&disk, &at()), Ok(vec![listed(PKG, "Claude")]));
    let forged = HiddenCopy::new(sibling(OTHER), OTHER.into(), "Other".into());
    assert_eq!(remove(&disk, &at(), &forged), Err(HiddenError::Refused));
    assert!(disk.has(&sibling(OTHER).join(SETTINGS)));
}

#[test]
fn an_app_name_without_its_end_mark_is_the_whole_package_name() {
    let (plain, leading) = ("NoMark", "_Lead");
    let disk = holding(&[plain, leading], &[]);
    let got = copies(&disk, &at()).unwrap();
    let apps: Vec<&str> = got.iter().map(HiddenCopy::app).collect();
    assert!(apps.contains(&plain) && apps.contains(&leading), "{apps:?}");
}

#[test]
fn a_missing_packages_root_lists_nothing_and_a_failed_list_is_unreadable() {
    let disk = Disk::with(&[]);
    disk.forget(&at().root);
    assert_eq!(copies(&disk, &at()), Ok(vec![]));
    let failing = holding(&[PKG], &[]).failing(Op::List);
    assert_eq!(copies(&failing, &at()), Err(HiddenError::Unreadable));
}

#[test]
fn remove_deletes_a_listed_copy_and_all_inside_it() {
    let disk = holding(&[PKG, OTHER], &[copy_in(PKG).join("words")]);
    assert_eq!(remove(&disk, &at(), &listed(PKG, "Claude")), Ok(()));
    assert!(!disk.has(&copy_in(PKG)));
    assert!(disk.has(&mirror_of(PKG)), "the mirror itself stays");
    assert_eq!(
        disk.files(),
        [copy_in(OTHER).join(SETTINGS)],
        "other copies stay"
    );
}

#[test]
fn remove_refuses_a_path_not_listed() {
    let disk = holding(&[PKG], &[data()]).failing(Op::RemoveTree);
    let forged = [
        HiddenCopy::new(data(), PKG.into(), "Claude".into()),
        HiddenCopy::new(mirror_of(PKG), PKG.into(), "Claude".into()),
        listed(OTHER, "Other"),
    ];
    for copy in &forged {
        assert_eq!(remove(&disk, &at(), copy), Err(HiddenError::Refused));
    }
    assert!(disk.has(&data()) && disk.has(&copy_in(PKG)));
}

#[test]
fn a_delete_the_disk_refuses_is_undeleted() {
    let disk = holding(&[PKG], &[]).failing(Op::RemoveTree);
    let got = remove(&disk, &at(), &listed(PKG, "Claude"));
    assert_eq!(got, Err(HiddenError::Undeleted));
}

#[test]
fn a_failed_list_refuses_every_delete() {
    let disk = holding(&[PKG], &[]).failing(Op::List);
    let got = remove(&disk, &at(), &listed(PKG, "Claude"));
    assert_eq!(got, Err(HiddenError::Unreadable));
    assert!(disk.has(&copy_in(PKG)));
}
