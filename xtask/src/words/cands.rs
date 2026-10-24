//! The first stage of a word list: wordfreq's tokens through the token rules and the main-script filter, each with
//! the forms the dictionary accepts; then joins, prefixes, fold merges and the stems text may add, all read from the
//! sources.

use std::collections::{HashMap, HashSet};

use super::dict::Dict;
use super::manifest::{Plan, Rules};
use super::wordfreq::Entry;
use super::{clean, merge, script};

/// What became of one token.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Fate {
    /// The dictionary accepts at least one of its forms, or the language has no dictionary.
    Kept,
    /// The dictionary accepts none of its forms.
    Rejected,
    /// Rejected, and its frequency went to its fold twin.
    Merged,
    /// Rejected, and one of the dictionary's own prefixes, so it never merges.
    Blocked,
    /// Rejected, and the left piece of a stem joined by an apostrophe.
    Joined,
    /// A dictionary stem whose two pieces are tokens (`don't`), kept only when text gives its frequency.
    Stem,
}

/// One token of wordfreq's list, or one joined stem.
#[derive(Debug)]
pub(crate) struct Cand {
    /// The token as wordfreq spells it, or the joined stem.
    pub(crate) word: String,
    /// The forms the dictionary accepts, its own first; the token itself without a dictionary.
    pub(crate) forms: Vec<String>,
    /// Its wordfreq bucket: lower is more frequent.
    pub(crate) bucket: u32,
    /// Its place in wordfreq's list.
    pub(crate) rank: usize,
    /// What became of it.
    pub(crate) fate: Fate,
}

/// How many words one rule changed or dropped, and the first few.
#[derive(Debug, Default, PartialEq, Eq)]
pub(crate) struct Tally {
    /// How many.
    pub(crate) count: usize,
    /// The first ones, up to the rules' `show`.
    pub(crate) first: Vec<String>,
}

impl Tally {
    /// Counts one more, keeping `item` while fewer than `show` are kept.
    pub(crate) fn add(&mut self, show: usize, item: impl FnOnce() -> String) {
        self.count += 1;
        if self.first.len() < show {
            self.first.push(item());
        }
    }
}

/// What the rules drawn from the sources did.
#[derive(Debug, Default)]
pub(crate) struct Facts {
    /// Words dropped for not being in the list's main script.
    pub(crate) script: Tally,
    /// Rejected tokens whose elision the dictionary accepts.
    pub(crate) elisions: Tally,
    /// Stems joined from two tokens.
    pub(crate) joins: Tally,
    /// Rejected tokens merged into a fold twin.
    pub(crate) merged: Tally,
    /// Rejected tokens that are one of the dictionary's prefixes.
    pub(crate) prefixes: Tally,
}

/// The candidates of one language, ready for the text rules and the cut.
#[derive(Debug)]
pub(crate) struct Candidates {
    /// Every candidate, wordfreq's order first, then the joins.
    pub(crate) cands: Vec<Cand>,
    /// How many entries wordfreq's list held.
    pub(crate) read: usize,
    /// The most words kept.
    pub(crate) max_words: usize,
    /// True when a dictionary checked the tokens.
    pub(crate) checked: bool,
    /// Every token of wordfreq's list, so a spelling's pieces can be checked.
    pub(crate) tokens: HashSet<String>,
    /// What the source rules did.
    pub(crate) facts: Facts,
}

/// The candidates of `entries`, as `plan` and `rules` say, checked against `dict` when there is one.
pub(crate) fn candidates(
    entries: Vec<Entry>,
    plan: &Plan<'_>,
    dict: Option<&Dict>,
    rules: &Rules,
) -> Candidates {
    let read = entries.len();
    let tokens = entries.iter().map(|e| e.word.clone()).collect();
    let mut facts = Facts::default();
    let mut cands: Vec<Cand> = main_script(entries, rules, &mut facts.script)
        .into_iter()
        .map(|(rank, entry)| cand(rank, entry, dict, rules, &mut facts.elisions))
        .collect();
    if let Some(dict) = dict {
        join(&mut cands, dict, rules.show, &mut facts.joins);
        merge::merge(&mut cands, dict, rules, &mut facts);
        stems(&mut cands, dict, &tokens);
    }
    Candidates {
        cands,
        read,
        max_words: plan.max_words,
        checked: dict.is_some(),
        tokens,
        facts,
    }
}

/// The entries that pass the token rules, with their places; when one script holds `main_script_share` of them,
/// the others go and are counted in `dropped`.
fn main_script(entries: Vec<Entry>, rules: &Rules, dropped: &mut Tally) -> Vec<(usize, Entry)> {
    let kept: Vec<(usize, Entry)> = entries
        .into_iter()
        .enumerate()
        .filter(|(_, e)| clean::keeps(&e.word, &rules.drop_chars))
        .collect();
    let words = kept.iter().map(|(_, e)| script::of(&e.word));
    let Some(main) = script::main(words, rules.main_script_share) else {
        return kept;
    };
    let (ours, others): (Vec<_>, Vec<_>) = kept
        .into_iter()
        .partition(|(_, e)| script::of(&e.word).is_none_or(|s| s == main));
    for (_, entry) in others {
        dropped.add(rules.show, || entry.word);
    }
    ours
}

/// The candidate of one token, counting it in `elisions` when only its elision is accepted.
fn cand(
    rank: usize,
    entry: Entry,
    dict: Option<&Dict>,
    rules: &Rules,
    elisions: &mut Tally,
) -> Cand {
    let forms = dict.map_or_else(|| vec![entry.word.clone()], |d| d.forms(&entry.word));
    let elided = format!("{}{}", entry.word, rules.apostrophe);
    if forms.first().is_some_and(|f| *f == elided) {
        elisions.add(rules.show, || format!("{} → {elided}", entry.word));
    }
    let fate = if forms.is_empty() {
        Fate::Rejected
    } else {
        Fate::Kept
    };
    Cand {
        word: entry.word,
        forms,
        bucket: entry.bucket,
        rank,
        fate,
    }
}

/// Adds each stem of `dict` whose left piece is a rejected token and whose right piece is a token; it takes the
/// rarer piece's bucket and the left piece's place, and the left piece is marked joined.
fn join(cands: &mut Vec<Cand>, dict: &Dict, show: usize, joins: &mut Tally) {
    let index: HashMap<String, usize> = cands
        .iter()
        .enumerate()
        .map(|(i, c)| (c.word.clone(), i))
        .collect();
    let find = |piece: &str| index.get(&piece.to_lowercase()).copied();
    let mut found: Vec<(usize, u32, String)> = Vec::new();
    for (stem, left, right) in dict.joins() {
        let (Some(l), Some(r)) = (find(left), find(right)) else {
            continue;
        };
        let (Some(lc), Some(rc)) = (cands.get(l), cands.get(r)) else {
            continue;
        };
        if lc.fate == Fate::Rejected {
            found.push((l, lc.bucket.max(rc.bucket), stem.to_string()));
        }
    }
    for (l, bucket, stem) in found {
        let Some(left) = cands.get_mut(l) else {
            continue;
        };
        left.fate = Fate::Joined;
        let rank = left.rank;
        joins.add(show, || stem.clone());
        cands.push(Cand {
            word: stem.clone(),
            forms: vec![stem],
            bucket,
            rank,
            fate: Fate::Kept,
        });
    }
}

/// Adds each stem of `dict` whose two pieces are tokens and that no candidate spells yet, as a `Stem` that only text
/// keeps; it takes the rarer piece's bucket and the left piece's place.
fn stems(cands: &mut Vec<Cand>, dict: &Dict, tokens: &HashSet<String>) {
    let spelt: HashSet<String> = cands.iter().map(|c| c.word.to_lowercase()).collect();
    let place: HashMap<&str, (u32, usize)> = cands
        .iter()
        .map(|c| (c.word.as_str(), (c.bucket, c.rank)))
        .collect();
    let mut found = Vec::new();
    for (stem, left, right) in dict.joins() {
        let (left, right) = (left.to_lowercase(), right.to_lowercase());
        if spelt.contains(&stem.to_lowercase())
            || !tokens.contains(&left)
            || !tokens.contains(&right)
        {
            continue;
        }
        let (Some(l), Some(r)) = (place.get(left.as_str()), place.get(right.as_str())) else {
            continue;
        };
        found.push((stem.to_string(), l.0.max(r.0), l.1));
    }
    for (stem, bucket, rank) in found {
        cands.push(Cand {
            word: stem.clone(),
            forms: vec![stem],
            bucket,
            rank,
            fate: Fate::Stem,
        });
    }
}
