//! The stage's list: a file off it is refused, whatever it is, and a listed one matches in any letter case.

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
        let unplanned = DevError::Unplanned(shown(extra));
        assert_eq!(stage.check(), Err(unplanned), "{extra}");
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
