//! Which script a letter belongs to, one table for the pack builder and the tracker: a word mixing scripts is never a
//! word, and a guess in a right-to-left script reads right to left.

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
    /// Arabic, with its presentation forms.
    Arabic,
    /// Devanagari.
    Devanagari,
    /// Bengali.
    Bengali,
    /// Tamil.
    Tamil,
    /// Thai.
    Thai,
    /// Georgian.
    Georgian,
    /// Han, kana and Hangul: Japanese mixes kanji and kana in one word, so they count as one.
    Cjk,
    /// Any letter of a script this table does not list.
    Other,
}

impl Script {
    /// True for the scripts of this table written right to left, after the Unicode bidirectional classes of their
    /// letters (R and AL).
    #[must_use]
    pub fn rtl(self) -> bool {
        matches!(self, Self::Hebrew | Self::Arabic)
    }
}

/// The Unicode blocks of each script, from the Unicode block list; a letter outside them is `Other`.
const BLOCKS: &[(char, char, Script)] = &[
    ('A', 'Z', Script::Latin),
    ('a', 'z', Script::Latin),
    ('\u{00C0}', '\u{024F}', Script::Latin),
    ('\u{0250}', '\u{02AF}', Script::Latin),
    ('\u{02B0}', '\u{036F}', Script::Common),
    ('\u{0370}', '\u{03FF}', Script::Greek),
    ('\u{0400}', '\u{052F}', Script::Cyrillic),
    ('\u{0530}', '\u{058F}', Script::Armenian),
    ('\u{0590}', '\u{05FF}', Script::Hebrew),
    ('\u{0600}', '\u{06FF}', Script::Arabic),
    ('\u{0750}', '\u{077F}', Script::Arabic),
    ('\u{0870}', '\u{08FF}', Script::Arabic),
    ('\u{0900}', '\u{097F}', Script::Devanagari),
    ('\u{0980}', '\u{09FF}', Script::Bengali),
    ('\u{0B80}', '\u{0BFF}', Script::Tamil),
    ('\u{0E00}', '\u{0E7F}', Script::Thai),
    ('\u{10A0}', '\u{10FF}', Script::Georgian),
    ('\u{1100}', '\u{11FF}', Script::Cjk),
    ('\u{1C80}', '\u{1C8F}', Script::Cyrillic),
    ('\u{1DC0}', '\u{1DFF}', Script::Common),
    ('\u{1E00}', '\u{1EFF}', Script::Latin),
    ('\u{1F00}', '\u{1FFF}', Script::Greek),
    ('\u{2C60}', '\u{2C7F}', Script::Latin),
    ('\u{2DE0}', '\u{2DFF}', Script::Cyrillic),
    ('\u{3000}', '\u{303F}', Script::Cjk),
    ('\u{3040}', '\u{31FF}', Script::Cjk),
    ('\u{3400}', '\u{4DBF}', Script::Cjk),
    ('\u{4E00}', '\u{9FFF}', Script::Cjk),
    ('\u{A640}', '\u{A69F}', Script::Cyrillic),
    ('\u{A720}', '\u{A7FF}', Script::Latin),
    ('\u{A8E0}', '\u{A8FF}', Script::Devanagari),
    ('\u{AB30}', '\u{AB6F}', Script::Latin),
    ('\u{AC00}', '\u{D7AF}', Script::Cjk),
    ('\u{F900}', '\u{FAFF}', Script::Cjk),
    ('\u{FB1D}', '\u{FB4F}', Script::Hebrew),
    ('\u{FB50}', '\u{FDFF}', Script::Arabic),
    ('\u{FE70}', '\u{FEFF}', Script::Arabic),
    ('\u{FF21}', '\u{FF3A}', Script::Latin),
    ('\u{FF41}', '\u{FF5A}', Script::Latin),
    ('\u{FF66}', '\u{FFDC}', Script::Cjk),
    ('\u{20000}', '\u{3134F}', Script::Cjk),
];

/// The script of the letter `c`.
fn script(c: char) -> Script {
    BLOCKS
        .iter()
        .find(|(first, last, _)| (*first..=*last).contains(&c))
        .map_or(Script::Other, |(_, _, script)| *script)
}

/// True when `c` is a combining mark or a modifier letter, which join a word of any script.
#[must_use]
pub fn joins_any(c: char) -> bool {
    script(c) == Script::Common
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn marks_join_any_script_and_words_of_one_script_are_single() {
        assert!(joins_any('\u{0301}'));
        assert!(!joins_any('e'));
        for word in ["e\u{0301}cole", "كتابٌ", "食べる", "l'homme", "x2"] {
            assert!(single(word), "{word}");
        }
        assert!(!single("abcعربي"));
        assert!(!single("hello\u{043C}"));
    }

    #[test]
    fn a_word_takes_its_first_script_and_only_hebrew_and_arabic_read_right_to_left() {
        assert_eq!(of("\u{0301}é"), Some(Script::Latin));
        assert_eq!(of("123"), None);
        assert!(of("أن").is_some_and(Script::rtl));
        assert!(of("שלום").is_some_and(Script::rtl));
        assert!(!of("école").is_some_and(Script::rtl));
        assert!(!of("слово").is_some_and(Script::rtl));
    }

    #[test]
    fn the_blocks_are_ordered_and_apart() {
        let pairs = BLOCKS.windows(2);
        assert!(pairs.into_iter().all(|p| p[0].1 < p[1].0));
    }
}
