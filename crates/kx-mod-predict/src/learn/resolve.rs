//! The one rule mapping a typed word to its entry, for learning and reading alike: the pack's spelling when the pack
//! holds its key, else the word as typed, else the entry it differs from in capitals only. An entry stored as typed
//! that the pack spells otherwise is a stray, merged into the pack's spelling by the next learn of it or a reconcile.

use kx_module_api::text::fold;

use super::Learner;
use super::clean::too_high;
use super::forget::Use;
use super::table::Id;
use crate::pack::Pack;

impl Learner {
    /// Merges every stray into its pack spelling, as a learn would, so none stays hidden; a forgotten word's strays
    /// count on the blocklist. `Actor::load_learner` calls it when `seed` loads.
    pub fn reconcile(&mut self, seed: Option<&Pack<'_>>) {
        self.undo = None;
        let Some(pack) = self.own(seed) else {
            return;
        };
        let spelt = |held: &str| seed_spelling(Some(pack), held).filter(|text| *text != held);
        let mut texts: Vec<&str> = self
            .words
            .iter()
            .filter_map(|(_, w)| spelt(&w.text))
            .collect();
        texts.sort_unstable();
        texts.dedup();
        for text in texts {
            if self.blocked.contains(text) {
                self.release(pack, text);
            } else {
                self.absorb(Some(pack), text, self.words.find(text));
            }
        }
    }

    /// Counts the uses of the forgotten `text`'s strays on the blocklist, as learns of it would: they merge into it
    /// once it comes back, and go while it stays hidden.
    fn release(&mut self, pack: &Pack<'_>, text: &str) {
        let strays = self.strays(Some(pack), text);
        let (uses, day) = (strays.iter().filter_map(|&id| self.words.get(id)))
            .fold((0, 0), |(n, d), w| {
                (w.count.saturating_add(n), w.day.max(d))
            });
        match self.blocked.used(text, uses, day, self.cfg.min_uses) {
            Use::Back(mark) => {
                if let Some(id) = self.absorb(Some(pack), text, self.words.find(text)) {
                    self.bump(id, mark.uses.saturating_sub(uses), mark.day);
                }
            }
            Use::Held | Use::Free => {
                for id in strays {
                    self.words.remove(id);
                    self.unlink(id);
                }
            }
        }
    }

    /// The spelling `word` counts as, with the id of its entry when the learner holds one.
    pub(super) fn resolve<'s>(
        &self,
        seed: Option<&Pack<'s>>,
        word: &'s str,
    ) -> (&'s str, Option<Id>) {
        if let Some(text) = seed_spelling(seed, word) {
            return (text, self.words.find(text));
        }
        let variant = || self.words.variant(self.casing(), word);
        (word, self.words.find(word).or_else(variant))
    }

    /// The id of the entry `word` maps to.
    pub(super) fn lookup(&self, seed: Option<&Pack<'_>>, word: &str) -> Option<Id> {
        self.resolve(seed, word).1
    }

    /// The strays of the pack spelling `text`: entries holding another spelling that the pack spells `text`.
    pub(super) fn strays(&self, seed: Option<&Pack<'_>>, text: &str) -> Vec<Id> {
        if seed.is_none() {
            return Vec::new();
        }
        let stray = |id: &Id| {
            let held = self.words.get(*id).map_or("", |w| &w.text);
            held != text && seed_spelling(seed, held) == Some(text)
        };
        self.words
            .group(self.casing(), text)
            .filter(stray)
            .collect()
    }

    /// Merges the strays of `text` into its entry `id`, else into the first stray, spelt `text` from then on; gives
    /// the entry, if any.
    pub(super) fn absorb(
        &mut self,
        seed: Option<&Pack<'_>>,
        text: &str,
        id: Option<Id>,
    ) -> Option<Id> {
        let mut strays = self.strays(seed, text).into_iter();
        let into = id.or_else(|| strays.next())?;
        if id.is_none() {
            self.words.respell(into, text);
        }
        for from in strays {
            self.merge(from, into);
        }
        Some(into)
    }

    /// Moves word `from`'s uses and chains onto word `into`, which keeps the later day; every table halves first
    /// while a merged count would pass the top, at most once per bit of a count.
    fn merge(&mut self, from: Id, into: Id) {
        for _ in 0..u16::BITS {
            if !self.merge_too_high(from, into) {
                break;
            }
            self.halve();
        }
        let Some(word) = self.words.remove(from) else {
            return;
        };
        let day = self
            .words
            .get(into)
            .map_or(word.day, |w| w.day.max(word.day));
        self.words.bump(into, word.count, day);
        for table in &mut self.chains {
            table.rekey(from, into);
        }
    }

    /// True when merging word `from` into word `into` would pass the top with the word's count or a met chain's.
    fn merge_too_high(&self, from: Id, into: Id) -> bool {
        let count = |id| self.words.get(id).map_or(0, |w| w.count);
        too_high(count(into), count(from))
            || (self.chains.iter()).any(|table| table.rekey_too_high(from, into))
    }
}

/// The pack's spelling `typed` counts toward: itself when the pack spells it so, else the most frequent spelling of
/// its folded key; `None` when the pack does not hold that key.
pub(super) fn seed_spelling<'p>(seed: Option<&Pack<'p>>, typed: &str) -> Option<&'p str> {
    let pack = seed?;
    let run = pack.run(&fold(typed, pack.casing()));
    let exact = run.clone().find(|&id| pack.spelling(id) == typed);
    let id = exact.or(Some(run.start)).filter(|id| run.contains(id))?;
    Some(pack.spelling(id)).filter(|s| !s.is_empty())
}
