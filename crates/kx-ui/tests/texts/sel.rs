//! The selection strip's texts (P10 design "The keycaps"): each keycap's word and done word and the strip's reader name
//! in every language, exactly as the owner chose them, and no key in the strip's files that the code does not name.

use crate::files::keys_of;
use crate::{all, lang};
use kx_module_api::ports::SelChip;
use kx_ui::LANGS;
use kx_ui::i18n::keys::{SEL_DONE, SEL_NAME, SEL_WORD, sel_chip_id};
use std::collections::BTreeSet;

/// Every key the strip's code names: each chip's word and done word, then the reader name.
fn keys() -> BTreeSet<String> {
    let words = SelChip::ALL.map(|c| format!("{SEL_WORD}{}", sel_chip_id(c)));
    let done = SelChip::ALL.map(|c| format!("{SEL_DONE}{}", sel_chip_id(c)));
    let all = words.into_iter().chain(done).chain([SEL_NAME.to_owned()]);
    all.collect()
}

/// The strip's texts in `tag`: the words, the done words and the reader name.
fn texts(tag: &str) -> (Vec<String>, Vec<String>, String) {
    let t = lang(tag);
    let of = |prefix: &str| SelChip::ALL.map(|c| t.text(&format!("{prefix}{}", sel_chip_id(c))));
    (of(SEL_WORD).into(), of(SEL_DONE).into(), t.text(SEL_NAME))
}

#[test]
fn every_strip_key_has_its_text_in_every_language() {
    for t in all() {
        let missing: Vec<String> = keys().into_iter().filter(|k| !t.has(k)).collect();
        assert!(missing.is_empty(), "{}: {missing:?}", t.lang().tag);
    }
}

#[test]
fn the_strips_files_hold_only_the_keys_the_code_names() {
    let group = SEL_NAME.split_once('.').map(|(g, _)| g).unwrap_or_default();
    for l in &LANGS {
        let named: BTreeSet<String> = l.files().flat_map(keys_of).collect();
        let ours: BTreeSet<String> = named
            .into_iter()
            .filter(|k| k.split_once('.').is_some_and(|(g, _)| g == group))
            .collect();
        assert_eq!(ours, keys(), "{}", l.tag);
    }
}

#[test]
fn each_language_says_the_owners_words() {
    let en = texts("en");
    assert_eq!(en.0, ["Copy", "Cut", "Paste", "Translate"]);
    assert_eq!(en.1, ["Copied", "Cut", "Pasted", "Opening"]);
    assert_eq!(en.2, "Selection actions");
    let fr = texts("fr");
    assert_eq!(fr.0, ["Copier", "Couper", "Coller", "Traduire"]);
    assert_eq!(fr.1, ["Copié", "Coupé", "Collé", "Ouverture"]);
    assert_eq!(fr.2, "Actions sur la sélection");
    let ar = texts("ar");
    assert_eq!(ar.0, ["نسخ", "قص", "لصق", "ترجمة"]);
    assert_eq!(ar.1, ["تم النسخ", "تم القص", "تم اللصق", "جارٍ الفتح"]);
    assert_eq!(ar.2, "إجراءات التحديد");
}

#[test]
fn the_arabic_words_hold_no_latin_letter_and_no_bidi_control() {
    let (words, done, name) = texts("ar");
    for text in words.iter().chain(&done).chain([&name]) {
        assert!(!text.chars().any(|c| c.is_ascii_alphabetic()), "{text}");
        assert!(
            !text.chars().any(|c| crate::CONTROLS.contains(&c)),
            "{text}"
        );
    }
}
