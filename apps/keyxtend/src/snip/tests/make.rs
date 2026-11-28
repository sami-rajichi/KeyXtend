//! The Snips folder the boot makes: guarded as the models folder is, its unsaved part inside it taking its rights; a
//! folder already there keeps its rights, a hidden run makes none, and a refusal says why.

use super::{MARK, dirs, marked};
use crate::hidden::Run;
use crate::snip::make;
use kx_platform::snip_dirs;
use kx_test_support::tempdir::TempDir;
use std::io::{self, ErrorKind};
use std::path::Path;

/// A guarded maker the system refuses.
fn refused(_: &Path) -> io::Result<()> {
    Err(ErrorKind::PermissionDenied.into())
}

#[test]
fn the_snips_folder_is_made_guarded_and_its_unsaved_part_inside_it() {
    let user = TempDir::new("snip-make").unwrap();
    let dirs = dirs(&user);
    assert_eq!(make(&Run::Normal, &dirs, marked), None);
    let (snips, unsaved) = snip_dirs(dirs.data_dir().path());
    assert!(snips.join(MARK).is_file(), "made by the guarded maker");
    assert!(unsaved.is_dir(), "the unsaved part");
    assert!(
        !unsaved.join(MARK).exists(),
        "which takes the folder's rights"
    );
    assert_eq!(make(&Run::Normal, &dirs, refused), None, "there already");
}

#[test]
fn a_hidden_run_makes_no_snips_folder_and_a_refusal_says_why() {
    let user = TempDir::new("snip-none").unwrap();
    let dirs = dirs(&user);
    assert_eq!(make(&Run::hidden("app".into()), &dirs, marked), None);
    let (snips, _) = snip_dirs(dirs.data_dir().path());
    assert!(!snips.exists());
    let refusal = make(&Run::Normal, &dirs, refused);
    assert_eq!(refusal, Some(ErrorKind::PermissionDenied));
}

#[cfg(windows)]
#[test]
fn the_windows_kit_makes_the_snips_folder_with_the_models_folders_maker() {
    let user = TempDir::new("snip-guard").unwrap();
    let dirs = dirs(&user);
    assert_eq!(make(&Run::Normal, &dirs, super::super::kit::guarded), None);
    let (snips, unsaved) = snip_dirs(dirs.data_dir().path());
    assert!(snips.is_dir() && unsaved.is_dir());
}
