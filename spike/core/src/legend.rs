//! What a character key shows: the main legend, a second (shifted or base) one and the AltGr one, as in the mock-up.

use std::ops::RangeInclusive;

use serde::Serialize;

/// Drawn before a lone Arabic mark so it has a base to sit on.
const DOTTED_CIRCLE: char = '\u{25CC}';
/// Arabic harakat, from fathatan to sukun.
const HARAKAT: RangeInclusive<char> = '\u{064B}'..='\u{0652}';
/// The Arabic block, which picks the Arabic font.
const ARABIC: RangeInclusive<char> = '\u{0600}'..='\u{06FF}';

/// A modifier state a key is read in.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Slot {
    /// No modifier.
    Base,
    /// Shift.
    Shift,
    /// AltGr.
    AltGr,
    /// Caps Lock on.
    Caps,
    /// Caps Lock on, with Shift.
    CapsShift,
}

/// What one key types in one layout, per modifier state; empty when it types nothing.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct KeyChars {
    /// No modifier.
    pub base: String,
    /// Shift.
    pub shift: String,
    /// AltGr.
    pub altgr: String,
    /// Caps Lock on, as the layout gives it.
    pub caps: String,
    /// Caps Lock on, with Shift.
    pub caps_shift: String,
    /// The states whose character is a dead key, which waits for the next key to add its accent.
    pub dead: Vec<Slot>,
}

impl KeyChars {
    /// The text in state `s`.
    pub fn get(&self, s: Slot) -> &str {
        match s {
            Slot::Base => &self.base,
            Slot::Shift => &self.shift,
            Slot::AltGr => &self.altgr,
            Slot::Caps => &self.caps,
            Slot::CapsShift => &self.caps_shift,
        }
    }

    /// True when state `s` is a dead key, so the app must see a real key press.
    pub fn is_dead(&self, s: Slot) -> bool {
        self.dead.contains(&s)
    }
}

/// The modifiers that change legends.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct LegendMods {
    /// Shift is latched.
    pub shift: bool,
    /// Caps Lock is on.
    pub caps: bool,
    /// AltGr is latched and the layout has AltGr characters.
    pub altgr: bool,
}

/// A character key's legends, ready to draw.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct Legend {
    /// What the key types now.
    pub main: String,
    /// The other of base and shifted, in the top-left corner.
    pub second: String,
    /// The AltGr character, in the bottom-right corner.
    pub third: String,
    /// `main` needs the Arabic font.
    pub main_ar: bool,
    /// `second` needs the Arabic font.
    pub second_ar: bool,
}

/// The state key `k` is read in with `m`; a state that types nothing falls back to a simpler one.
pub fn slot(k: &KeyChars, m: LegendMods) -> Slot {
    let (want, simpler) = match (m.caps, m.shift) {
        (true, true) => (Slot::CapsShift, Slot::Shift),
        (true, false) => (Slot::Caps, Slot::Base),
        (false, true) => (Slot::Shift, Slot::Base),
        (false, false) => (Slot::Base, Slot::Base),
    };
    let altgr = m.altgr.then_some(Slot::AltGr);
    altgr
        .into_iter()
        .chain([want, simpler])
        .find(|&s| !k.get(s).is_empty())
        .unwrap_or(Slot::Base)
}

/// True for one character that has upper and lower case.
fn is_letter(s: &str) -> bool {
    let mut it = s.chars();
    matches!((it.next(), it.next()), (Some(c), None) if c.to_lowercase().ne(c.to_uppercase()))
}

/// True when `s` has an Arabic character.
pub fn is_arabic(s: &str) -> bool {
    s.chars().any(|c| ARABIC.contains(&c))
}

/// `s` as drawn: a lone haraka gets a dotted circle to sit on.
pub fn shown(s: &str) -> String {
    let mut it = s.chars();
    match (it.next(), it.next()) {
        (Some(c), None) if HARAKAT.contains(&c) => format!("{DOTTED_CIRCLE}{c}"),
        _ => s.to_string(),
    }
}

/// What the key types with `m`: the face sends this, never the drawn legend with its dotted circle.
pub fn typed(k: &KeyChars, m: LegendMods) -> &str {
    k.get(slot(k, m))
}

/// The legends of key `k`: the top row and non-letters of left-to-right layouts also show the other case.
pub fn legend(k: &KeyChars, m: LegendMods, top_row: bool, rtl: bool) -> Legend {
    let main = typed(k, m);
    let other = if main == k.base { &k.shift } else { &k.base };
    let wants = top_row || (!rtl && !is_letter(&k.base));
    let second = if wants && !other.is_empty() && other != main {
        other.as_str()
    } else {
        ""
    };
    let third = if m.altgr { "" } else { k.altgr.as_str() };
    Legend {
        main: shown(main),
        second: shown(second),
        third: third.to_string(),
        main_ar: is_arabic(main),
        second_ar: is_arabic(second),
    }
}

/// Keys for tests in any module.
#[cfg(test)]
pub(crate) mod testkeys {
    use super::{KeyChars, Slot};

    /// A key that Caps Lock leaves alone.
    pub fn plain(base: &str, shift: &str, altgr: &str) -> KeyChars {
        KeyChars {
            base: base.into(),
            shift: shift.into(),
            altgr: altgr.into(),
            caps: base.into(),
            caps_shift: shift.into(),
            dead: Vec::new(),
        }
    }

    /// A letter: Caps Lock swaps its cases.
    pub fn letter(base: &str, shift: &str) -> KeyChars {
        KeyChars {
            caps: shift.into(),
            caps_shift: base.into(),
            ..plain(base, shift, "")
        }
    }

    /// A dead key with no Shift or AltGr character.
    pub fn dead(accent: &str) -> KeyChars {
        KeyChars {
            dead: vec![Slot::Base, Slot::Caps],
            ..plain(accent, "", "")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use super::testkeys::{letter, plain};

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
}
