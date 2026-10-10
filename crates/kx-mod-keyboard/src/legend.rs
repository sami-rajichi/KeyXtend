//! What a character key shows and types: the main legend, a second (shifted or base) one and the `AltGr` one,
//! as in the mock-up, read from the layout table in the state the latches and Caps Lock give.

use crate::latch::Latches;
use crate::spec::Latch;
use kx_module_api::is_arabic;
use kx_module_api::ports::{KeyChars, Scan, Slot};
use serde::Serialize;
use std::collections::BTreeMap;
use std::fmt;
use std::ops::RangeInclusive;

/// Drawn before Arabic marks that have no letter before them, so they have a base to sit on.
const DOTTED_CIRCLE: char = '\u{25CC}';
/// Arabic harakat, from fathatan to sukun.
const HARAKAT: RangeInclusive<char> = '\u{064B}'..='\u{0652}';

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

/// Starts a mark's name, a translation key such as `mark.fatha`.
pub const MARK_PREFIX: &str = "mark.";

/// Each haraka with its name's id after `MARK_PREFIX`, so readers can say a mark they cannot say bare.
pub const HARAKA_NAMES: [(char, &str); 8] = [
    ('\u{064B}', "fathatan"),
    ('\u{064C}', "dammatan"),
    ('\u{064D}', "kasratan"),
    ('\u{064E}', "fatha"),
    ('\u{064F}', "damma"),
    ('\u{0650}', "kasra"),
    ('\u{0651}', "shadda"),
    ('\u{0652}', "sukun"),
];

/// The names of the marks `s` holds, by translation key; none unless `s` is only marks with a name.
#[must_use]
pub fn mark_names(s: &str) -> Vec<String> {
    let name = |c: char| {
        HARAKA_NAMES
            .iter()
            .find(|&&(h, _)| h == c)
            .map(|&(_, id)| id)
    };
    let ids: Option<Vec<&str>> = s.chars().map(name).collect();
    ids.unwrap_or_default()
        .into_iter()
        .map(|id| format!("{MARK_PREFIX}{id}"))
        .collect()
}

/// The modifiers that change legends, and whether a click sends a shortcut.
#[derive(Clone, Copy, Default, PartialEq, Eq)]
#[allow(
    clippy::struct_excessive_bools,
    reason = "Each flag is one latch state the legend reads; they are set together from the latches."
)]
pub struct LegendMods {
    /// Shift is latched.
    pub shift: bool,
    /// Caps Lock is on.
    pub caps: bool,
    /// `AltGr` is latched and the layout has `AltGr` characters.
    pub altgr: bool,
    /// Ctrl, Alt or Win is latched, or `AltGr` on a layout without `AltGr` characters (Right Alt): a click sends a
    /// shortcut, so no key is empty.
    pub shortcut: bool,
}

/// Shows no field, so latches and Caps Lock stay out of any log, as `Latches` does.
impl fmt::Debug for LegendMods {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("LegendMods").finish_non_exhaustive()
    }
}

impl LegendMods {
    /// The modifiers from `latches` and Caps Lock `caps`; `AltGr` counts only when `keys` has an `AltGr` character.
    #[must_use]
    pub fn of(latches: &Latches, caps: bool, keys: &BTreeMap<Scan, KeyChars>) -> Self {
        let has_altgr = keys.values().any(|k| !k.altgr.is_empty());
        let on = |l| latches.is_on(l);
        let right_alt = on(Latch::AltGr) && !has_altgr;
        Self {
            shift: on(Latch::Shift),
            caps,
            altgr: has_altgr && on(Latch::AltGr),
            shortcut: on(Latch::Ctrl) || on(Latch::Alt) || on(Latch::Win) || right_alt,
        }
    }
}

/// A character key's legends, ready to draw.
#[derive(Clone, Default, PartialEq, Eq, Serialize)]
#[allow(
    clippy::struct_excessive_bools,
    reason = "Each legend's font flags sit flat beside it in the JSON that QML reads."
)]
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
    /// `third` needs the Arabic font.
    pub third_ar: bool,
    /// `main` is marks on the dotted circle, drawn in the theme's mark font so they attach to it.
    pub main_mark: bool,
    /// `second` is marks on the dotted circle.
    pub second_mark: bool,
    /// `third` is marks on the dotted circle.
    pub third_mark: bool,
}

/// Shows no field: what a key shows tells Shift, Caps Lock and `AltGr`, as `LegendMods` hides them.
impl fmt::Debug for Legend {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Legend").finish_non_exhaustive()
    }
}

/// The first state `m` reads where key `k` types something: the one `m` wants, then the same without Caps Lock.
fn found(k: &KeyChars, m: LegendMods) -> Option<Slot> {
    let (want, uncapped) = match (m.altgr, m.shift, m.caps) {
        (true, _, _) => (Slot::AltGr, Slot::AltGr),
        (false, true, true) => (Slot::CapsShift, Slot::Shift),
        (false, true, false) => (Slot::Shift, Slot::Shift),
        (false, false, true) => (Slot::Caps, Slot::Base),
        (false, false, false) => (Slot::Base, Slot::Base),
    };
    [want, uncapped].into_iter().find(|&s| !k.get(s).is_empty())
}

/// The one rule for an empty key: Shift or `AltGr` latched with no shortcut latch, and nothing typed in that state.
/// It shows no letter, readers hear its empty name, and a click sends nothing, as Windows' keyboard shows it.
#[must_use]
pub fn is_empty(k: &KeyChars, m: LegendMods) -> bool {
    (m.altgr || m.shift) && !m.shortcut && found(k, m).is_none()
}

/// The state key `k` is read in with `m`, none for an empty key (`is_empty`); Caps Lock that types nothing falls
/// back to the same state without it, and any other state to the base.
#[must_use]
pub fn slot(k: &KeyChars, m: LegendMods) -> Option<Slot> {
    (!is_empty(k, m)).then(|| found(k, m).unwrap_or(Slot::Base))
}

/// True for one character that has upper and lower case.
fn is_letter(s: &str) -> bool {
    let mut it = s.chars();
    matches!((it.next(), it.next()), (Some(c), None) if c.to_lowercase().ne(c.to_uppercase()))
}

/// True when `s` is Arabic letters only, such as ض or lam-alef; marks, digits and punctuation are not letters.
fn arabic_letters(s: &str) -> bool {
    !s.is_empty() && is_arabic(s) && s.chars().all(|c| c.is_alphabetic() && !is_mark(c))
}

/// True when `s` starts with a mark, which has no letter before it, so it is drawn on a dotted circle.
fn on_circle(s: &str) -> bool {
    s.chars().next().is_some_and(is_mark)
}

/// `s` as drawn: marks with no letter before them, one or more, get a dotted circle to sit on.
#[must_use]
pub fn shown(s: &str) -> String {
    if on_circle(s) {
        format!("{DOTTED_CIRCLE}{s}")
    } else {
        s.to_string()
    }
}

/// What the key types with `m`, empty for an empty key: the face sends this, never the drawn legend with its circle.
#[must_use]
pub fn typed(k: &KeyChars, m: LegendMods) -> &str {
    slot(k, m).map_or("", |s| k.get(s))
}

/// The legends of key `k`: the top row and non-letters of left-to-right layouts also show the other case.
/// The `AltGr` corner shows while `AltGr` is not latched, except on an Arabic letter, which it would crowd (owner
/// 2026-10-10); an empty key shows no main or second legend.
#[must_use]
pub fn legend(k: &KeyChars, m: LegendMods, top_row: bool, rtl: bool) -> Legend {
    let s = slot(k, m);
    let main = s.map_or("", |s| k.get(s));
    let other = if main == k.base { &k.shift } else { &k.base };
    let wants = s.is_some() && (top_row || (!rtl && !is_letter(&k.base)));
    let second = if wants && !other.is_empty() && other != main {
        other.as_str()
    } else {
        ""
    };
    let third = if m.altgr || arabic_letters(&k.base) {
        ""
    } else {
        k.altgr.as_str()
    };
    Legend {
        main: shown(main),
        second: shown(second),
        third: shown(third),
        main_ar: is_arabic(main),
        second_ar: is_arabic(second),
        third_ar: is_arabic(third),
        main_mark: on_circle(main),
        second_mark: on_circle(second),
        third_mark: on_circle(third),
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
