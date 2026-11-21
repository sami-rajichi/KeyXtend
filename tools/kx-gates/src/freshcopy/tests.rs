//! A fresh copy's pure parts: the start's refusals, the warning check, the folder's owner, and its end, which takes the
//! lent files even while the copy may still run.

use std::cell::Cell;
use std::fs;
use std::path::MAIN_SEPARATOR;

use kx_test_support::tempdir::TempDir;

use super::*;

#[test]
fn a_run_under_ui_access_or_beside_a_running_copy_is_refused() {
    assert_eq!(refuse("G19", Ok(false), false), Ok(()));
    let e = refuse("G19", Ok(true), false).expect_err("uiAccess");
    assert!(e.contains("uiAccess") && e.contains("cargo run"), "{e}");
    let e = refuse("G23", Ok(false), true).expect_err("a copy runs");
    assert!(
        e.contains(&format!("close {NAME}")) && e.contains("G23"),
        "{e}"
    );
    let e = refuse("G19", Err("no token".into()), false).expect_err("unknown");
    assert!(e.contains("no token"), "{e}");
}

#[test]
fn only_a_copy_owing_a_warning_has_its_log_read_and_it_must_hold_it() {
    let unread = || -> Result<String, String> { panic!("the log was read") };
    assert_eq!(heard(None, unread, 0), Ok(()));
    let quiet = heard(Some("gate input"), || Ok(String::new()), 0);
    assert!(quiet.is_err_and(|e| e.contains("no gate warning")));
    let gone = heard(Some("gate input"), || Err("no folder".into()), 0);
    assert_eq!(gone, Err("no folder".into()));
}

#[test]
fn only_a_data_folder_the_gate_made_is_used() {
    let data = Path::new(r"D:\build\data");
    assert_eq!(ours("G23", data, true), Ok(()));
    let e = ours("G19", data, false).expect_err("someone else's folder");
    assert!(
        e.contains(r"D:\build\data") && e.contains("G19") && e.contains("remove"),
        "{e}"
    );
}

#[test]
fn a_lent_file_must_lie_inside_the_copys_data_folder() {
    let data = Path::new(r"D:\build\data");
    assert_eq!(inside(data, &data.join("voice").join("k.sealed")), Ok(()));
    for out in [r"D:\owner\data\voice\k.sealed", r"D:\build\data\..\x"] {
        let e = inside(data, Path::new(out)).expect_err(out);
        assert!(!e.contains("owner") && !e.contains("build"), "{e}");
    }
}

#[test]
fn lent_files_go_one_already_gone_is_fine_and_a_failure_names_no_path() {
    let dir = TempDir::new("kx-gates-lent").expect("a folder");
    let (file, gone) = (dir.path().join("k.sealed"), dir.path().join("none"));
    fs::write(&file, b"stand-in").expect("written");
    assert_eq!(forget(&[file.clone(), gone]), Ok(()));
    assert!(!file.exists(), "removed");
    let e = forget(&[dir.path().to_path_buf()]).expect_err("a folder is no file");
    assert!(!e.contains(&dir.path().display().to_string()), "{e}");
}

#[test]
fn the_end_removes_the_folder_once_the_copy_is_over_else_only_its_lent_files() {
    let (finished, cleared) = (Cell::new(false), Cell::new(false));
    let finish = || -> Result<(), String> {
        finished.set(true);
        Ok(())
    };
    let forget = || -> Result<(), String> {
        cleared.set(true);
        Ok(())
    };
    assert_eq!(end(true, finish, forget), Ok(()));
    assert!(finished.get() && cleared.get(), "both");
    finished.set(false);
    let e = end(false, || panic!("never while it may run"), forget).expect_err("stays");
    assert!(
        e.contains("stays") && e.contains("lent files are gone"),
        "{e}"
    );
    assert!(!e.contains(MAIN_SEPARATOR), "no path: {e}");
    let failed = || -> Result<(), String> { Err("a lent file did not go".into()) };
    let e = end(false, || panic!("never"), failed).expect_err("stays");
    assert!(e.contains("stays") && e.contains("did not go"), "{e}");
    assert!(!finished.get());
}
