//! The store and the ui module agree on the renderer's buffer pool (ADR-0028): a value past its range goes back to
//! keeping none, the reset notice names it, and the section's other values stay.

use kx_module_api::{Notice, NoticeArg, args, keys, parse_as};
use kx_settings::{Files, Store};
use kx_test_support::tempdir::TempDir;
use kx_ui::harden::QT_POOL_BYTES;
use kx_ui::{ID, SPEC, UiSettings};

/// The ui settings and the load's notices, from a settings file whose `[ui]` section holds `lines`.
fn load(lines: &str) -> (UiSettings, Vec<Notice>) {
    let dir = TempDir::new("agree-pool").unwrap();
    let files = Files::new(dir.path());
    std::fs::write(&files.settings, format!("[ui]\n{lines}")).unwrap();
    let (store, report) = Store::load(files, &[(ID, &SPEC)]);
    (parse_as(store.get(ID).unwrap()).unwrap(), report.notices)
}

#[test]
fn a_buffer_pool_past_its_range_goes_back_to_none_and_the_reset_notice_names_it() {
    let (settings, notices) = load(&format!("buffer_pool_bytes = {}\ntip_ms = 700\n", u32::MAX));
    let defaults = UiSettings::from_defaults().unwrap();
    assert_eq!(settings.buffer_pool_bytes, defaults.buffer_pool_bytes);
    assert_eq!(settings.tip_ms, 700, "the other values stay");
    assert_eq!(notices.len(), 1, "{notices:?}");
    assert_eq!(notices[0].key, keys::SETTINGS_RESET);
    let named = notices[0].args.iter().find(|(name, _)| *name == args::KEYS);
    let pool = NoticeArg::List(vec!["buffer_pool_bytes".into()]);
    assert_eq!(named.map(|(_, list)| list), Some(&pool));
}

#[test]
fn qts_own_buffer_pool_is_kept() {
    let (settings, notices) = load(&format!("buffer_pool_bytes = {QT_POOL_BYTES}\n"));
    assert_eq!(settings.buffer_pool_bytes, QT_POOL_BYTES);
    assert!(notices.is_empty(), "{notices:?}");
}
