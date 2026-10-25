//! The learner as the scorer reads it: every word it is asked about maps to its entry by the rule learning uses.

use std::cmp::Reverse;
use std::collections::BinaryHeap;
use std::iter;

use super::table::Id;
use super::{Learner, seed_spelling};
use crate::pack::Pack;
use crate::score::Learned;

/// A learner with the pack its words map onto.
#[derive(Debug, Clone, Copy)]
pub struct View<'a> {
    /// The learner.
    learner: &'a Learner,
    /// The pack of the learner's language, when it has one.
    seed: Option<&'a Pack<'a>>,
}

impl<'a> View<'a> {
    /// `learner` read with `seed`.
    pub(super) fn new(learner: &'a Learner, seed: Option<&'a Pack<'a>>) -> Self {
        Self { learner, seed }
    }

    /// The ids of `words`; `None` when the learner lacks one.
    fn ids(&self, words: &[&str]) -> Option<Vec<Id>> {
        words
            .iter()
            .map(|w| self.learner.lookup(self.seed, w))
            .collect()
    }
}

impl Learned for View<'_> {
    fn count(&self, word: &str) -> u32 {
        let id = self.learner.lookup(self.seed, word);
        let word = id.and_then(|id| self.learner.words.get(id));
        word.map_or(0, |w| u32::from(w.count))
    }

    fn chain(&self, context: &[&str], word: &str) -> u32 {
        let Some(mut key) = self.ids(context) else {
            return 0;
        };
        let Some(id) = self.learner.lookup(self.seed, word) else {
            return 0;
        };
        key.push(id);
        let table = self.learner.table(key.len());
        table.map_or(0, |t| u32::from(t.count(&key)))
    }

    fn uses(&self) -> u64 {
        self.learner.words.total()
    }

    fn blocked(&self, word: &str) -> bool {
        let blocked = &self.learner.blocked;
        blocked.len() > 0 && blocked.contains(seed_spelling(self.seed, word).unwrap_or(word))
    }

    fn starting(&self, folded_prefix: &str, each: &mut dyn FnMut(&str)) {
        let casing = self.learner.casing();
        self.learner.words.starting(casing, folded_prefix, each);
    }

    fn by_count<'s>(
        &'s self,
        folded_prefix: &str,
    ) -> Box<dyn Iterator<Item = (&'s str, u32)> + 's> {
        let words = &self.learner.words;
        let ids = words.span(self.learner.casing(), folded_prefix);
        let counted = ids
            .iter()
            .filter_map(|&id| Some((words.get(id)?.count, Reverse(id))));
        let mut heap: BinaryHeap<(u16, Reverse<Id>)> = counted.collect();
        Box::new(iter::from_fn(move || {
            let (count, Reverse(id)) = heap.pop()?;
            Some((&*words.get(id)?.text, u32::from(count)))
        }))
    }

    fn following(&self, context: &[&str], each: &mut dyn FnMut(&str)) {
        let Some(ids) = self.ids(context) else {
            return;
        };
        let Some(table) = self.learner.table(ids.len() + 1) else {
            return;
        };
        for id in table.followers(&ids) {
            if let Some(word) = self.learner.words.get(id) {
                each(&word.text);
            }
        }
    }
}
