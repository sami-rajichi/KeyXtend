//! Casing by language and word spaces by script, through the tracker and the scorer: Turkish capitals, Korean gaps,
//! and no chips yet for Chinese and Japanese.

use super::*;

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

/// Each guess after typing `text` with `bytes`'s pack in `mode`, with the whole text its chip leaves, gap included.
fn typed(bytes: &[u8], text: &str, mode: TypingMode) -> Vec<(String, String)> {
    let pack = Pack::parse(bytes).unwrap();
    let Some(query) = typed_query(&pack, text) else {
        return Vec::new();
    };
    let found = guesses(
        pack.iso(),
        Some(&pack),
        &NoLearned,
        &query,
        mode,
        LIMIT,
        &CFG,
    );
    let plan = |t: &str| query.plan(t).unwrap().chip().into_inner();
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
    let found = typed(&korean(), "안", TypingMode::default());
    let first = ("안녕하세요".to_owned(), "안녕하세요 ".to_owned());
    assert_eq!(found.first(), Some(&first));
    assert!(!typed(&korean(), "", TypingMode::default()).is_empty());
}

#[test]
fn chinese_and_japanese_get_no_chips_until_their_input_mode() {
    for bytes in [chinese(), japanese()] {
        assert!(typed(&bytes, "", TypingMode::default()).is_empty());
        let pack = Pack::parse(&bytes).unwrap();
        assert!(
            guesses(
                pack.iso(),
                Some(&pack),
                &NoLearned,
                &ask(&[], ""),
                TypingMode::default(),
                LIMIT,
                &CFG
            )
            .is_empty()
        );
    }
    assert!(typed(&chinese(), "你", TypingMode::default()).is_empty());
    assert!(typed(&japanese(), "こ", TypingMode::default()).is_empty());
}

#[test]
fn a_capitalised_turkish_i_keeps_its_dot_and_a_dotless_capital_finds_dotless_words() {
    assert_eq!(
        guessed(&typed(&turkish(), "İs", TypingMode::default())),
        ["İstanbul"]
    );
    assert_eq!(
        guessed(&typed(&turkish(), "Is", TypingMode::default())),
        ["Isı", "Islak"]
    );
    let shift = TypingMode {
        shift: true,
        ..TypingMode::default()
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
    let next = typed(
        &pack("tr", &words, &chains),
        "IRMAK ",
        TypingMode::default(),
    );
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
    let found = guesses(
        pack.iso(),
        Some(&pack),
        &learned,
        &ask(&[], "ıl"),
        TypingMode::default(),
        LIMIT,
        &CFG,
    );
    assert_eq!(texts(&found), ["ılık"]);
}

#[test]
fn turkish_with_no_pack_folds_by_turkish_casing() {
    let learned = Fake::default()
        .lang("tr")
        .uses(100)
        .word("ılık", 50)
        .word("iyi", 50);
    let tr = |prefix: &str| {
        let found = guesses(
            "tr",
            None,
            &learned,
            &ask(&[], prefix),
            TypingMode::default(),
            LIMIT,
            &CFG,
        );
        texts(&found)
    };
    assert_eq!(tr("ıl"), ["ılık"]);
    assert_eq!(tr("ı"), ["ılık"], "iyi starts with a dotted i");
}

#[test]
fn a_pack_of_another_language_is_ignored() {
    let bytes = turkish();
    let pack = Pack::parse(&bytes).unwrap();
    let found = guesses(
        "en",
        Some(&pack),
        &NoLearned,
        &ask(&[], "is"),
        TypingMode::default(),
        LIMIT,
        &CFG,
    );
    assert!(found.is_empty());
}

#[test]
fn chinese_and_japanese_read_no_list() {
    for bytes in [chinese(), japanese()] {
        let pack = Pack::parse(&bytes).unwrap();
        let learned = Fake::default().lang(pack.iso()).uses(10).word("hello", 5);
        for (context, prefix) in [(&[][..], ""), (&["hello"][..], "h")] {
            let query = ask(context, prefix);
            let found = guesses(
                pack.iso(),
                Some(&pack),
                &learned,
                &query,
                TypingMode::default(),
                LIMIT,
                &CFG,
            );
            assert!(found.is_empty());
        }
        assert_eq!(learned.reads(), 0, "{}", pack.iso());
    }
}
