//! Tests of `window`.

use super::*;
use kx_test_support::tempdir::TempDir;
use std::io::Write;

/// The folder and file name of the log inside the temp folder.
const LOG_DIR: &str = "logs";
const LOG_FILE: &str = "target.log";

#[test]
fn the_log_makes_its_folder_and_keeps_what_earlier_opens_wrote() {
    let dir = TempDir::new("tw-log").unwrap();
    let file = dir.path().join(LOG_DIR).join(LOG_FILE);
    writeln!(open_log(&file).unwrap(), "first").unwrap();
    writeln!(open_log(&file).unwrap(), "second").unwrap();
    assert_eq!(std::fs::read_to_string(&file).unwrap(), "first\nsecond\n");
}

#[test]
fn a_layout_error_is_its_own_start_error_naming_the_layout() {
    let id = kx_module_api::ports::LayoutId::new(0x0419_0419);
    let err = StartError::from(LayoutError::NotInstalled(id));
    assert!(matches!(
        err,
        StartError::Layout(LayoutError::NotInstalled(_))
    ));
    assert!(err.to_string().contains(&id.to_string()), "{err}");
}

#[test]
fn a_log_that_cannot_open_names_its_file() {
    let dir = TempDir::new("tw-log-bad").unwrap();
    let blocker = dir.path().join(LOG_DIR);
    std::fs::write(&blocker, "a file where the folder should be").unwrap();
    let file = blocker.join(LOG_FILE);
    let err = open_log(&file).unwrap_err();
    assert!(matches!(&err, StartError::Log { file: f, .. } if *f == file));
}
