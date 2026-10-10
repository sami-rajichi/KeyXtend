use super::*;

use super::testkeys::{letter, plain};

mod marks;

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
fn a_legend_from_any_arabic_block_takes_the_arabic_font() {
    // Persian's rial sign (Presentation Forms-A) and a lam-alef form (Forms-B), as a layout may type them.
    let rial = legend(
        &keys("\u{FDFC}", "\u{FEFB}", ""),
        LegendMods::default(),
        true,
        false,
    );
    assert!(rial.main_ar && rial.second_ar, "{rial:?}");
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
    let dead = |k: &KeyChars, m| slot(k, m).is_some_and(|s| k.is_dead(s));
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
        legend(&a, mods(false, false, true), false, false),
        Legend::default(),
        "no AltGr character: an empty key"
    );
}

#[test]
fn altgr_without_a_character_leaves_the_key_empty_whatever_shift_and_caps_say() {
    let a = letter("a", "A");
    for (shift, caps) in [(false, false), (true, false), (false, true), (true, true)] {
        assert_eq!(slot(&a, mods(shift, caps, true)), None, "{shift} {caps}");
    }
    assert_eq!(typed(&a, mods(true, false, true)), "");
}

/// Arabic (102) AZERTY's س: nothing with Shift, the riyal sign on `AltGr`.
fn seen() -> KeyChars {
    KeyChars {
        caps: "س".into(),
        ..keys("س", "", "\u{20C1}")
    }
}

#[test]
fn shift_on_a_key_that_types_nothing_shifted_leaves_it_empty_but_its_altgr_corner() {
    for caps in [false, true] {
        assert_eq!(slot(&seen(), mods(true, caps, false)), None, "{caps}");
    }
    let l = legend(&seen(), mods(true, false, false), true, true);
    let parts = (l.main.as_str(), l.second.as_str(), l.third.as_str());
    assert_eq!(
        parts,
        ("", "", "\u{20C1}"),
        "no letter, as Windows shows it"
    );
    assert!(!l.main_ar && !l.main_mark);
}

#[test]
fn caps_lock_falls_back_to_the_same_state_without_it() {
    let b = KeyChars {
        caps: String::new(),
        caps_shift: String::new(),
        ..keys("b", "B", "")
    };
    assert_eq!(
        slot(&b, mods(false, true, false)),
        Some(Slot::Base),
        "alone"
    );
    assert_eq!(typed(&b, mods(true, true, false)), "B", "with Shift");
    assert_eq!(slot(&seen(), mods(false, true, false)), Some(Slot::Caps));
}

#[test]
fn a_key_with_nothing_latched_is_read_in_its_base_even_when_it_types_nothing() {
    let none = KeyChars::default();
    assert_eq!(slot(&none, LegendMods::default()), Some(Slot::Base));
    assert_eq!(slot(&none, mods(false, true, false)), Some(Slot::Base));
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

#[test]
fn legend_mods_hide_shift_caps_and_altgr() {
    let all = LegendMods {
        shift: true,
        caps: true,
        altgr: true,
    };
    assert_eq!(format!("{all:?}"), "LegendMods { .. }");
}

#[test]
fn a_debug_print_hides_the_legends_as_they_tell_shift_caps_and_altgr() {
    let shifted = Legend {
        main: "Q".into(),
        second: "q".into(),
        third: "@".into(),
        main_ar: true,
        ..Legend::default()
    };
    assert_eq!(format!("{shifted:?}"), "Legend { .. }");
}
