//! The stage's list: a file off it is refused, whatever it is, a listed one matches in any letter case, and the worker
//! beside the program is listed and has its imports walked.

use super::*;

#[test]
fn a_file_outside_the_list_is_refused_even_when_it_loads() {
    for extra in [
        "extra.exe",
        "qt.conf",
        "platforms/qextra.dll",
        "Qt6Extra.dll",
    ] {
        let loads: &[&str] = if extra == "Qt6Extra.dll" {
            &[extra]
        } else {
            &[]
        };
        let stage = Fake::new(&[(PROGRAM, loads), (extra, &[])]);
        let unplanned = StageError::Unplanned(shown(extra));
        assert_eq!(refusal(stage.check()), Some(unplanned), "{extra}");
    }
}

#[test]
fn listed_files_match_in_any_letter_case() {
    let stage = Fake::new(&[
        (PROGRAM, &["Qt6Core.dll"]),
        ("QT6CORE.DLL", &[]),
        ("QML/QtQuick/QMLDIR", &[]),
    ]);
    assert!(stage.check().is_ok());
}

#[test]
fn the_worker_beside_the_program_is_on_the_list() {
    let stage = Fake::new(&[(PROGRAM, &[]), (WORKER, &["KERNEL32.dll"])]);
    assert!(stage.check().is_ok());
}

#[test]
fn the_workers_imports_must_resolve_too() {
    let stage = Fake::new(&[(PROGRAM, &[]), (WORKER, &["missing.dll"])]);
    let err = StageError::Unresolved {
        by: WORKER.to_string(),
        lib: "missing.dll".to_string(),
    };
    assert_eq!(refusal(stage.check()), Some(err));
}

#[test]
fn the_worker_importing_the_network_library_is_refused() {
    let stage = Fake::new(&[
        (PROGRAM, &["Qt6Qml.dll"]),
        ("Qt6Qml.dll", &["Qt6Network.dll"]),
        ("Qt6Network.dll", &[]),
        (WORKER, &["Qt6Network.dll"]),
    ]);
    let err = StageError::NetworkImport {
        by: WORKER.to_string(),
        lib: "Qt6Network.dll".to_string(),
    };
    assert_eq!(refusal(stage.check()), Some(err));
}
