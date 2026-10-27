//! Which script a letter belongs to, one table for the pack builder and the tracker: a word mixing scripts is never a
//! word, and each script's row says which way it reads and what parts its words.

use std::cmp::Reverse;

/// A writing system, as far as the words need to tell them apart.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Script {
    /// Joins any script: modifier letters and combining marks.
    Common,
    /// Latin, with its extensions (Vietnamese included).
    Latin,
    /// Greek.
    Greek,
    /// Cyrillic.
    Cyrillic,
    /// Armenian.
    Armenian,
    /// Hebrew.
    Hebrew,
    /// Syriac.
    Syriac,
    /// Arabic, with its presentation forms.
    Arabic,
    /// Thaana.
    Thaana,
    /// N'Ko.
    Nko,
    /// Devanagari.
    Devanagari,
    /// Bengali.
    Bengali,
    /// Tamil.
    Tamil,
    /// Thai.
    Thai,
    /// Lao.
    Lao,
    /// Myanmar.
    Myanmar,
    /// Khmer.
    Khmer,
    /// Georgian, with its Mtavruli capitals and Nuskhuri.
    Georgian,
    /// Han and kana: Japanese mixes kanji and kana in one word, so they count as one.
    Han,
    /// Hangul, which Korean spaces between words.
    Hangul,
    /// Any letter of a script this table does not list.
    Other,
}

/// What a script's words need from the chips.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Facts {
    /// Written right to left, after the bidirectional classes of its letters (R and AL).
    rtl: bool,
    /// The text between two words, after UAX #14's line-break classes: nothing for ID and SA scripts.
    gap: &'static str,
    /// The other scripts its writing system holds, after ISO 15924's composite codes.
    mixes: &'static [Script],
}

/// A script that spaces its words and reads left to right.
const LTR: Facts = Facts {
    rtl: false,
    gap: " ",
    mixes: &[],
};
/// A script that spaces its words and reads right to left.
const RTL: Facts = Facts { rtl: true, ..LTR };
/// A script whose lines break between words without spaces (UAX #14 classes ID and SA).
const UNSPACED: Facts = Facts { gap: "", ..LTR };
/// Korean: ISO 15924's Kore writes Hangul with Han (Hanja).
const KORE: Facts = Facts {
    mixes: &[Script::Han],
    ..LTR
};

/// Each script's facts; a test checks every script has one row.
const FACTS: &[(Script, Facts)] = &[
    (Script::Common, LTR),
    (Script::Latin, LTR),
    (Script::Greek, LTR),
    (Script::Cyrillic, LTR),
    (Script::Armenian, LTR),
    (Script::Hebrew, RTL),
    (Script::Syriac, RTL),
    (Script::Arabic, RTL),
    (Script::Thaana, RTL),
    (Script::Nko, RTL),
    (Script::Devanagari, LTR),
    (Script::Bengali, LTR),
    (Script::Tamil, LTR),
    (Script::Thai, UNSPACED),
    (Script::Lao, UNSPACED),
    (Script::Myanmar, UNSPACED),
    (Script::Khmer, UNSPACED),
    (Script::Georgian, LTR),
    (Script::Han, UNSPACED),
    (Script::Hangul, KORE),
    (Script::Other, LTR),
];

impl Script {
    /// True for a script written right to left.
    #[must_use]
    pub fn rtl(self) -> bool {
        self.facts().rtl
    }

    /// The text a word chip types after a word of this script: a space, or nothing where words are not spaced.
    #[must_use]
    pub fn gap(self) -> &'static str {
        self.facts().gap
    }

    /// True when this script spaces its words, so word chips serve it.
    #[must_use]
    pub fn spaced(self) -> bool {
        !self.gap().is_empty()
    }

    /// True when this script's writing system holds `other`: it is this script or one it mixes in.
    #[must_use]
    pub fn holds(self, other: Self) -> bool {
        self == other || self.facts().mixes.contains(&other)
    }

    /// This script's row; an unlisted script, which a test rules out, takes `Other`'s.
    fn facts(self) -> Facts {
        let row = |s: Self| FACTS.iter().find(|(script, _)| *script == s).map(|r| r.1);
        row(self).or_else(|| row(Self::Other)).unwrap_or(LTR)
    }
}

/// The Unicode blocks of each script, from the Unicode block list; a letter outside them is `Other`.
const BLOCKS: &[(char, char, Script)] = &[
    ('A', 'Z', Script::Latin),
    ('a', 'z', Script::Latin),
    ('\u{00AA}', '\u{00AA}', Script::Latin),
    ('\u{00BA}', '\u{00BA}', Script::Latin),
    ('\u{00C0}', '\u{024F}', Script::Latin),
    ('\u{0250}', '\u{02AF}', Script::Latin),
    ('\u{02B0}', '\u{036F}', Script::Common),
    ('\u{0370}', '\u{03FF}', Script::Greek),
    ('\u{0400}', '\u{052F}', Script::Cyrillic),
    ('\u{0530}', '\u{058F}', Script::Armenian),
    ('\u{0590}', '\u{05FF}', Script::Hebrew),
    ('\u{0600}', '\u{06FF}', Script::Arabic),
    ('\u{0700}', '\u{074F}', Script::Syriac),
    ('\u{0750}', '\u{077F}', Script::Arabic),
    ('\u{0780}', '\u{07BF}', Script::Thaana),
    ('\u{07C0}', '\u{07FF}', Script::Nko),
    ('\u{0870}', '\u{08FF}', Script::Arabic),
    ('\u{0900}', '\u{097F}', Script::Devanagari),
    ('\u{0980}', '\u{09FF}', Script::Bengali),
    ('\u{0B80}', '\u{0BFF}', Script::Tamil),
    ('\u{0E00}', '\u{0E7F}', Script::Thai),
    ('\u{0E80}', '\u{0EFF}', Script::Lao),
    ('\u{1000}', '\u{109F}', Script::Myanmar),
    ('\u{10A0}', '\u{10FF}', Script::Georgian),
    ('\u{1100}', '\u{11FF}', Script::Hangul),
    ('\u{1780}', '\u{17FF}', Script::Khmer),
    ('\u{1C80}', '\u{1C8F}', Script::Cyrillic),
    ('\u{1C90}', '\u{1CBF}', Script::Georgian),
    ('\u{1DC0}', '\u{1DFF}', Script::Common),
    ('\u{1E00}', '\u{1EFF}', Script::Latin),
    ('\u{1F00}', '\u{1FFF}', Script::Greek),
    ('\u{2C60}', '\u{2C7F}', Script::Latin),
    ('\u{2D00}', '\u{2D2F}', Script::Georgian),
    ('\u{2DE0}', '\u{2DFF}', Script::Cyrillic),
    ('\u{3000}', '\u{303F}', Script::Han),
    ('\u{3040}', '\u{312F}', Script::Han),
    ('\u{3130}', '\u{318F}', Script::Hangul),
    ('\u{3190}', '\u{31FF}', Script::Han),
    ('\u{3400}', '\u{4DBF}', Script::Han),
    ('\u{4E00}', '\u{9FFF}', Script::Han),
    ('\u{A640}', '\u{A69F}', Script::Cyrillic),
    ('\u{A720}', '\u{A7FF}', Script::Latin),
    ('\u{A8E0}', '\u{A8FF}', Script::Devanagari),
    ('\u{A960}', '\u{A97F}', Script::Hangul),
    ('\u{AB30}', '\u{AB6F}', Script::Latin),
    ('\u{AC00}', '\u{D7FF}', Script::Hangul),
    ('\u{F900}', '\u{FAFF}', Script::Han),
    ('\u{FB1D}', '\u{FB4F}', Script::Hebrew),
    ('\u{FB50}', '\u{FDFF}', Script::Arabic),
    ('\u{FE70}', '\u{FEFF}', Script::Arabic),
    ('\u{FF21}', '\u{FF3A}', Script::Latin),
    ('\u{FF41}', '\u{FF5A}', Script::Latin),
    ('\u{FF66}', '\u{FF9F}', Script::Han),
    ('\u{FFA0}', '\u{FFDC}', Script::Hangul),
    ('\u{20000}', '\u{3134F}', Script::Han),
];

/// The script of the letter `c`.
fn script(c: char) -> Script {
    BLOCKS
        .iter()
        .find(|(first, last, _)| (*first..=*last).contains(&c))
        .map_or(Script::Other, |(_, _, script)| *script)
}

/// The joiners that stay inside a word of any script but never start or end one: UAX #29 never breaks a word at
/// them (ZWNJ, ZWJ), and Persian writes ZWNJ inside words.
const JOINERS: &[char] = &['\u{200C}', '\u{200D}'];

/// True when `c` is a combining mark or a modifier letter, which join a word of any script.
#[must_use]
pub fn joins_any(c: char) -> bool {
    script(c) == Script::Common
}

/// True when `c` is a joiner, which belongs to a word only between its letters.
#[must_use]
pub fn joins_inside(c: char) -> bool {
    JOINERS.contains(&c)
}

/// True when every letter of `word` is of one script, not counting the ones that join any script.
#[must_use]
pub fn single(word: &str) -> bool {
    let mut scripts = word
        .chars()
        .filter(|c| c.is_alphabetic())
        .map(script)
        .filter(|s| *s != Script::Common);
    let first = scripts.next();
    scripts.all(|s| Some(s) == first)
}

/// The script of `word`: that of its first letter not joining any script; `None` when it has none.
#[must_use]
pub fn of(word: &str) -> Option<Script> {
    word.chars()
        .filter(|c| c.is_alphabetic())
        .map(script)
        .find(|s| *s != Script::Common)
}

/// The commonest script in `scripts` when it and the scripts it holds (Hanja for Hangul) have at least `share` of the
/// words with a script; a tie goes to the one holding more, then to the first row. The builder and `Pack` share it.
#[must_use]
pub fn main(scripts: impl IntoIterator<Item = Option<Script>>, share: f64) -> Option<Script> {
    let mut counts = [0_u32; FACTS.len()];
    for script in scripts.into_iter().flatten() {
        let row = FACTS.iter().position(|(s, _)| *s == script);
        if let Some(n) = row.and_then(|at| counts.get_mut(at)) {
            *n = n.saturating_add(1);
        }
    }
    let rows = || FACTS.iter().map(|row| row.0).zip(counts);
    let sum = |all: u32, (_, n): (Script, u32)| all.saturating_add(n);
    let held = |top: Script| rows().filter(|(s, _)| top.holds(*s)).fold(0, sum);
    let total = rows().fold(0, sum);
    let ranked = rows().enumerate();
    let (_, (top, _)) = ranked.max_by_key(|&(at, (s, n))| (n, held(s), Reverse(at)))?;
    (total > 0 && f64::from(held(top)) >= share * f64::from(total)).then_some(top)
}

#[cfg(test)]
mod tests;
