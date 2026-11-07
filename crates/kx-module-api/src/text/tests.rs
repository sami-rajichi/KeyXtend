//! The neutral fold for search: capitals, accents, harakat, tatweel and the alef forms match in any language; the
//! wiped fold agrees with it, and a run of blanks reads as one space.

use super::*;
use crate::Zeroizing;

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

/// Texts the wiped fold must fold as `fold_any` does: French, Arabic with harakat and tatweel, a joiner, the Turkish
/// dotted capital, German ß, a ligature, and Greek with a final sigma in capitals.
const SAMPLES: [&str; 10] = [
    "Crédit Agricole",
    "أَحْمَد مـــرحبا على",
    "عبد\u{200c}الله",
    "İSTANBUL",
    "Straße",
    "ﬃ",
    "ΟΔΟΣ ΣΑΣ",
    "l’école",
    "",
    "MIXED Ägypten مصر",
];

/// `text` through the wiped fold.
fn wiped(text: &str) -> String {
    let mut out = Zeroizing::new(String::new());
    fold_any_into(text, &mut out);
    out.as_str().to_owned()
}

#[test]
fn the_wiped_fold_folds_as_fold_any_does() {
    for text in SAMPLES {
        assert_eq!(wiped(text), fold_any(text), "{text:?}");
    }
}

#[test]
fn the_wiped_fold_replaces_what_the_buffer_held_and_writes_in_place_when_it_has_room() {
    let mut out = Zeroizing::new(String::with_capacity(64));
    fold_any_into("Une très longue phrase", &mut out);
    let at = out.as_ptr();
    fold_any_into("ÉCOLE", &mut out);
    assert_eq!(out.as_str(), "ecole");
    assert_eq!(out.as_ptr(), at, "never moved");
}

#[test]
fn a_run_of_blanks_line_breaks_and_controls_reads_as_one_space() {
    let line = |text: &str| one_line(text.chars()).collect::<String>();
    assert_eq!(line("credit\r\n\tagricole"), "credit agricole");
    assert_eq!(line("  a\u{a0}\u{7}b  "), " a b ");
    assert_eq!(line("a\u{2028}\u{2029}b\u{85}"), "a b ");
    assert_eq!(
        line("عبد\u{200c}الله"),
        "عبد\u{200c}الله",
        "a joiner is no blank"
    );
    assert_eq!(line(""), "");
}

proptest::proptest! {
    #[test]
    fn the_wiped_fold_agrees_with_fold_any_on_any_text(text in proptest::prelude::any::<String>()) {
        proptest::prop_assert_eq!(wiped(&text), fold_any(&text));
    }
}

#[test]
fn a_blank_run_reads_as_the_one_space() {
    assert_eq!(one_line("a\tb".chars()).nth(1), Some(SPACE));
    assert_eq!(SPACE, ' ');
}
