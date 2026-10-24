//! The second stage of a word list: with real text, twins pick their spelling, pieces go and apostrophe words take
//! their frequency from the text; then the cut to the most frequent. Without text, only the cut runs.

use std::cmp::Reverse;
use std::collections::{HashMap, HashSet};

use super::cands::{Cand, Candidates, Facts, Fate, Tally};
use super::float;
use super::manifest::Rules;
use super::text::TextCounts;
use super::wordfreq::{bucket_of, freq};

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

/// What the text rules did.
#[derive(Debug, Default)]
pub(crate) struct TextFacts {
    /// Tokens whose twin spelling won in the text.
    pub(crate) twins: Tally,
    /// Words dropped as tokenizer pieces.
    pub(crate) pieces: Tally,
    /// Apostrophe words whose frequency came from the text.
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
}

/// What the text rules read: the counts, wordfreq's tokens, and what apostrophe words split into.
struct Judge<'a> {
    /// The text's counts.
    text: &'a TextCounts,
    /// Every wordfreq token.
    tokens: &'a HashSet<String>,
    /// How often the apostrophe forms holding each piece were seen.
    joined: HashMap<String, u64>,
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
pub(crate) fn finish(cands: Candidates, text: Option<&TextCounts>, rules: &Rules) -> WordList {
    let coverage = cands.checked.then(|| coverage(&cands.cands, rules));
    let mut facts = TextFacts::default();
    let judge = text.map(|text| Judge {
        text,
        tokens: &cands.tokens,
        joined: joined(&cands.cands, text, rules.apostrophe),
    });
    let wanted = |c: &Cand| c.fate == Fate::Kept || (c.fate == Fate::Stem && text.is_some());
    let kept: Vec<Kept> = (cands.cands.iter().enumerate())
        .filter(|(_, c)| wanted(c))
        .filter_map(|(i, c)| keep(i, c, judge.as_ref(), rules, &mut facts))
        .collect();
    let (words, of_cand) = survivors(kept, cands.max_words, cands.cands.len());
    WordList {
        words,
        read: cands.read,
        coverage,
        facts: cands.facts,
        text_facts: facts,
        of_cand,
        text: text.is_some(),
    }
}

/// Candidate `i` with its spelling and bucket, or `None` when the text shows a piece or never shows its stem.
fn keep(
    i: usize,
    cand: &Cand,
    judge: Option<&Judge<'_>>,
    rules: &Rules,
    facts: &mut TextFacts,
) -> Option<Kept> {
    let spelling = spelling(cand, judge.map(|j| j.text), rules, &mut facts.twins)?;
    let mut bucket = cand.bucket;
    if let Some(judge) = judge {
        let text = judge.text;
        if let Some(joined) = piece(&spelling, judge, rules) {
            let alone = text.alone(&spelling);
            facts
                .pieces
                .add(rules.show, || format!("{spelling} {alone}/{joined}"));
            return None;
        }
        if is_stem(&spelling, judge.tokens, rules.apostrophe) {
            let Some(seen) = bucket_of(text.count(&spelling), text.total) else {
                facts.unseen.add(rules.show, || spelling);
                return None;
            };
            let shift = format!("{spelling} {} → {}", freq(bucket), freq(seen));
            facts.stems.add(rules.show, || shift);
            bucket = seen;
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

/// The spelling `cand` takes: its other form seen most standing alone away from sentence starts, once seen
/// `min_case_seen` times and more than its own form, `twin_ratio` times more for a change of case only.
fn spelling(
    cand: &Cand,
    text: Option<&TextCounts>,
    rules: &Rules,
    twins: &mut Tally,
) -> Option<String> {
    let own = cand.forms.first()?;
    let Some(text) = text else {
        return Some(own.clone());
    };
    let others = cand.forms.iter().skip(1).map(|f| (f, text.seen(f)));
    let Some((form, seen)) = others.min_by_key(|&(_, n)| Reverse(n)) else {
        return Some(own.clone());
    };
    let own_seen = text.seen(own);
    let case = form.to_lowercase() == own.to_lowercase();
    let ratio = if case { rules.twin_ratio } else { 1.0 };
    if seen < rules.min_case_seen || float(seen) <= ratio * float(own_seen) {
        return Some(own.clone());
    }
    twins.add(rules.show, || format!("{} → {form}", cand.word));
    Some(form.clone())
}

/// How often the apostrophe words holding `spelling` were seen, when it is a piece: they were seen at least
/// `piece_min_seen` times and over `piece_ratio` times as often as it stands alone (`ll` of `I'll`).
fn piece(spelling: &str, judge: &Judge<'_>, rules: &Rules) -> Option<u64> {
    let lower = spelling.to_lowercase();
    let joined = judge.joined.get(&lower).copied().unwrap_or_default();
    let alone = judge.text.alone(spelling);
    let dwarfed = float(joined) > rules.piece_ratio * float(alone);
    (joined >= rules.piece_min_seen && dwarfed).then_some(joined)
}

/// How often the text shows the candidates' apostrophe forms holding each piece, by the piece in lower case: `t`
/// counts `don't` and `can't`.
fn joined(cands: &[Cand], text: &TextCounts, apostrophe: char) -> HashMap<String, u64> {
    let forms: HashSet<&str> = cands
        .iter()
        .flat_map(|c| c.forms.iter().map(String::as_str))
        .filter(|f| f.contains(apostrophe))
        .collect();
    let mut joined: HashMap<String, u64> = HashMap::new();
    for form in forms {
        let seen = text.count(form);
        let pieces: HashSet<String> = (form.split(apostrophe))
            .filter(|p| !p.is_empty())
            .map(str::to_lowercase)
            .collect();
        for piece in pieces {
            *joined.entry(piece).or_default() += seen;
        }
    }
    joined
}

/// True when `spelling` holds the apostrophe and each of its pieces is a wordfreq token (`don't`, `l'`), so its
/// wordfreq frequency mixes in the pieces' and the text's count is the truer one.
fn is_stem(spelling: &str, tokens: &HashSet<String>, apostrophe: char) -> bool {
    let mut pieces = spelling.split(apostrophe).filter(|p| !p.is_empty());
    spelling.contains(apostrophe) && pieces.all(|p| tokens.contains(&p.to_lowercase()))
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
fn survivors(kept: Vec<Kept>, max_words: usize, cands: usize) -> (Vec<Word>, Vec<Option<u32>>) {
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
    (order.into_iter().map(|(_, w)| word(w)).collect(), of_cand)
}

/// The finished word of `kept`.
fn word(kept: Kept) -> Word {
    Word {
        capital: kept.spelling.chars().next().is_some_and(char::is_uppercase),
        freq: freq(kept.bucket),
        spelling: kept.spelling,
    }
}
