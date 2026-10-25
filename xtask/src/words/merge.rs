//! The spelling-swap rule, driven by the fold table: a rejected token (`انا`, `فى`, `ete`) gives its frequency to the
//! most frequent accepted word with its fold key (`أنا`, `في`, `été`), unless that word is far rarer. A rejected
//! token that is one of the dictionary's own prefixes (`ال`) never merges.

use std::collections::HashMap;

use kx_mod_predict::{Casing, fold};

use super::cands::{Cand, Facts, Fate};
use super::dict::Dict;
use super::manifest::Rules;

/// Blocks each rejected prefix, then merges each other rejected token into its fold twin, folding as `casing` cases,
/// and counts both in `facts`.
pub(crate) fn merge(
    cands: &mut [Cand],
    dict: &Dict,
    casing: &Casing,
    rules: &Rules,
    facts: &mut Facts,
) {
    let twins = twins(cands, casing);
    let mut moves = Vec::new();
    for (i, cand) in cands.iter().enumerate() {
        if cand.fate != Fate::Rejected {
            continue;
        }
        if dict.is_prefix(&cand.word) {
            moves.push((i, None));
            continue;
        }
        let near = |j: &usize| {
            let limit = cand.bucket.saturating_add(rules.max_gap_cb);
            cands.get(*j).is_some_and(|t| t.bucket <= limit)
        };
        if let Some(j) = twins.get(&fold(&cand.word, casing)).copied().filter(near) {
            moves.push((i, Some(j)));
        }
    }
    for (from, to) in moves {
        apply(cands, from, to, casing, rules.show, facts);
    }
}

/// For each fold key, the most frequent kept candidate with a form of that key, whatever the list's order.
fn twins(cands: &[Cand], casing: &Casing) -> HashMap<String, usize> {
    let mut best: HashMap<String, usize> = HashMap::new();
    let order = |i: usize| cands.get(i).map(|c| (c.bucket, c.rank));
    for (i, cand) in cands.iter().enumerate() {
        if cand.fate != Fate::Kept {
            continue;
        }
        for form in &cand.forms {
            let slot = best.entry(fold(form, casing)).or_insert(i);
            if order(i) < order(*slot) {
                *slot = i;
            }
        }
    }
    best
}

/// Blocks `cands[from]` when `to` is `None`; otherwise merges it, giving `cands[to]` the lower bucket of the two.
fn apply(
    cands: &mut [Cand],
    from: usize,
    to: Option<usize>,
    casing: &Casing,
    show: usize,
    facts: &mut Facts,
) {
    let Some((word, bucket)) = cands.get(from).map(|c| (c.word.clone(), c.bucket)) else {
        return;
    };
    let key = fold(&word, casing);
    let fate = match to.map(|j| cands.get_mut(j)) {
        Some(Some(target)) => {
            target.bucket = target.bucket.min(bucket);
            let spelling = target.forms.iter().find(|f| fold(f, casing) == key);
            let spelling = spelling.cloned().unwrap_or_default();
            facts.merged.add(show, || format!("{word} → {spelling}"));
            Fate::Merged
        }
        Some(None) => return,
        None => {
            facts.prefixes.add(show, || word);
            Fate::Blocked
        }
    };
    if let Some(cand) = cands.get_mut(from) {
        cand.fate = fate;
    }
}
