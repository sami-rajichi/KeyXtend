//! Every file parses, has the same keys, holds every text the UI names, and keeps its language's punctuation.

use crate::{all, lang};
use fluent_bundle::FluentResource;
use fluent_bundle::concurrent::FluentBundle;
use kx_module_api::keys::ALL_KEYS;
use kx_theme::Themes;
use kx_ui::i18n::keys::{
    BAR, LANG_KEY, LIST_JOIN, LIST_MORE, MODULE, OK, RETRY, SPACE, THEME, TRAY_QUIT, TRAY_SHOW,
};
use kx_ui::{LANGS, find};
use std::collections::BTreeSet;

/// Module ids whose names the texts hold today; the app checks the real list (P3 Task 19).
const MODULES: [&str; 4] = ["kernel", "keyboard", "layouts", "ui"];
/// The language key's designs, `[keyboard] lang_key` values; the app checks the real list too.
const DESIGNS: [&str; 3] = ["carousel", "cycle", "list"];
/// Keys the keyboard view names beside its board ids.
const VIEW_KEYS: [&str; 2] = ["key.dpad", "tip.later"];
/// Characters French sets after a no-break space.
const HIGH_PUNCTUATION: [char; 4] = [':', ';', '!', '?'];
/// The spaces allowed before them: no-break and narrow no-break.
const NO_BREAK: [char; 2] = ['\u{a0}', '\u{202f}'];
/// Latin punctuation Arabic text writes as "،", "؛" and "؟".
const LATIN_ONLY: [char; 3] = [',', ';', '?'];

/// Every key in `ftl`: `group.name` for each attribute, and each message with its own text.
fn keys_of(ftl: &str) -> BTreeSet<String> {
    let mut keys = BTreeSet::new();
    let mut group = String::new();
    for line in ftl.lines().filter(|l| !l.starts_with('#')) {
        let Some((head, value)) = line.split_once(" =") else {
            continue;
        };
        if let Some(attr) = head.trim_start().strip_prefix('.') {
            keys.insert(format!("{group}.{attr}"));
        } else if !line.starts_with(char::is_whitespace) {
            head.clone_into(&mut group);
            if !value.trim().is_empty() {
                keys.insert(group.clone());
            }
        }
    }
    keys
}

/// The lines of `ftl` that hold text, not comments.
fn text_lines(ftl: &str) -> impl Iterator<Item = &str> {
    ftl.lines().filter(|l| !l.trim_start().starts_with('#'))
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
    let fixed = [TRAY_SHOW, TRAY_QUIT, RETRY, OK, LIST_JOIN, LIST_MORE, SPACE];
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
fn theme_names_keep_model_names_and_translate_plain_words() {
    let names = |tag: &str| ["native", "et66", "dolch"].map(|id| lang(tag).name_of(THEME, id));
    assert_eq!(names("en"), ["Native Adaptive", "ET66", "Modern Dolch"]);
    assert_eq!(names("fr"), ["Natif adaptatif", "ET66", "Dolch moderne"]);
    assert_eq!(names("ar"), ["الأصلي المتكيّف", "ET66", "دولش الحديث"]);
}

#[test]
fn french_keys_print_french_words_and_arabic_keys_the_english_ones() {
    let labels =
        |tag: &str| ["esc", "del", "caps", "shift"].map(|k| lang(tag).text(&format!("label.{k}")));
    assert_eq!(labels("fr"), ["Échap", "Suppr", "Verr. Maj", "Maj"]);
    assert_eq!(labels("ar"), ["Esc", "Del", "Caps Lock", "Shift"]);
    assert_eq!(lang("ar").text("key.enter"), "إدخال");
    assert_eq!(lang("fr").text("key.bksp"), "Retour arrière");
}

#[test]
fn french_puts_a_no_break_space_before_high_punctuation() {
    let fr = find("fr").expect("French");
    for line in text_lines(fr.ftl) {
        let chars: Vec<char> = line.chars().collect();
        for (i, c) in chars.iter().enumerate().skip(1) {
            let spaced = NO_BREAK.contains(&chars[i - 1]);
            assert!(!HIGH_PUNCTUATION.contains(c) || spaced, "{line}");
        }
    }
}

#[test]
fn arabic_uses_arabic_punctuation() {
    let ar = find("ar").expect("Arabic");
    for line in text_lines(ar.ftl) {
        assert!(!line.contains(LATIN_ONLY), "{line}");
    }
}
