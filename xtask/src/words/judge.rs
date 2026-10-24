//! The text rules of a word list: twins pick their spelling, pieces go, and apostrophe words wordfreq lacks take
//! their frequency from the text, kept on wordfreq's scale through their right piece.

use std::cmp::Reverse;
use std::collections::{HashMap, HashSet};

use super::cands::{Cand, Tally};
use super::float;
use super::manifest::Rules;
use super::text::TextCounts;
use super::wordfreq::{bucket_of, bucket_of_share, share};

/// What the text rules read: the counts, wordfreq's tokens, and what apostrophe words split into.
pub(crate) struct Judge<'a> {
    /// The text's counts.
    pub(crate) text: &'a TextCounts,
    /// Every wordfreq token with its bucket.
    tokens: &'a HashMap<String, u32>,
    /// How often the apostrophe forms holding each piece anywhere were seen.
    joined: HashMap<String, u64>,
    /// How often the apostrophe forms ending in each piece were seen.
    ending: HashMap<String, u64>,
}

impl<'a> Judge<'a> {
    /// The rules' view of `text`, with the pieces of every candidate form that holds `apostrophe`.
    pub(crate) fn new(
        cands: &[Cand],
        tokens: &'a HashMap<String, u32>,
        text: &'a TextCounts,
        apostrophe: char,
    ) -> Self {
        let forms: HashSet<&str> = (cands.iter())
            .flat_map(|c| c.forms.iter().map(String::as_str))
            .filter(|f| f.contains(apostrophe))
            .collect();
        let (mut joined, mut ending) = (HashMap::new(), HashMap::new());
        for form in forms {
            let seen = text.count(form);
            let pieces: HashSet<String> = (form.split(apostrophe))
                .filter(|p| !p.is_empty())
                .map(str::to_lowercase)
                .collect();
            for piece in pieces {
                *joined.entry(piece).or_default() += seen;
            }
            let last = form.rsplit(apostrophe).next().unwrap_or_default();
            *ending.entry(last.to_lowercase()).or_default() += seen;
        }
        Self {
            text,
            tokens,
            joined,
            ending,
        }
    }

    /// How often the apostrophe words holding `spelling` were seen, when it is a piece: they were seen at least
    /// `piece_min_seen` times and over `piece_ratio` times as often as it stands alone (`ll` of `I'll`).
    pub(crate) fn piece(&self, spelling: &str, rules: &Rules) -> Option<u64> {
        let joined = self.joined.get(&spelling.to_lowercase()).copied();
        let joined = joined.unwrap_or_default();
        let dwarfed = float(joined) > rules.piece_ratio * float(self.text.alone(spelling));
        (joined >= rules.piece_min_seen && dwarfed).then_some(joined)
    }

    /// True when `spelling` holds the apostrophe and each of its pieces is a wordfreq token (`don't`, `l'`), so the
    /// stem rule decides its frequency.
    pub(crate) fn is_stem(&self, spelling: &str, apostrophe: char) -> bool {
        let mut pieces = spelling.split(apostrophe).filter(|p| !p.is_empty());
        spelling.contains(apostrophe) && pieces.all(|p| self.tokens.contains_key(&p.to_lowercase()))
    }

    /// The bucket of the stem `spelling` of `bucket`, `None` when the text never shows it. A wordfreq token keeps its
    /// own; with a right piece (`hui` of a joined `aujourd'hui`) it takes its share of the piece's wordfreq share, as
    /// wordfreq split it there; an elision takes its text share.
    pub(crate) fn stem_bucket(&self, spelling: &str, bucket: u32, apostrophe: char) -> Option<u32> {
        let seen = self.text.count(spelling);
        if seen == 0 || self.tokens.contains_key(&spelling.to_lowercase()) {
            return (seen > 0).then_some(bucket);
        }
        let right = spelling
            .rsplit(apostrophe)
            .next()
            .unwrap_or_default()
            .to_lowercase();
        let Some(&piece) = self.tokens.get(&right) else {
            return bucket_of(seen, self.text.total);
        };
        let ending = self.ending.get(&right).copied().unwrap_or_default();
        let uses = (self.text.alone(&right) + ending).max(seen);
        bucket_of_share(share(piece) * float(seen) / float(uses))
    }
}

/// The spelling `cand` takes: its other form seen most standing alone away from sentence starts, once seen
/// `min_case_seen` times and more than its own form, `twin_ratio` times more for a change of case only.
pub(crate) fn spelling(
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
    let case = form.to_lowercase() == own.to_lowercase();
    let ratio = if case { rules.twin_ratio } else { 1.0 };
    if seen < rules.min_case_seen || float(seen) <= ratio * float(text.seen(own)) {
        return Some(own.clone());
    }
    twins.add(rules.show, || format!("{} → {form}", cand.word));
    Some(form.clone())
}
