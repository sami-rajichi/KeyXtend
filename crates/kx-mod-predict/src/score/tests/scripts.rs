//! Casing by language and word spaces by script, through the tracker and the scorer: Turkish capitals, Korean gaps,
//! and no chips yet for Chinese and Japanese.

use kx_module_api::ports::Piece;

use super::*;
use crate::track::{TrackConfig, Tracker};

/// Korean, which spaces its words.
fn korean() -> Vec<u8> {
    pack(
        "ko",
        &[("안녕하세요", 180), ("안녕", 170), ("감사합니다", 160)],
        &[],
    )
}

/// Chinese, which does not.
fn chinese() -> Vec<u8> {
    pack("zh", &[("你好", 190), ("我们", 180)], &[])
}

/// Japanese, kana and kanji, which does not either.
fn japanese() -> Vec<u8> {
    pack("ja", &[("こんにちは", 190), ("日本語", 180)], &[])
}

/// Turkish, with dotted and dotless i.
fn turkish() -> Vec<u8> {
    pack(
        "tr",
        &[("istanbul", 180), ("ısı", 170), ("ıslak", 160)],
        &[],
    )
}

/// Each guess after typing `text` with `bytes`'s pack in `mode`, with the text its plan types.
fn typed(bytes: &[u8], text: &str, mode: TypingMode) -> Vec<(String, String)> {
    let pack = Pack::parse(bytes).unwrap();
    let mut tracker = Tracker::new(TrackConfig {
        context_words: 2,
        max_word: 48,
    });
    let known = |w: &str| knows(Some(&pack), w);
    tracker.typed(&Piece::Text(Redacted::new(text.to_owned())), &known);
    let Some(query) = tracker.show(&known).words else {
        return Vec::new();
    };
    let found = guesses(Some(&pack), &NoLearned, &query, mode, LIMIT, &CFG);
    let plan = |t: &str| query.plan(t).unwrap().text.expose().clone();
    found
        .iter()
        .map(|g| (g.text.expose().clone(), plan(g.text.expose())))
        .collect()
}

/// The texts of `found`.
fn guessed(found: &[(String, String)]) -> Vec<&str> {
    found.iter().map(|f| f.0.as_str()).collect()
}

#[test]
fn korean_gets_chips_that_type_a_space_after_the_word() {
    let found = typed(&korean(), "안", PLAIN);
    let first = ("안녕하세요".to_owned(), "안녕하세요 ".to_owned());
    assert_eq!(found.first(), Some(&first));
    assert!(!typed(&korean(), "", PLAIN).is_empty());
}

#[test]
fn chinese_and_japanese_get_no_chips_until_their_input_mode() {
    for bytes in [chinese(), japanese()] {
        assert!(typed(&bytes, "", PLAIN).is_empty());
        let pack = Pack::parse(&bytes).unwrap();
        assert!(guesses(Some(&pack), &NoLearned, &ask(&[], ""), PLAIN, LIMIT, &CFG).is_empty());
    }
    assert!(typed(&chinese(), "你", PLAIN).is_empty());
    assert!(typed(&japanese(), "こ", PLAIN).is_empty());
}

#[test]
fn a_capitalised_turkish_i_keeps_its_dot_and_a_dotless_capital_finds_dotless_words() {
    assert_eq!(guessed(&typed(&turkish(), "İs", PLAIN)), ["İstanbul"]);
    assert_eq!(guessed(&typed(&turkish(), "Is", PLAIN)), ["Isı", "Islak"]);
    let shift = TypingMode {
        shift: true,
        ..PLAIN
    };
    let next = typed(&turkish(), "", shift);
    assert_eq!(guessed(&next).first(), Some(&"İstanbul"));
}

#[test]
fn a_turkish_context_typed_in_capitals_finds_its_twin_by_turkish_casing() {
    let words = [
        ("Irmak", 190),
        ("ırmak", 150),
        ("Hanım", 120),
        ("kenarı", 110),
    ];
    let chains: [Chain<'_>; 2] = [(&["ırmak"], "kenarı", 240), (&["Irmak"], "Hanım", 240)];
    let next = typed(&pack("tr", &words, &chains), "IRMAK ", PLAIN);
    assert_eq!(
        guessed(&next).first(),
        Some(&"kenarı"),
        "IRMAK lowers to ırmak"
    );
}

#[test]
fn a_learned_turkish_word_is_found_by_turkish_folding() {
    let learned = Fake::default().lang("tr").uses(100).word("ılık", 50);
    let bytes = turkish();
    let pack = Pack::parse(&bytes).unwrap();
    let found = guesses(Some(&pack), &learned, &ask(&[], "ıl"), PLAIN, LIMIT, &CFG);
    assert_eq!(texts(&found), ["ılık"]);
}
