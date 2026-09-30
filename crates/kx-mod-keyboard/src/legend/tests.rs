use super::*;

use super::testkeys::{letter, plain};

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

fn keys(base: &str, shift: &str, altgr: &str) -> KeyChars {
    plain(base, shift, altgr)
}

fn mods(shift: bool, caps: bool, altgr: bool) -> LegendMods {
    LegendMods { shift, caps, altgr }
}

#[test]
fn a_letter_shows_one_legend_and_caps_or_shift_flip_its_case() {
    let q = letter("q", "Q");
    assert_eq!(legend(&q, LegendMods::default(), false, false).main, "q");
    assert_eq!(legend(&q, mods(true, false, false), false, false).main, "Q");
    assert_eq!(legend(&q, mods(false, true, false), false, false).main, "Q");
    assert_eq!(
        legend(&q, mods(true, true, false), false, false).main,
        "q",
        "they cancel"
    );
    assert_eq!(legend(&q, LegendMods::default(), false, false).second, "");
}

#[test]
fn digits_and_symbols_show_the_other_case_in_the_corner() {
    let one = keys("1", "!", "");
    let l = legend(&one, LegendMods::default(), true, false);
    assert_eq!((l.main.as_str(), l.second.as_str()), ("1", "!"));
    let l = legend(&one, mods(true, false, false), true, false);
    assert_eq!((l.main.as_str(), l.second.as_str()), ("!", "1"));
    let semi = keys(";", ":", "");
    assert_eq!(
        legend(&semi, LegendMods::default(), false, false).second,
        ":"
    );
    let caps_digit = legend(&one, mods(false, true, false), true, false);
    assert_eq!(caps_digit.main, "1", "Caps Lock leaves digits alone");
}

#[test]
fn arabic_letters_show_the_shifted_form_only_on_the_top_row() {
    let dhal = keys("ذ", "\u{651}", "");
    let l = legend(&dhal, LegendMods::default(), true, true);
    assert_eq!(
        (l.main.as_str(), l.second.as_str()),
        ("ذ", "\u{25CC}\u{651}")
    );
    assert!(l.main_ar && l.second_ar);
    let dad = keys("ض", "\u{64E}", "");
    assert_eq!(legend(&dad, LegendMods::default(), false, true).second, "");
    let fatha = legend(&dad, mods(true, false, false), false, true);
    assert_eq!(
        fatha.main, "\u{25CC}\u{64E}",
        "a haraka sits on a dotted circle"
    );
}

#[test]
fn caps_lock_gives_what_the_layout_gives() {
    // French AZERTY: Caps Lock turns the é key into 2, as Shift does.
    let e2 = KeyChars {
        caps: "2".into(),
        caps_shift: "é".into(),
        ..plain("é", "2", "~")
    };
    assert_eq!(typed(&e2, mods(false, true, false)), "2");
    assert_eq!(typed(&e2, mods(true, true, false)), "é", "Shift undoes it");
    assert_eq!(typed(&e2, mods(false, true, true)), "~", "AltGr wins");
}

#[test]
fn a_dead_key_is_known_in_the_state_it_is_read_in() {
    // French AZERTY: the ^ key is dead alone and with Shift (¨).
    let hat = KeyChars {
        dead: vec![Slot::Base, Slot::Shift],
        ..plain("^", "¨", "")
    };
    let dead = |k: &KeyChars, m| k.is_dead(slot(k, m));
    assert!(dead(&hat, LegendMods::default()));
    assert!(dead(&hat, mods(true, false, false)));
    assert!(!dead(&keys("a", "A", ""), LegendMods::default()));
}

#[test]
fn lam_alef_is_one_legend_of_two_letters() {
    let la = keys("لا", "لآ", "");
    let l = legend(&la, LegendMods::default(), false, true);
    assert_eq!(l.main, "لا");
    assert!(l.main_ar);
}

#[test]
fn altgr_shows_in_the_corner_and_takes_over_when_latched() {
    let e = keys("e", "E", "€");
    let l = legend(&e, LegendMods::default(), false, false);
    assert_eq!((l.main.as_str(), l.third.as_str()), ("e", "€"));
    let l = legend(&e, mods(false, false, true), false, false);
    assert_eq!((l.main.as_str(), l.third.as_str()), ("€", ""));
    let a = keys("a", "A", "");
    assert_eq!(
        legend(&a, mods(false, false, true), false, false).main,
        "a",
        "no AltGr character"
    );
}

#[test]
fn altgr_without_a_character_keeps_shift_and_caps() {
    let a = letter("a", "A");
    assert_eq!(
        typed(&a, mods(true, false, true)),
        "A",
        "Shift still counts"
    );
    assert_eq!(typed(&a, mods(false, true, true)), "A", "Caps still counts");
    assert_eq!(slot(&a, mods(true, true, true)), Slot::CapsShift);
}

/// A two-key layout: q, and e with `euro` on `AltGr`.
fn table(euro: &str) -> BTreeMap<Scan, KeyChars> {
    let e = KeyChars {
        altgr: euro.into(),
        ..letter("e", "E")
    };
    BTreeMap::from([(Scan::new(0x10), letter("q", "Q")), (Scan::new(0x12), e)])
}

#[test]
fn the_mods_come_from_the_latches_and_caps_lock() {
    use crate::settings::Modifiers;
    let mut latches = Latches::new(Modifiers::Release);
    assert_eq!(
        LegendMods::of(&latches, true, &table("€")),
        mods(false, true, false)
    );
    latches.toggle(Latch::Shift, Scan::new(0x2A));
    latches.toggle(Latch::AltGr, Scan::new(0xE038));
    assert_eq!(
        LegendMods::of(&latches, false, &table("€")),
        mods(true, false, true)
    );
    assert_eq!(
        LegendMods::of(&latches, false, &table("")),
        mods(true, false, false),
        "AltGr counts only on a layout with AltGr characters"
    );
}
