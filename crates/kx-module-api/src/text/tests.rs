//! The neutral fold for search: capitals, accents, harakat, tatweel and the alef forms match in any language.

use super::*;

/// True when `text` holds `query` once both are folded for search.
fn finds(text: &str, query: &str) -> bool {
    fold_any(text).contains(&fold_any(query))
}

#[test]
fn ecole_in_capitals_finds_ecole_with_its_accent() {
    assert!(finds("une école", "ECOLE"));
    assert!(finds("ÉCOLE", "ecole"));
}

#[test]
fn arsalt_without_its_hamza_finds_it_with_hamza_and_harakat() {
    // "I sent the file" with a damma; the query "I sent" is typed without the hamza.
    assert!(finds("أرسلتُ الملف", "ارسلت"));
    assert!(finds("إلى", "الي"), "alef maksura reads as ya");
    assert!(finds("مـــرحبا", "مرحبا"), "tatweel drops");
}

#[test]
fn the_neutral_fold_is_the_default_casing_fold() {
    for text in ["Istanbul", "ılık", "ΟΔΥΣ", "l\u{2019}homme"] {
        assert_eq!(fold_any(text), fold(text, Casing::root()), "{text}");
    }
}
