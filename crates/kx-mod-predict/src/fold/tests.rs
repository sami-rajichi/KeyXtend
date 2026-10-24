//! Folding in French, English and Arabic, the shipped table's soundness, and folding twice changing nothing.

use proptest::prelude::*;

use super::*;

/// Random words tried by each property test.
const CASES: u32 = 1_024;
/// The first code point of Latin-1 Supplement's letters, `À`.
const LATIN_1_LETTERS: u32 = 0x00C0;
/// The last code point of Latin Extended-A, `ſ`.
const EXTENDED_A_END: u32 = 0x017F;

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
fn the_shipped_table_parses_and_maps_nothing_to_itself() {
    let folds = Folds::parse(TABLE).unwrap();
    assert!(!folds.0.is_empty());
    for (c, to) in &folds.0 {
        assert_ne!(
            to.as_deref(),
            Some(c.to_string().as_str()),
            "U+{:04X}",
            u32::from(*c)
        );
    }
}

#[test]
fn a_character_listed_twice_is_refused() {
    let twice = "[a]\ndrop = [\"é\"]\n[b.map]\n\"é\" = \"e\"";
    assert_eq!(Folds::parse(twice).unwrap_err(), FoldTableError::Twice('é'));
    let dropped_twice = "[a]\ndrop = [\"\\u0640\", \"\\u0640\"]";
    assert_eq!(
        Folds::parse(dropped_twice).unwrap_err(),
        FoldTableError::Twice('\u{0640}')
    );
    let same_key = "[a.map]\n\"é\" = \"e\"\n\"\\u00e9\" = \"e\"";
    assert!(matches!(
        Folds::parse(same_key),
        Err(FoldTableError::Parse(_))
    ));
}

#[test]
fn a_broken_table_is_refused() {
    let cases = [
        ("[a.map]\n\"e\" = \"e\"", FoldTableError::Itself('e')),
        ("[a.map]\n\"É\" = \"e\"", FoldTableError::Upper('É')),
        (
            "[a.map]\n\"é\" = \"è\"\n\"è\" = \"e\"",
            FoldTableError::Chained('é'),
        ),
        ("[a.map]\n\"é\" = \"E\"", FoldTableError::Chained('é')),
    ];
    for (text, want) in cases {
        assert_eq!(Folds::parse(text).unwrap_err(), want, "{text}");
    }
    for bad in [
        "[a]\nmap = 1",
        "[a]\nkeep = []",
        "[a.map]\n\"ab\" = \"c\"",
        "[a]\ndrop = [\"ab\"]",
    ] {
        assert!(
            matches!(Folds::parse(bad), Err(FoldTableError::Parse(_))),
            "{bad}"
        );
    }
}

/// A word mixing the table's characters with any others, so folding meets both.
fn word() -> impl Strategy<Value = String> {
    let listed: Vec<char> = Folds::parse(TABLE).unwrap().0.into_keys().collect();
    let pick = prop_oneof![
        prop::sample::select(listed),
        any::<char>(),
        prop::char::range('A', 'Z')
    ];
    prop::collection::vec(pick, 0..12).prop_map(|cs| cs.into_iter().collect())
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(CASES))]

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
