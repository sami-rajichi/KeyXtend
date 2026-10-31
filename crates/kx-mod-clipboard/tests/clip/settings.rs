//! The kernel's repair of `[clipboard]`: a value out of its range resets alone, with the notice (edge 2).

use std::fs;

use kx_mod_clipboard::{ClipSettings, ID, SPEC};
use kx_module_api::{keys, parse_as};
use kx_settings::{Files, Store};
use kx_test_support::tempdir::TempDir;

#[test]
fn a_max_entries_out_of_range_comes_back_as_its_default_with_the_notice() {
    let dir = TempDir::new("clip-repair").unwrap();
    let files = Files::new(dir.path());
    let text = "[clipboard]\nversion = 1\nmax_entries = 10000\nkeep_days = 3\n";
    fs::write(&files.settings, text).unwrap();
    let (store, report) = Store::load(files, &[(ID, &SPEC)]);
    let reset: Vec<_> = report.notices.iter().map(|n| n.key).collect();
    assert_eq!(reset, [keys::SETTINGS_RESET]);
    let settings: ClipSettings = parse_as(store.get(ID).unwrap()).unwrap();
    let defaults = ClipSettings::from_defaults().unwrap();
    assert_eq!(
        (settings.max_entries, settings.keep_days),
        (defaults.max_entries, 3)
    );
}
