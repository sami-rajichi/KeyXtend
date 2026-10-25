//! The second stage of a word list: with real text, the text rules (`judge`) run; then the cut to the most frequent
//! words, and the rarest go until the pack without its chains fits the words budget.

use std::cmp::Reverse;
use std::collections::{HashMap, HashSet};

use super::cands::{Cand, Candidates, Facts, Fate, Tally};
use super::judge::{self, Judge};
use super::manifest::Rules;
use super::text::TextCounts;
use super::wordfreq::{freq, share};
use super::{WordsError, fit};

/// One word of a finished list.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Word {
    /// Its correct spelling, capitalised where the dictionary wants it.
    pub(crate) spelling: String,
    /// Its frequency byte: higher is more frequent.
    pub(crate) freq: u8,
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

/// What the text rules did.
#[derive(Debug, Default)]
pub(crate) struct TextFacts {
    /// Tokens whose twin spelling won in the text.
    pub(crate) twins: Tally,
    /// Words dropped as tokenizer pieces.
    pub(crate) pieces: Tally,
    /// Apostrophe words, no wordfreq token, whose frequency the stem rule changed.
    pub(crate) stems: Tally,
    /// Apostrophe words dropped as never seen in the text.
    pub(crate) unseen: Tally,
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
    /// What the text rules did.
    pub(crate) text_facts: TextFacts,
    /// For each candidate, its word's place in `words`, when it made the list.
    pub(crate) of_cand: Vec<Option<u32>>,
    /// True when text counts were given.
    pub(crate) text: bool,
    /// The share of wordfreq's frequency mass the kept words take.
    pub(crate) covers: f64,
    /// How many words the words budget cut, and the budget in bytes.
    pub(crate) cut: (usize, usize),
}

/// A kept candidate with the spelling it takes.
struct Kept {
    /// Its spelling.
    spelling: String,
    /// Its bucket.
    bucket: u32,
    /// Its place in wordfreq's list.
    rank: usize,
    /// Its place among the candidates.
    cand: usize,
}

/// The finished list of `cands`; the text rules and the `Stem` candidates run only with `text`.
pub(crate) fn finish(
    cands: Candidates,
    text: Option<&TextCounts>,
    rules: &Rules,
) -> Result<WordList, WordsError> {
    let coverage = cands.checked.then(|| coverage(&cands.cands, rules));
    let mut facts = TextFacts::default();
    let judge = text.map(|text| Judge::new(&cands.cands, &cands.tokens, text, rules.apostrophe));
    let wanted = |c: &Cand| c.fate == Fate::Kept || (c.fate == Fate::Stem && text.is_some());
    let kept: Vec<Kept> = (cands.cands.iter().enumerate())
        .filter(|(_, c)| wanted(c))
        .filter_map(|(i, c)| keep(i, c, judge.as_ref(), rules, &mut facts))
        .collect();
    let (mut kept, mut of_cand) = survivors(kept, cands.max_words, cands.cands.len());
    let all: Vec<Word> = kept.iter().map(word).collect();
    let fits = fit::fitting(&cands.iso, &all, cands.words_budget)?;
    let cut = kept.len() - fits;
    kept.truncate(fits);
    let gone = |w: &Option<u32>| {
        w.and_then(|w| usize::try_from(w).ok())
            .is_some_and(|w| w >= fits)
    };
    of_cand
        .iter_mut()
        .filter(|w| gone(w))
        .for_each(|w| *w = None);
    let covers = covered(&cands, &of_cand) / cands.mass.max(f64::MIN_POSITIVE);
    Ok(WordList {
        words: kept.iter().map(word).collect(),
        read: cands.read,
        coverage,
        facts: cands.facts,
        text_facts: facts,
        of_cand,
        text: text.is_some(),
        covers,
        cut: (cut, cands.words_budget),
    })
}

/// The wordfreq share of the tokens whose candidates made the list, each token once, so it never passes the list's.
fn covered(cands: &Candidates, of_cand: &[Option<u32>]) -> f64 {
    let mut seen = HashSet::new();
    let listed = (cands.cands.iter().zip(of_cand)).filter(|(_, w)| w.is_some());
    let tokens = listed.filter_map(|(c, _)| Some((c.word.as_str(), *cands.tokens.get(&c.word)?)));
    tokens
        .filter(|(t, _)| seen.insert(*t))
        .map(|(_, b)| share(b))
        .sum()
}

/// Candidate `i` with its spelling and bucket, or `None` when the text shows a piece or never shows its stem.
fn keep(
    i: usize,
    cand: &Cand,
    judge: Option<&Judge<'_>>,
    rules: &Rules,
    facts: &mut TextFacts,
) -> Option<Kept> {
    let spelling = judge::spelling(cand, judge.map(|j| j.text), rules, &mut facts.twins)?;
    let mut bucket = cand.bucket;
    if let Some(judge) = judge {
        if let Some(joined) = judge.piece(&spelling, rules) {
            let alone = judge.text.alone(&spelling);
            let line = format!("{spelling} {alone}/{joined}");
            facts.pieces.add(rules.show, || line);
            return None;
        }
        if judge.is_stem(&spelling, rules.apostrophe) {
            let Some(seen) = judge.stem_bucket(&spelling, bucket, rules.apostrophe) else {
                facts.unseen.add(rules.show, || spelling);
                return None;
            };
            if seen != bucket {
                let shift = format!("{spelling} {} → {}", freq(bucket), freq(seen));
                facts.stems.add(rules.show, || shift);
                bucket = seen;
            }
        }
    }
    let rank = cand.rank;
    Some(Kept {
        spelling,
        bucket,
        rank,
        cand: i,
    })
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

/// Each spelling once with its lowest bucket, most frequent first and cut to `max_words`, with each of the `cands`
/// candidates' place in the result; words of one frequency byte keep wordfreq's order.
fn survivors(kept: Vec<Kept>, max_words: usize, cands: usize) -> (Vec<Kept>, Vec<Option<u32>>) {
    let mut words: Vec<Kept> = Vec::new();
    let mut index: HashMap<String, usize> = HashMap::new();
    let mut owners = Vec::with_capacity(kept.len());
    for word in kept {
        if let Some(&slot) = index.get(&word.spelling) {
            if let Some(seen) = words.get_mut(slot) {
                seen.bucket = seen.bucket.min(word.bucket);
                seen.rank = seen.rank.min(word.rank);
            }
            owners.push((word.cand, slot));
            continue;
        }
        index.insert(word.spelling.clone(), words.len());
        owners.push((word.cand, words.len()));
        words.push(word);
    }
    let mut order: Vec<(usize, Kept)> = words.into_iter().enumerate().collect();
    order.sort_by_key(|(_, w)| (Reverse(freq(w.bucket)), w.rank));
    order.truncate(max_words);
    let mut place = vec![None; index.len()];
    for (at, (slot, _)) in order.iter().enumerate() {
        if let Some(p) = place.get_mut(*slot) {
            *p = u32::try_from(at).ok();
        }
    }
    let mut of_cand = vec![None; cands];
    for (cand, slot) in owners {
        if let Some(c) = of_cand.get_mut(cand) {
            *c = place.get(slot).copied().flatten();
        }
    }
    (order.into_iter().map(|(_, w)| w).collect(), of_cand)
}

/// The finished word of `kept`.
fn word(kept: &Kept) -> Word {
    Word {
        freq: freq(kept.bucket),
        spelling: kept.spelling.clone(),
    }
}
