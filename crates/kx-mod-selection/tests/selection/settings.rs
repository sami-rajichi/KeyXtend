//! The kernel's repair of `[selection]`: a bad value resets alone, with the settings notice (plan edge case 15).

use std::fs;

use kx_mod_selection::{ID, SPEC, SelSettings, sites};
use kx_module_api::ports::Legend;
use kx_module_api::{keys, parse_as};
use kx_settings::{Files, Store};
use kx_test_support::tempdir::TempDir;

/// The `[selection]` section read from a file holding `lines`, and the keys of the notices it raised.
fn load(lines: &str) -> (SelSettings, Vec<&'static str>) {
    let dir = TempDir::new("sel-repair").unwrap();
    let files = Files::new(dir.path());
    fs::write(
        &files.settings,
        format!("[selection]\nversion = 1\n{lines}"),
    )
    .unwrap();
    let (store, report) = Store::load(files, &[(ID, &SPEC)]);
    let notices = report.notices.iter().map(|n| n.key).collect();
    (parse_as(store.get(ID).unwrap()).unwrap(), notices)
}

#[test]
fn a_bad_target_comes_back_empty_with_the_notice() {
    let (s, notices) = load("translate_to = \"fr_FR\"\ncaret_s = 9\n");
    assert_eq!(notices, [keys::SETTINGS_RESET]);
    assert_eq!((s.translate_to.as_str(), s.caret_s), ("", 9));
}

#[test]
fn an_unknown_site_comes_back_as_the_first_with_the_notice() {
    let (s, notices) = load("translate_site = \"bing\"\ntranslate_chars = 2000\n");
    assert_eq!(notices, [keys::SETTINGS_RESET]);
    let first = sites().unwrap().first().unwrap();
    assert_eq!(
        (s.translate_site.as_str(), s.translate_chars),
        (first.id.as_str(), 2000)
    );
}

#[test]
fn a_bad_look_comes_back_as_both_with_the_notice() {
    for bad in ["\"none\"", "false", "\"Words\""] {
        let (s, notices) = load(&format!("look = {bad}\nminimised = false\n"));
        assert_eq!(notices, [keys::SETTINGS_RESET], "{bad}");
        assert_eq!(s.look, Legend::Both, "{bad}");
        assert!(!s.minimised, "the other values stay");
    }
}

#[test]
fn a_good_section_raises_no_notice() {
    let (s, notices) = load("translate_to = \"ar\"\nlook = \"words\"\n");
    assert!(notices.is_empty(), "{notices:?}");
    assert_eq!((s.translate_to.as_str(), s.look), ("ar", Legend::Words));
}
