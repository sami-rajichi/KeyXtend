//! Tests of the word lists on tiny fixtures built in memory; nothing is downloaded.

mod budget;
mod build;
mod chains;
mod count;
mod curl;
mod fetch;
mod manifest;
mod merge;
mod output;
mod script;
mod text;
mod wordfreq;

use super::cands::candidates;
use super::clean;
use super::dict::Dict;
use super::fixture::{self, AFF, dic};
use super::list::{self, Word, WordList};
use super::manifest::Plan;
use super::report::report;
use super::text::TextCounts;
use super::wordfreq::{Entry, freq};

fn entries(words: &[(&str, u32)]) -> Vec<Entry> {
    let entry = |(word, bucket): &(&str, u32)| Entry {
        word: (*word).to_string(),
        bucket: *bucket,
    };
    words.iter().map(entry).collect()
}

fn plan(max_words: usize) -> Plan<'static> {
    Plan {
        iso: "fr".to_string(),
        list: String::new(),
        dict: None,
        max_words,
        words_budget_kb: 1_000,
        text: None,
    }
}

fn dict(aff: &str, words: &[&str]) -> Dict {
    Dict::parse(aff, &dic(words), fixture::rules().apostrophe).unwrap()
}

fn build_with(words: &[(&str, u32)], dict: Option<&Dict>, text: Option<&TextCounts>) -> WordList {
    let rules = fixture::rules();
    let cands = candidates(entries(words), &plan(100), dict, &rules);
    list::finish(cands, text, &rules).unwrap()
}

fn build(words: &[(&str, u32)], dict_words: &[&str]) -> WordList {
    build_with(words, Some(&dict(AFF, dict_words)), None)
}

fn spellings(list: &WordList) -> Vec<&str> {
    list.words.iter().map(|w| w.spelling.as_str()).collect()
}

fn word(spelling: &str, bucket: u32, capital: bool) -> Word {
    Word {
        spelling: spelling.to_string(),
        freq: freq(bucket),
        capital,
    }
}

#[test]
fn the_dictionary_drops_ete_into_ete_and_keeps_a_and_a_grave() {
    let words = [("ete", 300), ("été", 310), ("a", 200), ("à", 250)];
    let list = build(&words, &["été", "a", "à"]);
    let expected = [
        word("a", 200, false),
        word("à", 250, false),
        word("été", 300, false),
    ];
    assert_eq!(list.words, expected, "ete gives its frequency to été");
    assert_eq!(list.facts.merged.first, ["ete → été"]);
}

#[test]
fn capitals_return_with_their_flag() {
    let words = [("london", 400), ("i", 150), ("the", 130)];
    let list = build(&words, &["I", "London", "the"]);
    let expected = [
        word("the", 130, false),
        word("I", 150, true),
        word("London", 400, true),
    ];
    assert_eq!(list.words, expected);
}

#[test]
fn the_dictionary_lists_each_form_it_accepts_its_own_first() {
    let dict = dict(
        AFF,
        &["été", "London", "i", "I", "l", "l'", "jusqu'", "أنا"],
    );
    let forms = |w: &str| dict.forms(w);
    assert_eq!(forms("été"), ["été"]);
    assert_eq!(forms("london"), ["London"]);
    assert_eq!(forms("i"), ["i", "I"]);
    assert_eq!(forms("l"), ["l", "l'"]);
    assert_eq!(forms("jusqu"), ["jusqu'"]);
    assert_eq!(forms("أنا"), ["أنا"]);
    assert!(forms("ete").is_empty() && forms("انا").is_empty());
}

#[test]
fn an_elided_form_comes_from_the_dictionary() {
    let list = build(&[("jusqu", 300), ("homme", 310)], &["jusqu'", "homme"]);
    assert_eq!(spellings(&list), ["jusqu'", "homme"]);
    assert_eq!(list.facts.elisions.first, ["jusqu → jusqu'"]);
}

#[test]
fn a_join_takes_the_frequency_of_its_rarer_piece() {
    let words = [("aujourd", 500), ("hui", 520), ("homme", 300)];
    let list = build(&words, &["aujourd'hui", "homme"]);
    assert_eq!(
        list.words,
        [word("homme", 300, false), word("aujourd'hui", 520, false)]
    );
    assert_eq!(list.facts.joins.first, ["aujourd'hui"]);
    let gaps = list.coverage.map(|c| c.first);
    assert_eq!(
        gaps,
        Some(vec!["hui".to_string()]),
        "a joined piece is no gap"
    );
}

#[test]
fn digits_web_addresses_emails_and_mixed_scripts_are_dropped() {
    let rules = fixture::rules();
    let long = "a".repeat(129);
    let dropped = [
        "abc123",
        "www.x.org",
        "a@b.c",
        "a/b",
        "abcабв",
        "سلامabc",
        "٣٤",
        "♥",
        "",
        &long,
    ];
    for word in dropped {
        assert!(!clean::keeps(word, &rules.drop_chars), "{word} is kept");
    }
    let fits = "a".repeat(128);
    let kept = [
        "été",
        "don't",
        "l'",
        "食べる",
        "п\u{2bc}ять",
        "أنا",
        "ἀγάπη",
        &fits,
    ];
    for word in kept {
        assert!(clean::keeps(word, &rules.drop_chars), "{word} is dropped");
    }
}

#[test]
fn a_language_without_a_dictionary_keeps_its_clean_words_up_to_max_words() {
    let words = entries(&[("der", 120), ("abc123", 121), ("die", 122), ("das", 123)]);
    let rules = fixture::rules();
    let list = list::finish(candidates(words, &plan(2), None, &rules), None, &rules).unwrap();
    assert_eq!(spellings(&list), ["der", "die"]);
    assert_eq!(list.read, 4);
    assert!(list.coverage.is_none());
}

#[test]
fn words_of_one_frequency_keep_wordfreq_order() {
    let list = build_with(&[("c", 199), ("b", 201), ("a", 202)], None, None);
    assert_eq!(freq(201), freq(202));
    assert_eq!(spellings(&list), ["c", "b", "a"]);
}

#[test]
fn coverage_counts_the_rejected_among_the_most_frequent() {
    let list = build(&[("zz", 100), ("a", 110), ("yy", 120), ("xx", 130)], &["a"]);
    let coverage = list.coverage.unwrap();
    assert_eq!((coverage.rejected, coverage.top), (2, 3));
    assert_eq!(coverage.first, ["zz", "yy"]);
}

#[test]
fn the_report_prints_one_line_per_rule() {
    let list = build(&[("انا", 200), ("أنا", 300), ("zz", 310)], &["أنا"]);
    let lines = report("ar", &list);
    let starts = [
        "ar: 1 word kept (0 capitalised",
        "coverage ar: the dictionary rejected 1 of the 3 most frequent words; first 1: zz",
        "merged ar: 1 word; first 1: انا → أنا",
        "text ar: none, so twins, pieces and stems are skipped",
    ];
    assert_eq!(lines.len(), starts.len(), "{lines:?}");
    for (line, start) in lines.iter().zip(starts) {
        assert!(line.starts_with(start), "{line}");
    }
}
