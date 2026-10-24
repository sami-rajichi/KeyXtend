//! The Arabic hamza rule: a word the dictionary rejects (`انا`) that differs from a kept word (`أنا`) only by a bare
//! alef where that word has أ, إ or آ is dropped, and its most frequent such word takes the higher frequency.
//! A bare word the dictionary knows (`كان`) stays, and a form far rarer than it (`ألذى` for `الذى`) takes nothing.

use std::collections::HashMap;

use super::list::Cand;
use super::manifest::Hamza;

/// Applies the rule to `cands`, marking each dropped word as merged; returns each drop and the word it went to, as
/// places in `cands`, in list order.
pub(crate) fn merge(cands: &mut [Cand], hamza: &Hamza) -> Vec<(usize, usize)> {
    let key = |word: &str| -> String { word.chars().map(|c| bare(c, hamza)).collect() };
    let mut groups: HashMap<String, Vec<usize>> = HashMap::new();
    for (i, cand) in cands.iter().enumerate() {
        if cand.kept && cand.spelling.chars().any(|c| hamza.marked.contains(&c)) {
            groups.entry(key(&cand.spelling)).or_default().push(i);
        }
    }
    let mut merged = Vec::new();
    for i in 0..cands.len() {
        let Some((word, bucket)) = cands
            .get(i)
            .filter(|c| !c.kept)
            .map(|c| (c.spelling.clone(), c.bucket))
        else {
            continue;
        };
        let near = |c: &Cand| c.bucket <= bucket.saturating_add(hamza.max_gap_cb);
        // Each group is in list order, so the first match is the most frequent hamza form.
        let to = groups
            .get(&key(&word))
            .into_iter()
            .flatten()
            .copied()
            .find(|&j| {
                cands
                    .get(j)
                    .is_some_and(|c| near(c) && only_bare_differs(&word, &c.spelling, hamza))
            });
        if let Some(to) = to.filter(|&j| take(cands, i, j)) {
            merged.push((i, to));
        }
    }
    merged
}

/// Marks `cands[from]` merged and gives `cands[to]` the higher frequency of the two; false if either is missing.
fn take(cands: &mut [Cand], from: usize, to: usize) -> bool {
    let Some(freq) = cands.get(from).map(|c| c.freq) else {
        return false;
    };
    let Some(target) = cands.get_mut(to) else {
        return false;
    };
    target.freq = target.freq.max(freq);
    cands.get_mut(from).is_some_and(|c| {
        c.merged = true;
        true
    })
}

/// `c`, or the bare alef when `c` is one of the marked forms.
fn bare(c: char, hamza: &Hamza) -> char {
    if hamza.marked.contains(&c) {
        hamza.bare
    } else {
        c
    }
}

/// True when `plain` differs from `marked` somewhere, and only by bare alefs where `marked` has a marked form.
fn only_bare_differs(plain: &str, marked: &str, hamza: &Hamza) -> bool {
    let mut differs = false;
    for (a, b) in plain.chars().zip(marked.chars()) {
        if a != b {
            if a != hamza.bare || !hamza.marked.contains(&b) {
                return false;
            }
            differs = true;
        }
    }
    differs && plain.chars().count() == marked.chars().count()
}
