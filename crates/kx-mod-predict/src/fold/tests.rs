//! Folding in French, English and Arabic, the apostrophe, the table's soundness, and folding twice changing nothing.

use proptest::prelude::*;

use super::*;

mod table;

/// `word` folded with Unicode's default casing, as every language without casing rows folds it.
fn fold(word: &str) -> String {
    super::fold(word, Casing::root())
}

/// Random words tried by each property test.
const CASES: u32 = 1_024;
/// The first code point of Latin-1 Supplement's letters, `À`.
const LATIN_1_LETTERS: u32 = 0x00C0;
/// The last code point of Latin Extended-A, `ſ`.
const EXTENDED_A_END: u32 = 0x017F;
/// The first combining diacritic, the grave accent.
const COMBINING_FIRST: u32 = 0x0300;
/// The last combining diacritic of its block.
const COMBINING_LAST: u32 = 0x036F;
/// The Greek capitals, `Α` to `Ω`.
const GREEK_CAPITALS: (char, char) = ('\u{0391}', '\u{03A9}');
/// The Greek small letters, `α` to `ω`, final sigma included.
const GREEK_SMALL: (char, char) = ('\u{03B1}', '\u{03C9}');

/// Asserts that each `(word, folded)` pair folds as given.
fn folds(pairs: &[(&str, &str)]) {
    for (word, want) in pairs {
        assert_eq!(fold(word), *want, "{word}");
    }
}

#[test]
fn ecole_and_ecole_with_its_accent_share_a_key() {
    assert_eq!(fold("ecole"), fold("école"));
    assert_eq!(fold("École"), "ecole");
    assert_eq!(fold("ÉCOLE"), "ecole");
}

#[test]
fn french_words_lose_their_accents() {
    folds(&[
        ("être", "etre"),
        ("déjà", "deja"),
        ("français", "francais"),
        ("Ça", "ca"),
        ("Noël", "noel"),
        ("maïs", "mais"),
        ("où", "ou"),
        ("août", "aout"),
        ("hôtel", "hotel"),
        ("île", "ile"),
        ("L'Haÿ-les-Roses", "l'hay-les-roses"),
        ("à", "a"),
    ]);
}

#[test]
fn ligatures_spell_out() {
    folds(&[
        ("cœur", "coeur"),
        ("Œuvre", "oeuvre"),
        ("vitæ", "vitae"),
        ("straße", "strasse"),
    ]);
}

#[test]
fn other_latin_letters_fold_to_their_base() {
    folds(&[
        ("Łódź", "lodz"),
        ("Dvořák", "dvorak"),
        ("İstanbul", "istanbul"),
        ("Ångström", "angstrom"),
    ]);
}

#[test]
fn latin_without_accents_changes_only_case() {
    folds(&[
        ("Hello", "hello"),
        ("the", "the"),
        ("Don't", "don't"),
        ("LONDON", "london"),
    ]);
}

#[test]
fn decomposed_accents_fold_as_precomposed_ones_do() {
    assert_eq!(fold("e\u{0301}cole"), fold("école"));
    assert_eq!(fold("E\u{0301}COLE"), "ecole");
    folds(&[
        ("Noe\u{0308}l", "noel"),
        ("c\u{0327}a", "ca"),
        ("ho\u{0302}tel", "hotel"),
        ("a\u{0300}", "a"),
    ]);
}

#[test]
fn every_combining_diacritic_drops() {
    let marks = (COMBINING_FIRST..=COMBINING_LAST).filter_map(char::from_u32);
    for mark in marks {
        assert_eq!(fold(&format!("e{mark}")), "e", "U+{:04X}", u32::from(mark));
    }
}

#[test]
fn greek_final_sigma_folds_to_sigma() {
    assert_eq!(fold("ΟΔΥΣ"), "οδυσ");
    assert!(fold("οδυσσευς").starts_with(&fold("ΟΔΥΣ")));
    assert_eq!(fold("λόγος"), fold("λόγοσ"));
}

#[test]
fn arabic_madda_and_hamza_marks_stay() {
    for mark in ['\u{0653}', '\u{0654}', '\u{0655}'] {
        let word = format!("ا{mark}");
        assert_eq!(fold(&word), word, "U+{:04X}", u32::from(mark));
    }
}

#[test]
fn every_letter_of_latin_1_and_extended_a_folds_to_ascii() {
    let letters = (LATIN_1_LETTERS..=EXTENDED_A_END)
        .filter_map(char::from_u32)
        .filter(|c| c.is_alphabetic());
    for c in letters {
        let folded = fold(&c.to_string());
        let ascii = !folded.is_empty() && folded.bytes().all(|b| b.is_ascii_lowercase());
        assert!(ascii, "U+{:04X} folds to {folded:?}", u32::from(c));
    }
}

#[test]
fn arabic_alef_forms_fold_to_bare_alef() {
    folds(&[
        ("أن", "ان"),
        ("إن", "ان"),
        ("أنا", "انا"),
        ("إسلام", "اسلام"),
        ("آخر", "اخر"),
        ("ٱلْحَمْدُ", "الحمد"),
    ]);
    assert_eq!(fold("أن"), fold("إن"));
}

#[test]
fn alef_maksura_folds_to_ya() {
    folds(&[("على", "علي"), ("إلى", "الي"), ("مستشفى", "مستشفي")]);
}

#[test]
fn harakat_superscript_alef_and_tatweel_drop() {
    folds(&[
        ("مَدْرَسَة", "مدرسة"),
        ("كُتُب", "كتب"),
        ("محمّد", "محمد"),
        ("شكراً", "شكرا"),
        ("كتابٌ", "كتاب"),
        ("كتابٍ", "كتاب"),
        ("هٰذا", "هذا"),
        ("مـــرحبا", "مرحبا"),
        ("الســلام", "السلام"),
    ]);
}

#[test]
fn ta_marbuta_hamza_on_a_seat_and_other_letters_stay() {
    for word in ["مدرسة", "مسؤول", "سائل", "شيء", "ثقة", "كتب"] {
        assert_eq!(fold(word), word);
    }
}

#[test]
fn the_typographic_apostrophe_folds_to_the_plain_one() {
    assert_eq!(fold("l\u{2019}homme"), "l'homme");
    assert_eq!(fold("Don\u{2019}t"), fold("don't"));
    assert_eq!(apostrophe(), Some('\''));
    assert!(is_apostrophe('\'') && is_apostrophe('\u{2019}'));
    for other in ['a', '-', '`', '\u{2018}', '\u{02BC}'] {
        assert!(!is_apostrophe(other), "U+{:04X}", u32::from(other));
    }
}

/// A word mixing the table's characters with any others, so folding meets both.
fn word() -> impl Strategy<Value = String> {
    let listed: Vec<char> = Folds::parse(TABLE).unwrap().map.into_keys().collect();
    let pick = prop_oneof![
        prop::sample::select(listed),
        any::<char>(),
        prop::char::range('A', 'Z')
    ];
    prop::collection::vec(pick, 0..12).prop_map(|cs| cs.into_iter().collect())
}

/// A Greek word of capitals and small letters, so sigma meets every position.
fn greek() -> impl Strategy<Value = String> {
    let (capitals, small) = (GREEK_CAPITALS, GREEK_SMALL);
    let letter = prop_oneof![
        prop::char::range(capitals.0, capitals.1),
        prop::char::range(small.0, small.1),
    ];
    prop::collection::vec(letter, 0..10).prop_map(|cs| cs.into_iter().collect())
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(CASES))]

    #[test]
    fn a_greek_word_folds_to_a_prefix_of_its_longer_forms(start in greek(), rest in greek()) {
        let longer = fold(&format!("{start}{rest}"));
        prop_assert!(longer.starts_with(&fold(&start)), "{} then {}", start, rest);
    }

    #[test]
    fn folding_twice_equals_folding_once(word in word()) {
        let once = fold(&word);
        prop_assert_eq!(fold(&once), once);
    }

    #[test]
    fn folding_any_text_twice_equals_folding_once(word in "\\PC*") {
        let once = fold(&word);
        prop_assert_eq!(fold(&once), once);
    }
}
