//! The script table: marks join any script, words of one script are single, and each script's row gives its
//! direction and the gap between its words.

use super::*;

/// Every script, so a test can check each has a row.
const ALL: [Script; 18] = [
    Script::Common,
    Script::Latin,
    Script::Greek,
    Script::Cyrillic,
    Script::Armenian,
    Script::Hebrew,
    Script::Arabic,
    Script::Devanagari,
    Script::Bengali,
    Script::Tamil,
    Script::Thai,
    Script::Lao,
    Script::Myanmar,
    Script::Khmer,
    Script::Georgian,
    Script::Han,
    Script::Hangul,
    Script::Other,
];

#[test]
fn only_zwnj_and_zwj_join_inside_a_word() {
    assert!(joins_inside('\u{200C}') && joins_inside('\u{200D}'));
    for c in ['\u{200B}', '\u{200E}', '\u{2060}', 'a', ' ', '\''] {
        assert!(!joins_inside(c), "{c:?}");
    }
    assert!(single("کتاب\u{200C}ها"), "a joiner is no letter");
}

#[test]
fn marks_join_any_script_and_words_of_one_script_are_single() {
    assert!(joins_any('\u{0301}'));
    assert!(!joins_any('e'));
    for word in ["e\u{0301}cole", "كتابٌ", "食べる", "한국어", "l'homme", "x2"] {
        assert!(single(word), "{word}");
    }
    assert!(!single("abcعربي"));
    assert!(!single("hello\u{043C}"));
    assert!(!single("韓國어"), "Hanja beside Hangul");
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
fn korean_spaces_its_words_while_chinese_japanese_and_thai_do_not() {
    let gap = |word: &str| of(word).map(Script::gap);
    assert_eq!(gap("한국어"), Some(" "));
    assert_eq!(gap("أن"), Some(" "));
    assert_eq!(gap("école"), Some(" "));
    assert!(of("한국어").is_some_and(Script::spaced) && !of("你好").is_some_and(Script::spaced));
    for unspaced in [
        "你好",
        "ひらがな",
        "カタカナ",
        "ภาษาไทย",
        "ພາສາລາວ",
        "ខ្មែរ",
        "မြန်မာ",
    ] {
        assert_eq!(gap(unspaced), Some(""), "{unspaced}");
    }
}

#[test]
fn every_script_has_exactly_one_row() {
    for script in ALL {
        let rows = FACTS.iter().filter(|(s, _)| *s == script).count();
        assert_eq!(rows, 1, "{script:?}");
    }
    assert_eq!(FACTS.len(), ALL.len());
}

#[test]
fn the_blocks_are_ordered_and_apart() {
    let pairs = BLOCKS.windows(2);
    assert!(pairs.into_iter().all(|p| p[0].1 < p[1].0));
}

#[test]
fn every_script_holds_itself_and_hangul_also_holds_han_as_kore_does() {
    assert!(ALL.iter().all(|s| s.holds(*s)));
    assert!(Script::Hangul.holds(Script::Han));
    assert!(!Script::Han.holds(Script::Hangul), "Chinese drops Hangul");
    assert!(!Script::Latin.holds(Script::Greek));
}

#[test]
fn the_main_script_holds_the_share_of_the_words_with_a_script() {
    let scripts = |words: &[&str]| words.iter().map(|w| of(w)).collect::<Vec<_>>();
    let latin = scripts(&["one", "two", "три", "123"]);
    assert_eq!(
        main(latin.clone(), 0.6),
        Some(Script::Latin),
        "the digits aside"
    );
    assert_eq!(main(latin, 0.7), None);
    let korean = scripts(&["사랑", "학교", "친구", "漢字"]);
    assert_eq!(
        main(korean, 1.0),
        Some(Script::Hangul),
        "Hanja counts as Korean"
    );
    let chinese = scripts(&["漢字", "日本", "東京", "사랑"]);
    assert_eq!(main(chinese.clone(), 1.0), None, "Hangul is not Chinese");
    assert_eq!(main(chinese, 0.75), Some(Script::Han));
    assert_eq!(main(scripts(&["123", ""]), 0.0), None, "no letter");
}

#[test]
fn a_tie_goes_to_the_script_holding_more_then_to_the_first_row() {
    let scripts = |words: &[&str]| words.iter().map(|w| of(w)).collect::<Vec<_>>();
    for words in [["one", "три"], ["три", "one"]] {
        assert_eq!(main(scripts(&words), 0.5), Some(Script::Latin), "{words:?}");
    }
    for words in [["사랑", "漢字"], ["漢字", "사랑"]] {
        assert_eq!(
            main(scripts(&words), 1.0),
            Some(Script::Hangul),
            "{words:?}"
        );
    }
}
