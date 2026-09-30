//! What a character key shows and types: the main legend, a second (shifted or base) one and the `AltGr` one,
//! as in the mock-up, read from the layout table in the state the latches and Caps Lock give.

use crate::latch::Latches;
use crate::spec::Latch;
use kx_module_api::ports::{KeyChars, Scan, Slot};
use serde::Serialize;
use std::collections::BTreeMap;
use std::ops::RangeInclusive;

/// Drawn before a lone Arabic mark so it has a base to sit on.
const DOTTED_CIRCLE: char = '\u{25CC}';
/// Arabic harakat, from fathatan to sukun.
const HARAKAT: RangeInclusive<char> = '\u{064B}'..='\u{0652}';
/// The Arabic block, which picks the Arabic font.
const ARABIC: RangeInclusive<char> = '\u{0600}'..='\u{06FF}';

/// True for an Arabic haraka, which sits on the letter before it.
#[must_use]
pub fn is_haraka(c: char) -> bool {
    HARAKAT.contains(&c)
}

/// Arabic combining marks besides the harakat: Quranic signs, maddah and hamza marks, the superscript alef,
/// and the small high and low signs (Unicode's Arabic nonspacing marks).
const MORE_MARKS: [RangeInclusive<char>; 7] = [
    '\u{0610}'..='\u{061A}',
    '\u{0653}'..='\u{065F}',
    '\u{0670}'..='\u{0670}',
    '\u{06D6}'..='\u{06DC}',
    '\u{06DF}'..='\u{06E4}',
    '\u{06E7}'..='\u{06E8}',
    '\u{06EA}'..='\u{06ED}',
];

/// True for any Arabic combining mark: a haraka or another mark that sits on the letter before it.
#[must_use]
pub fn is_mark(c: char) -> bool {
    is_haraka(c) || MORE_MARKS.iter().any(|r| r.contains(&c))
}

/// The modifiers that change legends.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct LegendMods {
    /// Shift is latched.
    pub shift: bool,
    /// Caps Lock is on.
    pub caps: bool,
    /// `AltGr` is latched and the layout has `AltGr` characters.
    pub altgr: bool,
}

impl LegendMods {
    /// The modifiers from `latches` and Caps Lock `caps`; `AltGr` counts only when `keys` has an `AltGr` character.
    #[must_use]
    pub fn of(latches: &Latches, caps: bool, keys: &BTreeMap<Scan, KeyChars>) -> Self {
        let has_altgr = keys.values().any(|k| !k.altgr.is_empty());
        Self {
            shift: latches.is_on(Latch::Shift),
            caps,
            altgr: has_altgr && latches.is_on(Latch::AltGr),
        }
    }
}

/// A character key's legends, ready to draw.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct Legend {
    /// What the key types now.
    pub main: String,
    /// The other of base and shifted, in the top-left corner.
    pub second: String,
    /// The `AltGr` character, in the bottom-right corner.
    pub third: String,
    /// `main` needs the Arabic font.
    pub main_ar: bool,
    /// `second` needs the Arabic font.
    pub second_ar: bool,
}

/// The state key `k` is read in with `m`; a state that types nothing falls back to a simpler one.
#[must_use]
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
#[must_use]
pub fn is_arabic(s: &str) -> bool {
    s.chars().any(|c| ARABIC.contains(&c))
}

/// `s` as drawn: a lone haraka gets a dotted circle to sit on.
#[must_use]
pub fn shown(s: &str) -> String {
    let mut it = s.chars();
    match (it.next(), it.next()) {
        (Some(c), None) if is_haraka(c) => format!("{DOTTED_CIRCLE}{c}"),
        _ => s.to_string(),
    }
}

/// What the key types with `m`: the face sends this, never the drawn legend with its dotted circle.
#[must_use]
pub fn typed(k: &KeyChars, m: LegendMods) -> &str {
    k.get(slot(k, m))
}

/// The legends of key `k`: the top row and non-letters of left-to-right layouts also show the other case.
#[must_use]
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
    #[must_use]
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
    #[must_use]
    pub fn letter(base: &str, shift: &str) -> KeyChars {
        KeyChars {
            caps: shift.into(),
            caps_shift: base.into(),
            ..plain(base, shift, "")
        }
    }

    /// A dead key with no Shift or `AltGr` character.
    #[must_use]
    pub fn dead(accent: &str) -> KeyChars {
        KeyChars {
            dead: vec![Slot::Base, Slot::Caps],
            ..plain(accent, "", "")
        }
    }
}

#[cfg(test)]
mod tests;
