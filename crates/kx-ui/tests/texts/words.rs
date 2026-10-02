//! The words each language uses: key names and labels, theme and product names, and its punctuation.

use crate::files::keys_of;
use crate::{all, lang};
use kx_ui::find;
use kx_ui::i18n::keys::{MODULE, THEME};

/// Arabic names of keys that print a Latin word: "مفتاح" and that word (owner, 2026-09-30).
const AR_LATIN_NAMES: [(&str, &str); 8] = [
    ("esc", "مفتاح Esc"),
    ("tab", "مفتاح Tab"),
    ("shift", "مفتاح Shift"),
    ("rshift", "مفتاح Shift الأيمن"),
    ("ctrl", "مفتاح Ctrl"),
    ("alt", "مفتاح Alt"),
    ("altgr", "مفتاح Alt Gr"),
    ("win", "مفتاح Windows"),
];
/// Arabic names that keep Microsoft's established term.
const AR_TERMS: [(&str, &str); 5] = [
    ("bksp", "مسافة للخلف"),
    ("del", "حذف"),
    ("enter", "إدخال"),
    ("caps", "تأمين الأحرف الكبيرة"),
    ("space", "مسافة"),
];
/// "Switch": Arabic keeps it for the mode button, never for a key's name.
const AR_SWITCH: &str = "التبديل";
/// The group of key names.
const KEY_GROUP: &str = "key.";
/// Characters French sets after a no-break space.
const HIGH_PUNCTUATION: [char; 4] = [':', ';', '!', '?'];
/// The spaces allowed before them and inside guillemets: no-break and narrow no-break.
const NO_BREAK: [char; 2] = ['\u{a0}', '\u{202f}'];
/// French guillemets, which hold a no-break space on their inner side.
const GUILLEMETS: [char; 2] = ['«', '»'];
/// Latin punctuation Arabic text writes as "،", "؛" and "؟".
const LATIN_ONLY: [char; 3] = [',', ';', '?'];

/// The lines of `ftl` that hold text, not comments.
fn text_lines(ftl: &str) -> impl Iterator<Item = &str> {
    ftl.lines().filter(|l| !l.trim_start().starts_with('#'))
}

#[test]
fn theme_names_keep_model_names_and_translate_plain_words() {
    let names = |tag: &str| ["native", "et66", "dolch"].map(|id| lang(tag).name_of(THEME, id));
    assert_eq!(names("en"), ["Native Adaptive", "ET66", "Modern Dolch"]);
    assert_eq!(names("fr"), ["Natif adaptatif", "ET66", "Dolch moderne"]);
    assert_eq!(names("ar"), ["الأصلي المتكيّف", "ET66", "دولش الحديث"]);
}

#[test]
fn the_kernel_settings_carry_the_product_name() {
    for t in all() {
        assert_eq!(t.name_of(MODULE, "kernel"), "KeyXtend", "{}", t.lang().tag);
    }
}

#[test]
fn french_keys_print_french_words_and_arabic_keys_the_english_ones() {
    let labels = |tag: &str| {
        ["esc", "del", "caps", "altgr", "bksp", "shift"]
            .map(|k| lang(tag).text(&format!("label.{k}")))
    };
    let fr = [
        "Échap",
        "Suppr",
        "Verr. Maj",
        "Alt Gr",
        "Retour arrière",
        "Maj",
    ];
    assert_eq!(labels("fr"), fr);
    let p1 = ["Esc", "Del", "Caps", "AltGr", "Back", "Shift"];
    assert_eq!(labels("en"), p1, "the P1 test build's words");
    assert_eq!(labels("ar"), p1, "Arabic keyboards print the English words");
    assert_eq!(lang("fr").text("key.bksp"), "Retour arrière");
}

#[test]
fn arabic_names_latin_printed_keys_by_that_word_and_the_others_by_microsoft_terms() {
    let ar = lang("ar");
    for (id, name) in AR_LATIN_NAMES.iter().chain(&AR_TERMS) {
        assert_eq!(ar.text(&format!("{KEY_GROUP}{id}")), *name);
    }
    let names = keys_of(ar.lang().ftl)
        .into_iter()
        .filter(|k| k.starts_with(KEY_GROUP));
    for key in names {
        assert!(!ar.text(&key).contains(AR_SWITCH), "{key}");
    }
}

#[test]
fn french_spaces_high_punctuation_and_guillemets_with_no_break_spaces() {
    let fr = find("fr").expect("French");
    for line in text_lines(fr.ftl) {
        let chars: Vec<char> = line.chars().collect();
        for (i, c) in chars.iter().enumerate() {
            let before = i.checked_sub(1).map(|b| chars[b]);
            let spaced_before = before.is_some_and(|b| NO_BREAK.contains(&b));
            let spaced_after = chars.get(i + 1).is_some_and(|a| NO_BREAK.contains(a));
            let needs_before = HIGH_PUNCTUATION.contains(c) || *c == GUILLEMETS[1];
            assert!(!needs_before || spaced_before, "{line}");
            assert!(*c != GUILLEMETS[0] || spaced_after, "{line}");
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
