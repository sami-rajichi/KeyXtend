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
fn tatweel_harakat_the_alef_forms_and_alef_maksura_all_fold() {
    assert_eq!(fold_any("مـــرحبا"), "مرحبا", "tatweel");
    assert_eq!(fold_any("مَدْرَسَةٌ"), "مدرسة", "harakat");
    for alef in ["أ", "إ", "آ", "ٱ"] {
        assert_eq!(fold_any(alef), "ا", "{alef}");
    }
    assert_eq!(fold_any("على"), "علي", "alef maksura");
}

#[test]
fn a_capital_i_folds_to_i_in_any_language() {
    assert_eq!(fold_any("ILIK"), "ilik", "never the Turkish dotless ı");
    assert_eq!(fold_any("ılık"), "ilik");
}
