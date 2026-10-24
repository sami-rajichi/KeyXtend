//! One language's word list: wordfreq's entries cleaned, elided, checked against the dictionary, merged by the hamza
//! rule and cut to the most frequent; with the coverage line that shows the dictionary's gaps.

use std::cmp::Reverse;
use std::collections::HashMap;

use super::dict::{Dict, Verdict};
use super::manifest::{Plan, Rules};
use super::wordfreq::{self, Entry};
use super::{clean, hamza};
use crate::text::plural;

/// The noun the report counts.
const WORD: &str = "word";

/// One word of a finished list.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Word {
    /// Its correct spelling, capitalised where the dictionary wants it.
    pub(crate) spelling: String,
    /// Its frequency byte: higher is more frequent.
    pub(crate) freq: u8,
    /// True when it is always capitalised.
    pub(crate) capital: bool,
}

/// How many of the most frequent words the dictionary rejected.
#[derive(Debug, PartialEq, Eq)]
pub(crate) struct Coverage {
    /// How many it rejected.
    pub(crate) rejected: usize,
    /// How many of the most frequent words were looked at.
    pub(crate) top: usize,
    /// The first rejected ones, most frequent first.
    pub(crate) first: Vec<String>,
}

/// A finished word list.
#[derive(Debug)]
pub(crate) struct WordList {
    /// The kept words, most frequent first.
    pub(crate) words: Vec<Word>,
    /// How many entries wordfreq's list held.
    pub(crate) read: usize,
    /// The coverage line's numbers, for a language with a dictionary.
    pub(crate) coverage: Option<Coverage>,
    /// Each word the hamza rule dropped, with a word it went to.
    pub(crate) merged: Vec<(String, String)>,
}

/// A token that passed the token rules, before the hamza rule and the cut.
#[derive(Debug)]
pub(crate) struct Cand {
    /// Its spelling.
    pub(crate) spelling: String,
    /// Its frequency byte.
    pub(crate) freq: u8,
    /// True when the dictionary wants its capital.
    pub(crate) capital: bool,
    /// True when the dictionary accepts it, or it is an elision.
    pub(crate) kept: bool,
    /// True when the hamza rule dropped it.
    pub(crate) merged: bool,
    /// Its wordfreq bucket.
    pub(crate) bucket: u32,
    /// Its place in wordfreq's list.
    pub(crate) rank: usize,
}

/// The word list of `entries`, as `plan` and `rules` say, checked against `dict` when there is one.
pub(crate) fn build(
    entries: Vec<Entry>,
    plan: &Plan<'_>,
    dict: Option<&Dict>,
    rules: &Rules,
) -> WordList {
    let read = entries.len();
    let mut cands = candidates(entries, plan, dict, rules);
    let places = plan
        .hamza
        .map(|h| hamza::merge(&mut cands, h))
        .unwrap_or_default();
    let spelling = |i: usize| cands.get(i).map(|c| c.spelling.clone()).unwrap_or_default();
    let merged = places
        .iter()
        .map(|&(from, to)| (spelling(from), spelling(to)))
        .collect();
    let coverage = dict.map(|_| coverage(&cands, rules));
    WordList {
        words: survivors(cands, plan.max_words),
        read,
        coverage,
        merged,
    }
}

/// Every entry that passes the token rules, elided and with the dictionary's verdict.
fn candidates(
    entries: Vec<Entry>,
    plan: &Plan<'_>,
    dict: Option<&Dict>,
    rules: &Rules,
) -> Vec<Cand> {
    let mut cands = Vec::new();
    for (rank, entry) in entries.into_iter().enumerate() {
        let elided = clean::elide(&entry.word, plan.elisions, rules.apostrophe);
        let exempt = elided.is_some();
        let word = elided.unwrap_or(entry.word);
        if !clean::keeps(&word, &rules.drop_chars) {
            continue;
        }
        let verdict = match dict {
            Some(dict) if !exempt => dict.check(&word),
            _ => Verdict::Plain,
        };
        let (spelling, capital, kept) = match verdict {
            Verdict::Plain => (word, false, true),
            Verdict::Capital(capital) => (capital, true, true),
            Verdict::Rejected => (word, false, false),
        };
        let freq = wordfreq::freq(entry.bucket, rules.steps_per_zipf);
        cands.push(Cand {
            spelling,
            freq,
            capital,
            kept,
            merged: false,
            bucket: entry.bucket,
            rank,
        });
    }
    cands
}

/// The words among wordfreq's `coverage_top` most frequent that the dictionary rejected; a word the hamza rule
/// merged is not a gap, as its frequency went to its hamza form.
fn coverage(cands: &[Cand], rules: &Rules) -> Coverage {
    let rejected: Vec<&str> = cands
        .iter()
        .take_while(|c| c.rank < rules.coverage_top)
        .filter(|c| !c.kept && !c.merged)
        .map(|c| c.spelling.as_str())
        .collect();
    Coverage {
        rejected: rejected.len(),
        top: rules.coverage_top,
        first: rejected
            .iter()
            .take(rules.coverage_show)
            .map(ToString::to_string)
            .collect(),
    }
}

/// The kept words, each spelling once with its highest frequency, most frequent first and cut to `max_words`.
fn survivors(cands: Vec<Cand>, max_words: usize) -> Vec<Word> {
    let mut words: Vec<Word> = Vec::new();
    let mut index: HashMap<String, usize> = HashMap::new();
    for cand in cands.into_iter().filter(|c| c.kept && !c.merged) {
        if let Some(word) = index.get(&cand.spelling).and_then(|&i| words.get_mut(i)) {
            word.freq = word.freq.max(cand.freq);
            word.capital |= cand.capital;
            continue;
        }
        index.insert(cand.spelling.clone(), words.len());
        words.push(Word {
            spelling: cand.spelling,
            freq: cand.freq,
            capital: cand.capital,
        });
    }
    // A stable sort keeps wordfreq's order between words of one frequency.
    words.sort_by_key(|w| Reverse(w.freq));
    words.truncate(max_words);
    words
}

/// The lines `words list` prints for `iso`: the counts, the coverage line and the hamza line.
pub(crate) fn report(iso: &str, list: &WordList, rules: &Rules) -> Vec<String> {
    let caps = list.words.iter().filter(|w| w.capital).count();
    let freqs = list.words.iter().map(|w| w.freq);
    let (low, high) = (
        freqs.clone().min().unwrap_or_default(),
        freqs.max().unwrap_or_default(),
    );
    let kept = list.words.len();
    let mut lines = vec![format!(
        "{iso}: {kept} {} kept ({caps} capitalised, frequency {low}..={high}) of {} in wordfreq",
        plural(kept, WORD),
        list.read
    )];
    if let Some(c) = &list.coverage {
        lines.push(format!(
            "coverage {iso}: the dictionary rejected {} of the {} most frequent words; first {}: {}",
            c.rejected,
            c.top,
            c.first.len(),
            c.first.join(", ")
        ));
    }
    if !list.merged.is_empty() {
        let first: Vec<String> = list
            .merged
            .iter()
            .take(rules.coverage_show)
            .map(|(from, to)| format!("{from} → {to}"))
            .collect();
        let count = list.merged.len();
        lines.push(format!(
            "hamza {iso}: {count} {} merged; first {}: {}",
            plural(count, WORD),
            first.len(),
            first.join(", ")
        ));
    }
    lines
}
