//! Tests of `close`.

use std::path::PathBuf;

use kx_platform_fake::FakeInstance;
use kx_settings::Files;
use kx_test_support::tempdir::TempDir;

use super::*;
use crate::ownset::{self, Kept};

/// The program a running copy reports.
const EXE: &str = r"C:\x\keyxtend.exe";
/// The owner's settings file.
const OWNER: &str = "[kernel]\nlog_level = \"debug\"\n";

#[test]
fn quit_is_posted_to_the_running_copy_and_starts_nothing() {
    let session = FakeInstance::new();
    let mut claim = session.claim().expect("the first copy runs");
    claim.listen(Box::new(|_| {})).expect("it listens");
    assert_eq!(ask_quit(&session.clone()), Ok(()));
    assert_eq!(session.sent(), [Command::Quit]);
}

#[test]
fn a_copy_just_started_gets_quit_once_it_listens() {
    let session = FakeInstance::new();
    let late = std::cell::RefCell::new(Vec::new());
    let waits = std::cell::Cell::new(0);
    let wait = || {
        waits.set(waits.get() + 1);
        if waits.get() == 2 {
            let mut claim = session.claim().expect("the started copy runs");
            claim.listen(Box::new(|_| {})).expect("it listens");
            late.borrow_mut().push(claim);
        }
    };
    assert_eq!(
        ask_quit_tries(&session.clone(), 5, || false, wait),
        Ok(Asked::Sent(3))
    );
    assert_eq!(session.sent(), [Command::Quit]);
    let never = ask_quit_tries(&FakeInstance::new(), 3, || false, || {});
    assert!(
        never.is_err_and(|e| e.contains("command window")),
        "it never listened"
    );
}

#[test]
fn a_copy_that_ended_by_itself_gets_no_more_tries() {
    let session = FakeInstance::new();
    let gone = std::cell::Cell::new(false);
    let tries = ask_quit_tries(&session.clone(), 5, || gone.get(), || gone.set(true));
    assert_eq!(tries, Ok(Asked::Gone(1)), "one try, then it was gone");
    assert!(session.sent().is_empty());
}

#[test]
fn a_copy_that_cannot_hear_quit_says_why() {
    let none = ask_quit(&FakeInstance::new()).expect_err("no copy listens");
    assert!(
        none.why.contains("command window") && !none.refused,
        "{none:?}"
    );
    let refused = Unasked::from(UiError::Refused(5));
    assert!(refused.refused, "{refused:?}");
    let why = refused.why;
    assert!(why.contains("code 5") && why.contains("uiAccess"), "{why}");
}

#[test]
fn the_result_names_the_product_as_its_keyboard() {
    let quit = (EXE.to_string(), true);
    let v = result(Some(&quit), &Ok(false));
    assert_eq!(v["keyboard"], product::NAME);
    assert_eq!(
        (v["gate"].as_str(), v["pass"].as_bool()),
        (Some("close"), Some(true))
    );
}

#[test]
fn close_passes_only_when_the_copy_ended_and_any_backup_came_back() {
    let back = result(None, &Ok(true));
    assert_eq!(back["settings_back"].as_bool(), Some(true));
    assert_eq!(back["pass"].as_bool(), Some(true), "no copy ran");
    let hung = (EXE.to_string(), false);
    assert_eq!(
        result(Some(&hung), &Ok(false))["pass"].as_bool(),
        Some(false)
    );
    let refused = result(None, &Err("the backup stays".into()));
    assert_eq!(refused["pass"].as_bool(), Some(false));
    assert_eq!(refused["settings_error"].as_str(), Some("the backup stays"));
}

/// A data folder holding the gate's file and a sidecar a stopped gate left with the owner's.
fn left() -> (TempDir, Files, PathBuf) {
    let dir = TempDir::new("gates-close").expect("a temp folder");
    let (files, side) = (Files::new(dir.path()), dir.path().join("side.json"));
    std::fs::write(&files.settings, OWNER).expect("written");
    let (kept, _) = Kept::take(&files, &side).expect("kept");
    kept.write(&toml::Table::new()).expect("the gate's file");
    std::mem::forget(kept);
    (dir, files, side)
}

/// The settings file's text.
fn text(files: &Files) -> String {
    std::fs::read_to_string(&files.settings).expect("reads")
}

#[test]
fn a_quit_that_did_not_end_keeps_the_backup_and_says_so() {
    let (_dir, files, side) = left();
    let back = |ended| ownset::recover(&files, &side, ended, || false);
    let hung = || Ok(Some((EXE.to_string(), false)));
    let v = closed(hung, back, Ok(side.clone())).expect("a result");
    let e = v["settings_error"].as_str().unwrap_or_default();
    assert!(e.contains(&side.display().to_string()), "{v}");
    assert_eq!(v["pass"].as_bool(), Some(false));
    assert!(side.exists(), "a copy still exiting may yet save");
    assert_ne!(text(&files), OWNER, "nothing restored under it");
    let quit = || Ok(Some((EXE.to_string(), true)));
    let v = closed(quit, back, Ok(side.clone())).expect("a result");
    assert_eq!(v["settings_back"].as_bool(), Some(true), "{v}");
    assert_eq!((text(&files), side.exists()), (OWNER.to_string(), false));
}

#[test]
fn with_no_copy_and_no_backup_it_names_the_place_it_checked() {
    let dir = TempDir::new("gates-close").expect("a temp folder");
    let (files, side) = (Files::new(dir.path()), dir.path().join("side.json"));
    let back = |ended| ownset::recover(&files, &side, ended, || false);
    let e = closed(|| Ok(None), back, Ok(side.clone())).expect_err("nothing to do");
    assert!(e.contains(&side.display().to_string()), "{e}");
    let refused = || Err(Unasked::from(UiError::Refused(5)));
    let e = closed(refused, |_| panic!("no restore"), Ok(side.clone())).expect_err("refused");
    assert!(e.contains("uiAccess"), "{e}");
}

#[test]
fn a_refused_backup_name_still_asks_the_copy_to_quit_first_then_says_why() {
    let asked = std::cell::Cell::new(false);
    let quit = || {
        asked.set(true);
        Ok(Some((EXE.to_string(), true)))
    };
    let bad = Err("the backup is named as the results file".to_string());
    let v = closed(quit, |_| panic!("no restore"), bad).expect("a result");
    assert!(asked.get(), "the copy was asked to quit");
    assert_eq!(v["program"].as_str(), Some(EXE));
    let e = v["settings_error"].as_str().unwrap_or_default();
    assert!(e.contains("results file"), "{v}");
    assert_eq!(v["pass"].as_bool(), Some(false));
}
