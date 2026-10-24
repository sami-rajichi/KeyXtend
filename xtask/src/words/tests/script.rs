//! The main-script filter: a list mostly in one script drops the words of the others.

use super::{build_with, spellings};

/// `n` Latin words, each its own bucket.
fn latin(n: u32) -> Vec<(String, u32)> {
    (0..n)
        .map(|i| {
            (
                format!("w{}", char::from(b'a' + u8::try_from(i).unwrap())),
                100 + i,
            )
        })
        .collect()
}

fn run(words: &[(String, u32)]) -> super::WordList {
    let refs: Vec<(&str, u32)> = words.iter().map(|(w, b)| (w.as_str(), *b)).collect();
    build_with(&refs, None, None)
}

#[test]
fn a_list_mostly_in_one_script_drops_the_other_scripts() {
    let mut words = latin(19);
    words.push(("слово".to_string(), 50));
    let list = run(&words);
    assert_eq!(list.words.len(), 19);
    assert_eq!(list.facts.script.first, ["слово"]);
}

#[test]
fn a_mixed_list_stays_whole() {
    let mut words = latin(8);
    words.extend([("слово", 50), ("мир", 51)].map(|(w, b)| (w.to_string(), b)));
    let list = run(&words);
    assert_eq!(list.words.len(), 10, "Latin holds 0.8, under the share");
    assert_eq!(list.facts.script.count, 0);
}

#[test]
fn kanji_kana_and_hangul_count_as_one_script() {
    let cjk = [
        "食べる",
        "ひらがな",
        "漢字",
        "カタカナ",
        "한국어",
        "日本",
        "東京",
        "すし",
        "ラーメン",
    ];
    let mut words: Vec<(String, u32)> = (100..).zip(cjk).map(|(b, w)| (w.to_string(), b)).collect();
    words.push(("sushi".to_string(), 99));
    let list = run(&words);
    assert_eq!(spellings(&list), cjk, "nine of ten words share the script");
    assert_eq!(list.facts.script.first, ["sushi"]);
}
