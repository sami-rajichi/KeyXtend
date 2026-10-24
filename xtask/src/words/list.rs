//! The second stage of a word list: twins and pieces from real text when there is some, then the cut to the most
//! frequent; with the report lines that show what every rule did.

use std::cmp::Reverse;
use std::collections::HashMap;

use super::cands::{Cand, Candidates, Facts, Fate, Tally};
use super::float;
use super::manifest::Rules;
use super::text::TextCounts;
use super::wordfreq::{freq, share};
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
    /// What the source rules did.
    pub(crate) facts: Facts,
    /// Tokens whose twin spelling won in the text.
    pub(crate) twins: Tally,
    /// Words dropped as tokenizer pieces.
    pub(crate) pieces: Tally,
    /// True when text counts were given.
    pub(crate) text: bool,
}

/// A kept candidate with the spelling it takes.
struct Kept {
    /// Its spelling.
    spelling: String,
    /// Its wordfreq bucket.
    bucket: u32,
    /// Its place in wordfreq's list.
    rank: usize,
}

/// The finished list of `cands`: with `text`, twins pick their spelling and pieces go; without it, neither runs.
pub(crate) fn finish(cands: Candidates, text: Option<&TextCounts>, rules: &Rules) -> WordList {
    let coverage = cands.checked.then(|| coverage(&cands.cands, rules));
    let (mut twins, mut pieces) = (Tally::default(), Tally::default());
    let mut kept = Vec::new();
    for cand in cands.cands.iter().filter(|c| c.fate == Fate::Kept) {
        let Some(spelling) = spelling(cand, text, rules, &mut twins) else {
            continue;
        };
        if text.is_some_and(|t| is_piece(&spelling, cand.bucket, t, rules)) {
            pieces.add(rules.show, || spelling);
            continue;
        }
        let (bucket, rank) = (cand.bucket, cand.rank);
        kept.push(Kept {
            spelling,
            bucket,
            rank,
        });
    }
    WordList {
        words: survivors(kept, cands.max_words),
        read: cands.read,
        coverage,
        facts: cands.facts,
        twins,
        pieces,
        text: text.is_some(),
    }
}

/// The spelling `cand` takes: its form seen most in `text` away from sentence starts, once seen `min_case_seen`
/// times; otherwise its own form.
fn spelling(
    cand: &Cand,
    text: Option<&TextCounts>,
    rules: &Rules,
    twins: &mut Tally,
) -> Option<String> {
    let own = cand.forms.first()?;
    let Some(text) = text.filter(|_| cand.forms.len() > 1) else {
        return Some(own.clone());
    };
    let mut best = (own, text.seen(own));
    for form in cand.forms.iter().skip(1) {
        let seen = text.seen(form);
        if seen > best.1 {
            best = (form, seen);
        }
    }
    let (form, seen) = best;
    if seen < rules.min_case_seen || form == own {
        return Some(own.clone());
    }
    twins.add(rules.show, || format!("{} → {form}", cand.word));
    Some(form.clone())
}

/// True when `spelling` stands alone in `text` under 1 / `piece_ratio` of what its bucket predicts, judged only
/// when that prediction is at least `piece_min_expected`. A spelling with the apostrophe (`l'`, `aujourd'hui`) is
/// never a piece: it rarely stands alone by nature, and the pieces sought (`s`, `t`, `ll`) are bare.
fn is_piece(spelling: &str, bucket: u32, text: &TextCounts, rules: &Rules) -> bool {
    let expected = share(bucket) * float(text.total);
    !spelling.contains(rules.apostrophe)
        && expected >= rules.piece_min_expected
        && float(text.alone(spelling)) < expected / rules.piece_ratio
}

/// The words among wordfreq's `coverage_top` most frequent that the dictionary rejected; a merged, joined or prefix
/// token is no gap, as its frequency went elsewhere or it is no word.
fn coverage(cands: &[Cand], rules: &Rules) -> Coverage {
    let mut gaps = Tally::default();
    let top = cands
        .iter()
        .filter(|c| c.rank < rules.coverage_top && c.fate == Fate::Rejected);
    for cand in top {
        gaps.add(rules.show, || cand.word.clone());
    }
    Coverage {
        rejected: gaps.count,
        top: rules.coverage_top,
        first: gaps.first,
    }
}

/// Each spelling once with its lowest bucket, most frequent first and cut to `max_words`; words of one frequency
/// byte keep wordfreq's order.
fn survivors(kept: Vec<Kept>, max_words: usize) -> Vec<Word> {
    let mut words: Vec<Kept> = Vec::new();
    let mut index: HashMap<String, usize> = HashMap::new();
    for word in kept {
        if let Some(seen) = index.get(&word.spelling).and_then(|&i| words.get_mut(i)) {
            seen.bucket = seen.bucket.min(word.bucket);
            seen.rank = seen.rank.min(word.rank);
            continue;
        }
        index.insert(word.spelling.clone(), words.len());
        words.push(word);
    }
    words.sort_by_key(|w| (Reverse(freq(w.bucket)), w.rank));
    words.truncate(max_words);
    let word = |w: Kept| Word {
        capital: w.spelling.chars().next().is_some_and(char::is_uppercase),
        freq: freq(w.bucket),
        spelling: w.spelling,
    };
    words.into_iter().map(word).collect()
}

/// The lines `words list` prints for `iso`: the counts, the coverage line, one line per rule that did something,
/// and a note when no text was given to a checked list.
pub(crate) fn report(iso: &str, list: &WordList) -> Vec<String> {
    let caps = list.words.iter().filter(|w| w.capital).count();
    let low = list.words.iter().map(|w| w.freq).min().unwrap_or_default();
    let high = list.words.iter().map(|w| w.freq).max().unwrap_or_default();
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
    lines.extend(rule_lines(iso, list));
    if list.coverage.is_some() && !list.text {
        lines.push(format!("text {iso}: none, so twins and pieces are skipped"));
    }
    lines
}

/// One report line per rule that changed or dropped a word: how many, then the first ones.
fn rule_lines(iso: &str, list: &WordList) -> Vec<String> {
    let f = &list.facts;
    let rules = [
        ("script", &f.script),
        ("elisions", &f.elisions),
        ("joins", &f.joins),
        ("merged", &f.merged),
        ("prefixes", &f.prefixes),
        ("twins", &list.twins),
        ("pieces", &list.pieces),
    ];
    let line = |(name, tally): &(&str, &Tally)| {
        let (count, first) = (tally.count, &tally.first);
        let noun = plural(count, WORD);
        let shown = first.join(", ");
        format!(
            "{name} {iso}: {count} {noun}; first {}: {shown}",
            first.len()
        )
    };
    rules
        .iter()
        .filter(|(_, t)| t.count > 0)
        .map(line)
        .collect()
}
