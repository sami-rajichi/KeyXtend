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

/// Arabic words whose keys share the two-letter, four-byte prefix كت; its third letters cross from lead byte D8 to D9.
const ARABIC: [(&str, u8); 9] = [
    ("قلم", 190),
    ("كبير", 180),
    ("كتاب", 170),
    ("كتابة", 160),
    ("كتب", 150),
    ("كتف", 140),
    ("كثير", 130),
    ("كلمة", 120),
    ("لون", 110),
];
/// Prefixes looked up in `ARABIC`, each with the words it finds: hits, and misses before, between and past the keys.
/// U+063F is the last letter under lead byte D8, so its successor ends in a byte no letter takes.
const ARABIC_SPANS: [(&str, usize); 18] = [
    ("", 9),
    ("ف", 0),
    ("ق", 1),
    ("ك", 7),
    ("كت", 4),
    ("كتا", 2),
    ("كتاب", 2),
    ("كتابة", 1),
    ("كتابي", 0),
    ("كتب", 1),
    ("كتت", 0),
    ("كتف", 1),
    ("كتق", 0),
    ("كث", 1),
    ("كج", 0),
    ("ك\u{063F}", 0),
    ("ل", 1),
    ("م", 0),
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
fn the_script_is_the_builders_main_script_over_every_spelling() {
    let ko = list("ko", &[("漢字", 200), ("사랑", 150), ("학교", 100)]);
    assert_eq!(
        Pack::parse(&ko).unwrap().script(),
        Some(Script::Hangul),
        "Hanja counts as Korean, even first"
    );
    let zh = list("zh", &[("你好", 200), ("我们", 150)]);
    assert_eq!(Pack::parse(&zh).unwrap().script(), Some(Script::Han));
    let ja = list(
        "ja",
        &[("ひらがな", 200), ("日本語", 150), ("カタカナ", 100)],
    );
    assert_eq!(
        Pack::parse(&ja).unwrap().script(),
        Some(Script::Han),
        "kana and kanji count as one"
    );
    let mixed = list("en", &[("hello", 200), ("world", 150), ("你好", 100)]);
    assert_eq!(
        Pack::parse(&mixed).unwrap().script(),
        None,
        "a list the builder left mixed"
    );
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

#[test]
fn a_prefix_span_holds_its_completions_over_multi_byte_keys() {
    let bytes = list("ar", &ARABIC);
    let pack = Pack::parse(&bytes).unwrap();
    for (prefix, want) in ARABIC_SPANS {
        let span: Vec<WordId> = pack.span(prefix).collect();
        let found: Vec<WordId> = pack.completions(prefix).collect();
        assert_eq!((span.len(), &span), (want, &found), "{prefix}");
    }
}
