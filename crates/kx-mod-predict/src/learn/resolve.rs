//! The one rule mapping a typed word to its entry, for learning and reading alike: the pack's spelling when the pack
//! holds its key, else the word as typed, else the entry it differs from in capitals only. An entry stored as typed
//! that the pack spells otherwise is a stray, merged into the pack's spelling when learning reaches it.

use super::Learner;
use super::table::Id;
use crate::fold::fold;
use crate::pack::Pack;

impl Learner {
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

    /// Merges the strays of `text` into its entry `id`, else into the first stray, spelt `text` from then on, as it
    /// is learned on `day`; gives the entry, if any.
    pub(super) fn absorb(
        &mut self,
        seed: Option<&Pack<'_>>,
        text: &str,
        id: Option<Id>,
        day: u32,
    ) -> Option<Id> {
        let mut strays = self.strays(seed, text).into_iter();
        let into = id.or_else(|| strays.next())?;
        if id.is_none() {
            self.words.respell(into, text);
        }
        for from in strays {
            self.merge(from, into, day);
        }
        Some(into)
    }

    /// Moves word `from`'s uses onto word `into`, used on `day`, and its chains onto `into`'s.
    fn merge(&mut self, from: Id, into: Id, day: u32) {
        let Some(word) = self.words.remove(from) else {
            return;
        };
        self.words.bump(into, word.count, day);
        for table in &mut self.chains {
            table.rekey(from, into);
        }
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
