//! The fold table's soundness: it parses, maps nothing to itself, and refuses repeats, chains and a bad apostrophe.

use super::super::*;

#[test]
fn the_shipped_table_parses_and_maps_nothing_to_itself() {
    let folds = Folds::parse(TABLE).unwrap();
    assert!(!folds.map.is_empty());
    for (c, to) in &folds.map {
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

#[test]
fn the_apostrophe_section_folds_every_form_to_one_character() {
    let one = Folds::parse("[apostrophe.map]\n\"\u{2019}\" = \"'\"\n\"\u{02BC}\" = \"'\"").unwrap();
    assert_eq!(one.apostrophe, Some('\''));
    let none = Folds::parse("[a.map]\n\"é\" = \"e\"").unwrap();
    assert_eq!(none.apostrophe, None);
    for bad in [
        "[apostrophe.map]\n\"\u{2019}\" = \"'\"\n\"\u{02BC}\" = \"`\"",
        "[apostrophe.map]\n\"\u{2019}\" = \"''\"",
        "[apostrophe]\ndrop = [\"\u{2019}\"]",
        "[apostrophe]\nmap = {}",
    ] {
        assert_eq!(
            Folds::parse(bad).unwrap_err(),
            FoldTableError::Apostrophe,
            "{bad}"
        );
    }
}
