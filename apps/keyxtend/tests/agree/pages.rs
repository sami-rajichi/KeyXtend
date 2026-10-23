//! The store and the keyboard agree on the shortcut pages: a bad list goes back to its default alone, and the reset
//! notice names it.

use crate::sizes::load;
use kx_mod_keyboard::{KeyboardSettings, SPEC};
use kx_module_api::{NoticeArg, args, keys};

#[test]
fn bad_shortcut_pages_go_back_to_their_default_alone_and_the_reset_notice_names_them() {
    let defaults: KeyboardSettings = SPEC.defaults_as().unwrap();
    for bad in ["[[\"copy\", \"copy\"]]", "[[\"Copy\"]]", "[[\"copy\"], []]"] {
        let (settings, notices) = load(&format!("shortcut_pages = {bad}\nrepeat = false\n"));
        assert_eq!(settings.shortcut_pages, defaults.shortcut_pages, "{bad}");
        assert!(!settings.repeat, "the other values stay");
        assert_eq!(notices.len(), 1, "{notices:?}");
        assert_eq!(notices[0].key, keys::SETTINGS_RESET);
        let named = notices[0].args.iter().find(|(name, _)| *name == args::KEYS);
        let pages = NoticeArg::List(vec!["shortcut_pages".into()]);
        assert_eq!(named.map(|(_, list)| list), Some(&pages), "{bad}");
    }
}
