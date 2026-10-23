//! Every file parses, has the same keys and holds every text the UI names.

use crate::all;
use crate::windows::TITLES;
use fluent_bundle::FluentResource;
use fluent_bundle::concurrent::FluentBundle;
use kx_module_api::keys::ALL_KEYS;
use kx_theme::ModeChoice;
use kx_theme::Themes;
use kx_ui::LANGS;
use kx_ui::i18n::keys::{
    BADGE_FAST, BADGE_HOLDING, BADGE_JOIN, BADGE_SLOW, BANNER_MORE, BAR, COMBO_JOIN, COMBO_THEN,
    DPAD, KEYS_TIP, LANG_KEY, LAYER_PAGE, LIST_COUNT, LIST_JOIN, LIST_MORE, MARK_JOIN, MODE,
    MODULE, OK, PAGE_BACK, PAGE_OF, PAGE_ON, RETRY, SIZE, SIZE_NAME, SIZES, SPACE, THEME,
    TRAY_QUIT, TRAY_SHOW,
};
use std::collections::BTreeSet;

/// Module ids whose names the texts hold today; the app checks the real list (P3 Task 19).
const MODULES: [&str; 7] = [
    "kernel", "keyboard", "layouts", "mouse", "power", "scroll", "ui",
];
/// The language key's designs, `[keyboard] lang_key` values; the app checks the real list too.
const DESIGNS: [&str; 3] = ["carousel", "cycle", "list"];
/// Keys the keyboard view names beside its board ids.
const VIEW_KEYS: [&str; 10] = [
    "key.dpad",
    "key.empty",
    "key.layer-next",
    "tip.later",
    "tip.mouse-off",
    "tip.scroll-off",
    "tip.no-shortcuts",
    "tip.power-off",
    "tip.rclick",
    "tip.grab",
];
/// What starts a Fluent term, a text the files share but code never names.
const TERM: char = '-';

/// Every key in `ftl`: `group.name` for each attribute, and each message with its own text; terms are skipped.
pub fn keys_of(ftl: &str) -> BTreeSet<String> {
    let mut keys = BTreeSet::new();
    let mut group = None;
    for line in ftl.lines().filter(|l| !l.starts_with('#')) {
        let Some((head, value)) = line.split_once(" =") else {
            continue;
        };
        if let Some(attr) = head.trim_start().strip_prefix('.') {
            if let Some(g) = &group {
                keys.insert(format!("{g}.{attr}"));
            }
        } else if !line.starts_with(char::is_whitespace) {
            group = (!head.starts_with(TERM)).then(|| head.to_owned());
            if let Some(g) = group.as_ref().filter(|_| !value.trim().is_empty()) {
                keys.insert(g.clone());
            }
        }
    }
    keys
}

#[test]
fn every_file_parses_without_errors() {
    for l in &LANGS {
        let res = FluentResource::try_new(l.ftl.to_owned());
        let res = res.unwrap_or_else(|(_, e)| panic!("{}: {e:?}", l.tag));
        let mut bundle = FluentBundle::new_concurrent(Vec::new());
        assert!(bundle.add_resource(res).is_ok(), "{}: a key twice", l.tag);
    }
}

#[test]
fn the_three_files_have_the_same_keys() {
    let english = keys_of(LANGS[0].ftl);
    assert!(english.len() > VIEW_KEYS.len());
    for t in all() {
        let keys = keys_of(t.lang().ftl);
        assert_eq!(keys, english, "{}", t.lang().tag);
        assert!(keys.iter().all(|k| t.has(k)), "{}", t.lang().tag);
    }
}

#[test]
fn every_notice_and_view_key_has_its_text() {
    let fixed = [
        TRAY_SHOW,
        TRAY_QUIT,
        RETRY,
        OK,
        LIST_JOIN,
        LIST_MORE,
        LIST_COUNT,
        SPACE,
        BANNER_MORE,
        PAGE_OF,
        PAGE_BACK,
        PAGE_ON,
        MARK_JOIN,
        BADGE_HOLDING,
        BADGE_JOIN,
        BADGE_SLOW,
        BADGE_FAST,
        COMBO_JOIN,
        COMBO_THEN,
        KEYS_TIP,
        LAYER_PAGE,
    ];
    for t in all() {
        let missing: Vec<&str> = ALL_KEYS
            .iter()
            .chain(&fixed)
            .chain(&VIEW_KEYS)
            .copied()
            .filter(|k| !t.has(k))
            .collect();
        assert!(missing.is_empty(), "{}: {missing:?}", t.lang().tag);
    }
}

#[test]
fn every_theme_bar_button_module_and_design_has_its_name() {
    let themes = Themes::shipped().expect("the tokens load");
    let bar = toml::Table::try_from(&themes.icons.bar).expect("the bar as a table");
    let named = themes.list.iter().map(|t| format!("{THEME}.{}", t.look.id));
    let named = named.chain(bar.keys().map(|id| format!("{BAR}.{id}")));
    let named = named.chain(MODULES.iter().map(|id| format!("{MODULE}.{id}")));
    let named: Vec<String> = named
        .chain(DESIGNS.iter().map(|id| format!("{LANG_KEY}.{id}")))
        .collect();
    for t in all() {
        let missing: Vec<&String> = named.iter().filter(|k| !t.has(k)).collect();
        assert!(missing.is_empty(), "{}: {missing:?}", t.lang().tag);
    }
}

#[test]
fn every_size_mode_window_and_dpad_part_has_its_text() {
    let themes = Themes::shipped().expect("the tokens load");
    let dpad = toml::Table::try_from(&themes.icons.dpad).expect("the D-pad as a table");
    let modes = [ModeChoice::Light, ModeChoice::Dark, ModeChoice::Auto];
    let modes = modes.map(|m| serde_json::to_value(m).expect("a mode"));
    let named = SIZES
        .iter()
        .flat_map(|id| [format!("{SIZE}.{id}"), format!("{SIZE_NAME}.{id}")]);
    let named = named.chain(dpad.keys().map(|id| format!("{DPAD}.{id}")));
    let named = named.chain(
        modes
            .iter()
            .map(|m| format!("{MODE}.{}", m.as_str().unwrap_or_default())),
    );
    let fixed = TITLES.iter().map(|k| (*k).to_owned());
    let named: Vec<String> = named.chain(fixed).collect();
    for t in all() {
        let missing: Vec<&String> = named.iter().filter(|k| !t.has(k)).collect();
        assert!(missing.is_empty(), "{}: {missing:?}", t.lang().tag);
    }
}

#[test]
fn each_default_size_preset_has_a_size_letter_and_name() {
    let presets = crate::keyboard_defaults()["presets"]
        .as_array()
        .map(Vec::len)
        .expect("the presets are a list");
    assert_eq!(presets, SIZES.len(), "one size button text per preset");
}
