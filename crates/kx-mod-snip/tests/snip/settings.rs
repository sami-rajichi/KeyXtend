//! The kernel's repair of `[snip]`: a bad value resets alone to its default, with the settings notice.

use std::fs;

use kx_mod_snip::{ID, SPEC, SnipSettings};
use kx_module_api::ports::{Rect, SnipKeyboard};
use kx_module_api::{Notice, NoticeArg, args, keys, parse_as};
use kx_settings::{Files, Store};
use kx_test_support::tempdir::TempDir;

/// The `[snip]` section read from a file holding `lines`, and the notices it raised.
fn read(lines: &str) -> (SnipSettings, Vec<Notice>) {
    let dir = TempDir::new("snip-repair").unwrap();
    let files = Files::new(dir.path());
    fs::write(&files.settings, format!("[snip]\nversion = 1\n{lines}")).unwrap();
    let (store, report) = Store::load(files, &[(ID, &SPEC)]);
    (parse_as(store.get(ID).unwrap()).unwrap(), report.notices)
}

/// As `read`, with each notice's key alone.
fn load(lines: &str) -> (SnipSettings, Vec<&'static str>) {
    let (s, notices) = read(lines);
    (s, notices.iter().map(|n| n.key).collect())
}

/// The defaults.
fn defaults() -> SnipSettings {
    SnipSettings::from_defaults().unwrap()
}

/// An editor's place as a file from before `editor_full` holds it.
const PLACED: &str = "editor = { x = 10, y = 20, w = 800, h = 500 }\n";

/// That place.
const AT: Rect = Rect {
    x: 10,
    y: 20,
    w: 800,
    h: 500,
};

#[test]
fn a_bad_frame_rate_comes_back_as_its_default_alone_with_the_notice() {
    let (s, notices) = load("fps = 20\ncountdown_s = 5\n");
    assert_eq!(notices, [keys::SETTINGS_RESET]);
    assert_eq!((s.fps, s.countdown_s), (defaults().fps, 5));
}

#[test]
fn an_unknown_keyboard_way_comes_back_as_its_default_alone_with_the_notice() {
    let (s, notices) = load("keyboard = \"off\"\nmic_on = true\n");
    assert_eq!(notices, [keys::SETTINGS_RESET]);
    assert_eq!(s.keyboard, defaults().keyboard);
    assert!(s.mic_on, "the other values stay");
}

#[test]
fn a_broken_editor_place_comes_back_empty_alone_with_the_notice() {
    let (s, notices) = load("editor = { x = 1, y = 2 }\nundo_max = 100\n");
    assert_eq!(notices, [keys::SETTINGS_RESET]);
    assert_eq!((s.editor, s.undo_max), (None, 100));
}

#[test]
fn a_good_section_raises_no_notice() {
    let (s, notices) = load(&format!("keyboard = \"shown\"\nfps = 15\n{PLACED}"));
    assert!(notices.is_empty(), "{notices:?}");
    assert_eq!(
        (s.keyboard, s.fps, s.editor),
        (SnipKeyboard::Shown, 15, Some(AT))
    );
}

#[test]
fn a_file_from_before_editor_full_opens_the_editor_unmaximised() {
    let (s, notices) = load(PLACED);
    assert!(notices.is_empty(), "{notices:?}");
    assert_eq!((s.editor, s.editor_full), (Some(AT), None));
}

#[test]
fn a_work_area_with_no_box_under_it_comes_back_empty_alone_with_the_notice() {
    let (s, notices) = read("editor_full = { x = 0, y = 0, w = 1920, h = 1040 }\nundo_max = 100\n");
    let named = (args::KEYS, NoticeArg::List(vec!["editor_full".into()]));
    let told: Vec<_> = notices.iter().map(|n| n.key).collect();
    assert_eq!(told, [keys::SETTINGS_RESET]);
    assert!(notices[0].args.contains(&named), "{:?}", notices[0].args);
    assert_eq!((s.editor, s.editor_full, s.undo_max), (None, None, 100));
}

#[test]
fn a_broken_box_takes_its_good_work_area_with_it_and_the_rest_stays() {
    let full = "editor_full = { x = 0, y = 0, w = 1920, h = 1040 }\n";
    let (s, notices) = load(&format!("editor = {{ x = 1 }}\n{full}undo_max = 100\n"));
    assert_eq!(notices, [keys::SETTINGS_RESET]);
    assert_eq!((s.editor, s.editor_full, s.undo_max), (None, None, 100));
}

#[test]
fn a_frame_side_out_of_range_comes_back_as_its_default_alone_with_the_notice() {
    let (s, notices) = load("frame_min_px = 0\nundo_max = 100\n");
    assert_eq!(notices, [keys::SETTINGS_RESET]);
    assert_eq!((s.frame_min_px, s.undo_max), (defaults().frame_min_px, 100));
}

#[test]
fn a_broken_work_area_comes_back_empty_alone_and_the_box_stays() {
    let (s, notices) = load(&format!("{PLACED}editor_full = {{ x = 0 }}\n"));
    assert_eq!(notices, [keys::SETTINGS_RESET]);
    assert_eq!((s.editor, s.editor_full), (Some(AT), None));
}
