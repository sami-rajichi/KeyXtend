//! The store and the keyboard agree on the hard size bounds: a size past one goes back to its default, and the
//! reset notice names it.

use kx_mod_keyboard::{ID, KeyboardSettings, SPEC};
use kx_module_api::{Notice, NoticeArg, args, keys, parse_as};
use kx_settings::{Files, Store};
use kx_test_support::tempdir::TempDir;

/// The keyboard's settings and the load's notices, from a settings file whose `[keyboard]` section holds `lines`.
pub(crate) fn load(lines: &str) -> (KeyboardSettings, Vec<Notice>) {
    let dir = TempDir::new("agree-sizes").unwrap();
    let files = Files::new(dir.path());
    std::fs::write(&files.settings, format!("[keyboard]\n{lines}")).unwrap();
    let (store, report) = Store::load(files, &[(ID, &SPEC)]);
    (parse_as(store.get(ID).unwrap()).unwrap(), report.notices)
}

#[test]
fn a_size_past_a_hard_bound_goes_back_to_its_default_and_the_reset_notice_names_it() {
    let (settings, notices) = load("size_min = 0.1\nsize_max = 1000.0\n");
    let defaults: KeyboardSettings = SPEC.defaults_as().unwrap();
    let limits = |s: &KeyboardSettings| (s.size_min, s.size_max);
    assert_eq!(limits(&settings), limits(&defaults));
    assert_eq!(notices.len(), 1, "{notices:?}");
    assert_eq!(notices[0].key, keys::SETTINGS_RESET);
    let named = notices[0].args.iter().find(|(name, _)| *name == args::KEYS);
    let both = NoticeArg::List(vec!["size_max".into(), "size_min".into()]);
    assert_eq!(named.map(|(_, list)| list), Some(&both));
}
