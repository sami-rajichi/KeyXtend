//! Arabic marks: which characters are marks, the dotted circle they sit on, their font flag and their names.

use super::*;

#[test]
fn every_arabic_combining_mark_is_a_mark_but_letters_are_not() {
    let common = [
        '\u{064E}', '\u{0652}', '\u{0653}', '\u{0654}', '\u{065F}', '\u{0670}',
    ];
    let quranic = [
        '\u{0610}', '\u{061A}', '\u{06D6}', '\u{06DC}', '\u{06DF}', '\u{06E4}', '\u{06E7}',
        '\u{06ED}',
    ];
    for c in common.into_iter().chain(quranic) {
        assert!(is_mark(c), "{:04X}", c as u32);
    }
    let letters = [
        '\u{0645}', '\u{0627}', '\u{0660}', '\u{06DD}', '\u{06E5}', '\u{06E9}', 'a',
    ];
    for c in letters {
        assert!(!is_mark(c), "{:04X}", c as u32);
    }
}

#[test]
fn any_mark_or_pair_of_marks_sits_on_a_dotted_circle() {
    assert_eq!(
        shown("\u{651}\u{64E}"),
        "\u{25CC}\u{651}\u{64E}",
        "shadda, fatha"
    );
    assert_eq!(shown("\u{653}"), "\u{25CC}\u{653}", "maddah, not a haraka");
    assert_eq!(
        shown("\u{628}\u{64E}"),
        "\u{628}\u{64E}",
        "a letter carries it"
    );
    assert_eq!(shown(""), "");
}

#[test]
fn the_altgr_marks_sit_on_a_dotted_circle_in_the_arabic_font() {
    // Shadda and a vowel on AltGr: in the corner of a key with no Arabic letter, and on ض with AltGr latched.
    let quote = keys("'", "4", "\u{651}\u{64E}");
    let l = legend(&quote, LegendMods::default(), false, true);
    assert_eq!(l.third, "\u{25CC}\u{651}\u{64E}");
    assert!(l.third_ar);
    let dad = keys("ض", "\u{64E}", "\u{651}\u{64E}");
    let l = legend(&dad, mods(false, false, true), false, true);
    assert_eq!(l.main, "\u{25CC}\u{651}\u{64E}", "AltGr latched");
    assert!(l.main_ar);
    let euro = legend(&keys("e", "E", "€"), LegendMods::default(), false, false);
    assert!(!euro.third_ar, "the euro keeps the Latin font");
}

#[test]
fn only_marks_on_their_circle_are_flagged_for_the_mark_font() {
    let quote = keys("'", "\u{64E}", "\u{651}\u{64E}");
    let l = legend(&quote, LegendMods::default(), true, true);
    assert_eq!(
        (l.main_mark, l.second_mark, l.third_mark),
        (false, true, true)
    );
    let dad = keys("ض", "\u{64E}", "\u{651}\u{64E}");
    let l = legend(&dad, mods(false, false, true), false, true);
    assert!(l.main_mark && !l.third_mark, "AltGr latched: {l:?}");
    let carried = legend(
        &keys("\u{628}\u{64E}", "", ""),
        LegendMods::default(),
        false,
        true,
    );
    assert!(!carried.main_mark, "a letter carries its mark");
    let plain = legend(&keys("e", "E", "€"), LegendMods::default(), true, false);
    assert!(!(plain.main_mark || plain.second_mark || plain.third_mark));
}

#[test]
fn readers_hear_marks_by_name_only_when_every_mark_has_one() {
    assert_eq!(mark_names("\u{651}\u{64E}"), ["mark.shadda", "mark.fatha"]);
    assert_eq!(mark_names("\u{64B}"), ["mark.fathatan"]);
    assert!(mark_names("ض").is_empty(), "a letter says itself");
    assert!(
        mark_names("\u{628}\u{64E}").is_empty(),
        "a letter with its mark"
    );
    assert!(mark_names("\u{653}").is_empty(), "a mark with no name");
    assert!(mark_names("").is_empty());
}

#[test]
fn every_haraka_has_its_own_name() {
    let named: Vec<char> = HARAKA_NAMES.iter().map(|&(c, _)| c).collect();
    assert_eq!(named, HARAKAT.collect::<Vec<char>>());
    let ids: std::collections::BTreeSet<&str> = HARAKA_NAMES.iter().map(|&(_, id)| id).collect();
    assert_eq!(ids.len(), HARAKA_NAMES.len());
}
