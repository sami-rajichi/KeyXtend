//! Tests of the word lists on tiny fixtures built in memory; nothing is downloaded.

mod fetch;
mod manifest;
mod wordfreq;

use super::clean;
use super::dict::{Dict, Verdict};
use super::fixture::{self, AFF, dic};
use super::list::{self, Word, WordList};
use super::manifest::{Hamza, Plan};
use super::wordfreq::{Entry, freq};

/// The frequency byte of `bucket` with the project's 25 steps per Zipf.
fn byte(bucket: u32) -> u8 {
    freq(bucket, fixture::rules().steps_per_zipf)
}

fn entries(words: &[(&str, u32)]) -> Vec<Entry> {
    let entry = |(word, bucket): &(&str, u32)| Entry {
        word: (*word).to_string(),
        bucket: *bucket,
    };
    words.iter().map(entry).collect()
}

fn plan<'a>(elisions: &'a [String], hamza: Option<&'a Hamza>) -> Plan<'a> {
    Plan {
        list: String::new(),
        dict: None,
        max_words: 100,
        elisions,
        hamza,
    }
}

fn build(words: &[(&str, u32)], dict_words: &[&str], plan: &Plan<'_>) -> WordList {
    let dict = Dict::parse(AFF, &dic(dict_words)).unwrap();
    list::build(entries(words), plan, Some(&dict), &fixture::rules())
}

fn spellings(list: &WordList) -> Vec<&str> {
    list.words.iter().map(|w| w.spelling.as_str()).collect()
}

fn word(spelling: &str, bucket: u32, capital: bool) -> Word {
    Word {
        spelling: spelling.to_string(),
        freq: byte(bucket),
        capital,
    }
}

fn arabic() -> Hamza {
    Hamza {
        bare: 'ا',
        marked: vec!['أ', 'إ', 'آ'],
        max_gap_cb: 100,
    }
}

#[test]
fn the_dictionary_drops_ete_and_keeps_ete_a_and_a_grave() {
    let words = [("ete", 300), ("été", 310), ("a", 200), ("à", 250)];
    let list = build(&words, &["été", "a", "à"], &plan(&[], None));
    assert_eq!(spellings(&list), ["a", "à", "été"]);
}

#[test]
fn capitals_return_with_their_flag() {
    let words = [("london", 400), ("i", 150), ("the", 130)];
    let list = build(&words, &["I", "London", "the"], &plan(&[], None));
    let expected = [
        word("the", 130, false),
        word("I", 150, true),
        word("London", 400, true),
    ];
    assert_eq!(list.words, expected);
}

#[test]
fn the_dictionary_says_plain_capital_or_rejected() {
    let dict = Dict::parse(AFF, &dic(&["été", "London", "أنا"])).unwrap();
    assert_eq!(dict.check("été"), Verdict::Plain);
    assert_eq!(dict.check("london"), Verdict::Capital("London".to_string()));
    assert_eq!(dict.check("ete"), Verdict::Rejected);
    assert_eq!(dict.check("أنا"), Verdict::Plain);
    assert_eq!(dict.check("انا"), Verdict::Rejected);
}

#[test]
fn the_bare_alef_word_goes_and_its_frequency_moves_to_the_hamza_form() {
    let hamza = arabic();
    let words = [
        ("انا", 200),
        ("أنا", 300),
        ("ان", 150),
        ("أن", 250),
        ("إن", 260),
    ];
    let list = build(&words, &["أنا", "أن", "إن"], &plan(&[], Some(&hamza)));
    let expected = [
        word("أن", 150, false),
        word("أنا", 200, false),
        word("إن", 260, false),
    ];
    assert_eq!(
        list.words, expected,
        "ان goes to its most frequent hamza form only"
    );
    let merged = [("انا", "أنا"), ("ان", "أن")].map(|(a, b)| (a.to_string(), b.to_string()));
    assert_eq!(list.merged, merged);
    assert_eq!(
        list.coverage.map(|c| c.rejected),
        Some(0),
        "a merged word is not a gap"
    );
}

#[test]
fn a_bare_alef_word_the_dictionary_knows_stays_with_its_frequency() {
    let hamza = arabic();
    let words = [("كان", 200), ("ابن", 300), ("كأن", 400), ("أبن", 500)];
    let list = build(
        &words,
        &["كان", "ابن", "كأن", "أبن"],
        &plan(&[], Some(&hamza)),
    );
    let expected = [
        word("كان", 200, false),
        word("ابن", 300, false),
        word("كأن", 400, false),
        word("أبن", 500, false),
    ];
    assert_eq!(list.words, expected);
    assert!(list.merged.is_empty());
}

#[test]
fn a_rejected_bare_alef_word_without_a_hamza_form_stays_a_gap() {
    let hamza = arabic();
    let list = build(
        &[("اذا", 200), ("أنا", 300)],
        &["أنا", "إذن"],
        &plan(&[], Some(&hamza)),
    );
    assert_eq!(spellings(&list), ["أنا"]);
    assert_eq!(
        list.coverage.map(|c| c.first),
        Some(vec!["اذا".to_string()])
    );
}

#[test]
fn a_hamza_form_keeps_its_own_higher_frequency() {
    let hamza = arabic();
    let list = build(
        &[("أنا", 120), ("انا", 400)],
        &["أنا"],
        &plan(&[], Some(&hamza)),
    );
    assert_eq!(list.words, [word("أنا", 120, false)]);
}

#[test]
fn two_hamza_forms_both_stay() {
    let hamza = arabic();
    let list = build(
        &[("أن", 120), ("إن", 130)],
        &["أن", "إن"],
        &plan(&[], Some(&hamza)),
    );
    assert_eq!(spellings(&list), ["أن", "إن"]);
    assert!(list.merged.is_empty());
}

#[test]
fn french_elisions_take_back_their_apostrophe() {
    let elisions = ["l", "qu"].map(String::from);
    let words = [("l", 140), ("qu", 160), ("x", 170), ("homme", 300)];
    let list = build(&words, &["homme"], &plan(&elisions, None));
    assert_eq!(spellings(&list), ["l'", "qu'", "homme"]);
}

#[test]
fn a_spelling_seen_twice_keeps_its_highest_frequency() {
    let elisions = ["l".to_string()];
    let list = build(&[("l", 140), ("l'", 300)], &["l'"], &plan(&elisions, None));
    assert_eq!(list.words, [word("l'", 140, false)]);
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
    let plan = Plan {
        max_words: 2,
        ..plan(&[], None)
    };
    let words = entries(&[("der", 120), ("abc123", 121), ("die", 122), ("das", 123)]);
    let list = list::build(words, &plan, None, &fixture::rules());
    assert_eq!(spellings(&list), ["der", "die"]);
    assert_eq!(list.read, 4);
    assert!(list.coverage.is_none());
}

#[test]
fn coverage_counts_the_rejected_among_the_most_frequent() {
    let words = [("zz", 100), ("a", 110), ("yy", 120), ("xx", 130)];
    let list = build(&words, &["a"], &plan(&[], None));
    let coverage = list.coverage.unwrap();
    assert_eq!((coverage.rejected, coverage.top), (2, 3));
    assert_eq!(coverage.first, ["zz", "yy"]);
}

#[test]
fn the_report_prints_the_counts_the_coverage_and_the_merges() {
    let hamza = arabic();
    let list = build(
        &[("انا", 200), ("أنا", 300), ("zz", 310)],
        &["أنا"],
        &plan(&[], Some(&hamza)),
    );
    let lines = list::report("ar", &list, &fixture::rules());
    assert_eq!(lines.len(), 3);
    assert!(
        lines[0].starts_with("ar: 1 word kept (0 capitalised"),
        "{}",
        lines[0]
    );
    assert!(
        lines[1].ends_with("rejected 1 of the 3 most frequent words; first 1: zz"),
        "{}",
        lines[1]
    );
    assert!(
        lines[2].ends_with("1 word merged; first 1: انا → أنا"),
        "{}",
        lines[2]
    );
}

#[test]
fn a_hamza_form_far_rarer_than_the_bare_word_takes_nothing() {
    let hamza = arabic();
    let words = [("الذى", 300), ("ألذى", 401), ("اخرى", 300), ("أخرى", 400)];
    let list = build(&words, &["ألذى", "أخرى"], &plan(&[], Some(&hamza)));
    assert_eq!(
        list.words,
        [word("أخرى", 300, false), word("ألذى", 401, false)]
    );
    let merged = vec![("اخرى".to_string(), "أخرى".to_string())];
    assert_eq!(
        list.merged, merged,
        "100 centibels rarer still merges, 101 does not"
    );
    assert_eq!(
        list.coverage.map(|c| c.first),
        Some(vec!["الذى".to_string()])
    );
}
