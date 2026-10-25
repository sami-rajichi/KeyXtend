//! The lookups over the lexicon's keys and the frequencies: a prefix's id range, and the pack's main script.

use super::super::*;
use crate::script::Script;

/// Followers kept per context in these tests.
const FOLLOWERS: u8 = 3;
/// Prefixes looked up: empty, folded Latin at each length, past every key, unfolded, and the last code point.
const PREFIXES: [&str; 11] = [
    "",
    "e",
    "ec",
    "eco",
    "ecole",
    "ecolex",
    "f",
    "zz",
    "zzz",
    "é",
    "\u{10FFFF}",
];

/// A pack of `words` in `iso`, without chains.
fn list(iso: &str, words: &[(&str, u8)]) -> Vec<u8> {
    let flags = Flags {
        checked: false,
        chains: false,
    };
    let mut writer = PackWriter::new(iso, flags, FOLLOWERS).unwrap();
    for (word, freq) in words {
        writer.add_word(word, *freq).unwrap();
    }
    writer.finish().unwrap()
}

#[test]
fn the_script_is_the_most_frequent_words() {
    let zh = list("zh", &[("hello", 100), ("你好", 200), ("我们", 150)]);
    assert_eq!(Pack::parse(&zh).unwrap().script(), Some(Script::Han));
    let en = list("en", &[("hello", 200), ("你好", 100)]);
    assert_eq!(Pack::parse(&en).unwrap().script(), Some(Script::Latin));
}

#[test]
fn a_prefix_span_holds_the_ids_its_completions_give() {
    let words = [
        ("école", 150),
        ("Ecole", 10),
        ("eco", 90),
        ("ed", 80),
        ("a", 70),
        ("été", 60),
        ("zz", 50),
        ("ÿ", 40),
    ];
    let bytes = list("fr", &words);
    let pack = Pack::parse(&bytes).unwrap();
    for prefix in PREFIXES {
        let span: Vec<WordId> = pack.span(prefix).collect();
        let found: Vec<WordId> = pack.completions(prefix).collect();
        assert_eq!(span, found, "{prefix}");
    }
}
