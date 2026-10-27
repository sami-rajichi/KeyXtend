//! The early stop (Fagin, Lotem and Naor, 2001, the threshold algorithm): every follower is scored in full, then the
//! learned words in count order and the seed words in frequency order, until no word left can enter. A word that
//! follows nothing scores a sum monotone in its count and its frequency, so the result is the full scan's.

use std::ops::Range;

use super::Model;
use crate::fold::fold;
use crate::pack::{Pack, WordId};
use crate::score::sink::Sink;

/// The frequency bytes a pack holds, one bucket each.
const LEVELS: usize = 1 << u8::BITS;

impl Model<'_> {
    /// Offers `sink` the candidates for `folded` that can win: every follower that fits it, then the learned and the
    /// seed words best first, until the most a word left can score no longer enters.
    pub(in crate::score) fn offer(&self, folded: &str, sink: &mut Sink<'_>) {
        self.offer_known(folded, sink);
        let pack = self.seed.as_ref().map(|s| s.pack);
        let mut learned = self.learned.by_count(folded).peekable();
        let ranked = pack.map(|p| by_freq(p, p.span(folded)));
        let mut seed = ranked.unwrap_or_default().into_iter().peekable();
        loop {
            let count = learned.peek().map(|&(_, count)| count);
            let freq = pack.zip(seed.peek()).map(|(p, &id)| p.freq(id));
            let most = self.ceiling(count.unwrap_or_default(), freq);
            if (count.is_none() && freq.is_none()) || !sink.wants(most) {
                return;
            }
            if let Some((word, _)) = learned.next() {
                self.offer_learned(word, sink);
            }
            let next = pack.zip(seed.next()).map(|(p, id)| p.spelling(id));
            if let Some(word) = next.filter(|w| !w.is_empty()) {
                self.offer_learned(word, sink);
            }
        }
    }

    /// Offers every candidate for `folded` in full, with no early stop: the reference `offer` must match.
    #[cfg(all(test, feature = "build"))]
    pub(in crate::score) fn offer_every(&self, folded: &str, sink: &mut Sink<'_>) {
        self.learned
            .starting(folded, &mut |w| self.offer_learned(w, sink));
        let Some(seed) = &self.seed else {
            return;
        };
        let ids: Vec<WordId> = if folded.is_empty() {
            (0..seed.pack.word_count()).collect()
        } else {
            seed.pack.completions(folded).collect()
        };
        for id in ids {
            let word = seed.pack.spelling(id);
            if !word.is_empty() {
                self.offer_learned(word, sink);
            }
        }
    }

    /// Offers every follower that fits `folded`, learned and seed, in full: a follower's score is not bounded by its
    /// count or frequency.
    fn offer_known(&self, folded: &str, sink: &mut Sink<'_>) {
        let fits = |w: &str| folded.is_empty() || fold(w, self.casing).starts_with(folded);
        self.each_follower(&fits, &mut |w| self.offer_learned(w, sink));
    }
}

/// The ids of `span`, most frequent first, lower ids first among equals: a counting sort over the frequency bytes.
fn by_freq(pack: &Pack<'_>, span: Range<WordId>) -> Vec<WordId> {
    let mut starts = [0_usize; LEVELS];
    for id in span.clone() {
        if let Some(n) = starts.get_mut(usize::from(pack.freq(id))) {
            *n += 1;
        }
    }
    let mut end = 0;
    for start in starts.iter_mut().rev() {
        (*start, end) = (end, end + *start);
    }
    let mut out = vec![0; end];
    for id in span {
        if let Some(at) = starts.get_mut(usize::from(pack.freq(id))) {
            if let Some(slot) = out.get_mut(*at) {
                *slot = id;
            }
            *at += 1;
        }
    }
    out
}
