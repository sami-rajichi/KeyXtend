//! Casing by language: Turkish's dotted and dotless i through capitals, lower case and folding, and Unicode's default
//! everywhere else.

use proptest::prelude::*;

use super::*;
use crate::fold::fold;

/// Random texts tried by the property test.
const CASES: u32 = 256;

#[test]
fn the_table_parses_and_holds_greek_dutch_and_turkish() {
    let langs = LANGS.as_ref().unwrap();
    let mut isos: Vec<&String> = langs.keys().collect();
    isos.sort();
    assert_eq!(isos, ["el", "nl", "tr"]);
    assert!(std::ptr::eq(Casing::of("fr"), Casing::root()));
}

#[test]
fn dutch_capitalises_ij_as_one_letter() {
    let nl = Casing::of("nl");
    assert_eq!(nl.capital("ijsselmeer"), "IJsselmeer");
    assert_eq!(nl.capital("ijs"), "IJs");
    assert_eq!(nl.upper("ijs"), "IJS");
    assert_eq!(Casing::of("en").capital("ijs"), "Ijs");
}

#[test]
fn greek_drops_the_tonos_in_all_capitals_and_keeps_it_on_a_first_capital() {
    let el = Casing::of("el");
    assert_eq!(el.upper("καλημέρα"), "ΚΑΛΗΜΕΡΑ");
    assert_eq!(el.upper("Άρης"), "ΑΡΗΣ");
    assert_eq!(el.upper("ΐ"), "Ϊ");
    assert_eq!(el.capital("όλα"), "Όλα");
    assert_eq!(el.lower("ΚΑΛΗΜΈΡΑ"), "καλημέρα");
}

#[test]
fn greek_all_capitals_keep_a_vowel_pair_apart_with_a_diaeresis() {
    let el = Casing::of("el");
    assert_eq!(el.upper("τσάι"), "ΤΣΑΪ");
    assert_eq!(el.upper("άυλος"), "ΑΫΛΟΣ");
    assert_eq!(el.upper("Άυλος"), "ΑΫΛΟΣ");
    assert_eq!(
        el.upper("είναι"),
        "ΕΙΝΑΙ",
        "the tonos on the pair's second vowel"
    );
}

#[test]
fn a_capitalised_turkish_i_takes_its_dot() {
    let tr = Casing::of("tr");
    assert_eq!(tr.capital("is"), "İs");
    assert_eq!(tr.capital("istanbul"), "İstanbul");
    assert_eq!(tr.upper("isim"), "İSİM");
    assert_eq!(tr.capital("ılık"), "Ilık");
    assert_eq!(Casing::of("en").capital("is"), "Is");
    assert_eq!(Casing::of("en").upper("isim"), "ISIM");
}

#[test]
fn turkish_lowers_i_to_dotless_i_and_dotted_i_to_i() {
    let tr = Casing::of("tr");
    assert_eq!(tr.lower("IRMAK"), "ırmak");
    assert_eq!(tr.lower("İSTANBUL"), "istanbul");
    assert_eq!(tr.lower("I\u{0307}STANBUL"), "istanbul");
    assert_eq!(Casing::of("en").lower("IRMAK"), "irmak");
}

#[test]
fn a_typed_capital_i_folds_to_dotless_i_in_turkish_and_to_i_in_english() {
    let (tr, en) = (Casing::of("tr"), Casing::of("en"));
    assert_eq!(fold("I", tr), "ı");
    assert_eq!(fold("I", en), "i");
    assert_eq!(fold("ılık", tr), "ılık");
    assert_eq!(fold("ılık", en), "ilik");
    assert_eq!(fold("İstanbul", tr), fold("istanbul", tr));
}

#[test]
fn a_broken_table_leaves_every_language_on_the_default() {
    assert!(
        parse("[tr]\nrows = [{ text = \"\", lower = \"i\", title = \"i\", upper = \"i\" }]")
            .is_none()
    );
    assert!(parse("[tr]\nrows = 1").is_none());
    assert!(parse("[tr]\nrows = []\nother = 1").is_none());
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(CASES))]

    #[test]
    fn the_default_casing_is_unicodes_and_turkish_differs_only_at_its_rows(text in "\\PC{0,12}") {
        let root = Casing::root();
        prop_assert_eq!(root.lower(&text), text.to_lowercase());
        prop_assert_eq!(root.upper(&text), text.to_uppercase());
        let rows = ['I', 'İ', 'i'];
        if !text.chars().any(|c| rows.contains(&c)) {
            prop_assert_eq!(Casing::of("tr").lower(&text), text.to_lowercase());
        }
        let tr = Casing::of("tr");
        let once = fold(&text, tr);
        prop_assert_eq!(fold(&once, tr), once);
    }
}
