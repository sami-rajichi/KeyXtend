//! Quick-fill's Arabic: every text is in Arabic script with no Latin word but a name, its digits are Western, and its
//! counts take their CLDR forms (ADR-0017).

use super::{PLACE, filled};
use crate::lang;
use kx_module_api::is_arabic;
use kx_module_api::ports::PasswordAsk;
use kx_ui::i18n::keys::{FILL_ASK, FILL_JOIN, FILL_MASK, FILL_VALUES, ask_id};

/// Texts that are marks, not words: the mask and what joins a hint's values.
const MARKS: [&str; 2] = [FILL_MASK, FILL_JOIN];
/// Latin names an Arabic text may hold: the product, Wi-Fi, `KeePass` and a place's short name.
const NAMES: [&str; 4] = ["KeyXtend", "Wi-Fi", "KeePass", PLACE];
/// What Fluent writes for a variable a text was not given.
const UNFILLED: char = '{';
/// Arabic-Indic digits, which no text writes (ADR-0017).
const INDIC: std::ops::RangeInclusive<char> = '\u{660}'..='\u{669}';

#[test]
fn every_arabic_text_is_arabic_with_no_latin_word_but_a_name() {
    let ar = lang("ar");
    for (key, text) in filled(&ar) {
        if MARKS.contains(&key.as_str()) || text.contains(UNFILLED) {
            continue;
        }
        assert!(is_arabic(&text), "{key}: {text}");
        let left = NAMES.iter().fold(text.clone(), |t, n| t.replace(n, ""));
        assert!(
            !left.chars().any(|c| c.is_ascii_alphabetic()),
            "{key}: {text}"
        );
        assert!(!text.chars().any(|c| INDIC.contains(&c)), "{key}: {text}");
    }
}

#[test]
fn arabic_counts_fields_values_and_tries_in_their_cldr_forms() {
    let ar = lang("ar");
    let counts = [0, 1, 2, 3, 11, 100];
    let fields: Vec<String> = counts.map(|n| ar.fill_fields("شخص", n)).into();
    let want = [
        "شخص · بلا حقول",
        "شخص · حقل واحد",
        "شخص · حقلان",
        "شخص · 3 حقول",
        "شخص · 11 حقلًا",
        "شخص · 100 حقل",
    ];
    assert_eq!(fields, want);
    let values: Vec<String> = counts.map(|n| ar.counted(FILL_VALUES, n)).into();
    let want = [
        "بلا قيم",
        "قيمة واحدة",
        "قيمتان",
        "3 قيم",
        "11 قيمة",
        "100 قيمة",
    ];
    assert_eq!(values, want);
    let check = format!("{FILL_ASK}{}", ask_id(PasswordAsk::Check { tries: 0 }));
    let tries: Vec<String> = [1, 2, 5].map(|n| ar.counted(&check, n)).into();
    let ends = ["بقيت محاولة واحدة.", "بقيت محاولتان.", "بقيت 5 محاولات."];
    for (text, end) in tries.iter().zip(ends) {
        assert!(text.ends_with(end), "{text}");
    }
}

#[test]
fn a_new_password_names_its_least_length_in_arabic() {
    let ar = lang("ar");
    let new = format!("{FILL_ASK}{}", ask_id(PasswordAsk::New { min: 0 }));
    let start = "أنشئ كلمة مرور للنسخ الاحتياطي من ";
    let lengths = [(8, "8 أحرف"), (12, "12 حرفًا"), (64, "64 حرفًا")];
    for (n, length) in lengths {
        let text = ar.counted(&new, n);
        assert!(
            text.starts_with(&format!("{start}{length} على الأقل.")),
            "{text}"
        );
    }
}
