//! The shortcuts agree with the Windows table, which alone lists them: every id on the default pages has a Windows
//! entry, every id Windows knows has its name in English, French and Arabic, and every key name it sends reads in all
//! three as keyboards print it.

use crate::texts::missing;
use kx_mod_keyboard::view::SHORTCUT_NAME_PREFIX;
use kx_mod_keyboard::{KeyboardSettings, SPEC};
use kx_module_api::ports::{KeyName, ShortcutId, ShortcutInfo, Shortcuts};
use kx_platform_windows::WinShortcuts;
use kx_ui::i18n::keys::{KEYNAME, LABEL};
use kx_ui::{I18n, LANGS};
use std::collections::BTreeSet;

/// Every shortcut the Windows table lists, with its steps.
fn windows() -> Vec<ShortcutInfo> {
    WinShortcuts.known()
}

/// True when language `t` itself reads key name `n`: by its own label or key-name text, or as the name itself.
fn reads(t: &I18n, n: &KeyName) -> bool {
    let own = [LABEL, KEYNAME].map(|g| format!("{g}.{}", n.as_str()));
    let itself = t.key_name(n) == Some(n.as_str().to_ascii_uppercase());
    own.iter().any(|k| t.has(k)) || itself
}

#[test]
fn every_id_on_the_default_pages_has_a_windows_entry() {
    let known: BTreeSet<ShortcutId> = windows().into_iter().map(|s| s.id).collect();
    let pages = SPEC
        .defaults_as::<KeyboardSettings>()
        .unwrap()
        .shortcut_pages;
    let ids: Vec<&ShortcutId> = pages.iter().flatten().collect();
    assert!(!ids.is_empty(), "the owner's pages");
    let lacking: Vec<&str> = ids
        .into_iter()
        .filter(|id| !known.contains(id))
        .map(ShortcutId::as_str)
        .collect();
    assert_eq!(lacking, Vec::<&str>::new());
}

#[test]
fn every_id_windows_knows_has_its_name_in_every_language() {
    let names = windows()
        .into_iter()
        .map(|s| format!("{SHORTCUT_NAME_PREFIX}{}", s.id.as_str()));
    let wanted: BTreeSet<String> = names.collect();
    assert!(!wanted.is_empty(), "the Windows table");
    assert_eq!(missing(&wanted), Vec::new());
}

#[test]
fn every_key_name_windows_sends_reads_in_every_language() {
    let steps = windows().into_iter().flat_map(|s| s.steps);
    let names: BTreeSet<KeyName> = steps
        .flat_map(|s| s.held.into_iter().chain([s.key]))
        .collect();
    assert!(!names.is_empty(), "the Windows table");
    for l in &LANGS {
        let t = I18n::new(l);
        let unread: Vec<&str> = names
            .iter()
            .filter(|n| !reads(&t, n))
            .map(KeyName::as_str)
            .collect();
        assert_eq!(unread, Vec::<&str>::new(), "{}", l.tag);
    }
}
