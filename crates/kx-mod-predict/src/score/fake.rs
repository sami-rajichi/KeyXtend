//! A learner for the scorer's tests: counts set by hand.

use std::cell::Cell;
use std::collections::{HashMap, HashSet};

use kx_module_api::text::{Casing, fold};

use super::Learned;

/// Learned counts a test sets.
#[derive(Debug, Default)]
pub(super) struct Fake {
    /// Each word's uses.
    words: HashMap<String, u32>,
    /// Each chain's uses: its context, then its word.
    chains: HashMap<(Vec<String>, String), u32>,
    /// The seed words the user forgot.
    blocked: HashSet<String>,
    /// The words learned in all.
    uses: u64,
    /// The language in front, whose casing folds the learned words; none folds as Unicode's default.
    iso: String,
    /// The calls the scorer made, so a test sees it read nothing.
    reads: Cell<usize>,
}

impl Fake {
    /// Sets the language in front.
    pub(super) fn lang(mut self, iso: &str) -> Self {
        iso.clone_into(&mut self.iso);
        self
    }

    /// Sets the words learned in all.
    pub(super) fn uses(mut self, uses: u64) -> Self {
        self.uses = uses;
        self
    }

    /// Sets `word`'s uses.
    pub(super) fn word(mut self, word: &str, count: u32) -> Self {
        self.words.insert(word.to_owned(), count);
        self
    }

    /// Sets the uses of `word` after `context`.
    pub(super) fn chain(mut self, context: &[&str], word: &str, count: u32) -> Self {
        let context = context.iter().map(|w| (*w).to_owned()).collect();
        self.chains.insert((context, word.to_owned()), count);
        self
    }

    /// The calls the scorer made.
    pub(super) fn reads(&self) -> usize {
        self.reads.get()
    }

    /// Counts one call.
    fn read(&self) {
        self.reads.set(self.reads.get() + 1);
    }

    /// Blocks the seed word `word`.
    pub(super) fn block(mut self, word: &str) -> Self {
        self.blocked.insert(word.to_owned());
        self
    }
}

impl Learned for Fake {
    fn count(&self, word: &str) -> u32 {
        self.read();
        self.words.get(word).copied().unwrap_or_default()
    }

    fn chain(&self, context: &[&str], word: &str) -> u32 {
        self.read();
        let context = context.iter().map(|w| (*w).to_owned()).collect();
        let key = (context, word.to_owned());
        self.chains.get(&key).copied().unwrap_or_default()
    }

    fn uses(&self) -> u64 {
        self.read();
        self.uses
    }

    fn blocked(&self, word: &str) -> bool {
        self.read();
        self.blocked.contains(word)
    }

    fn starting(&self, folded_prefix: &str, each: &mut dyn FnMut(&str)) {
        self.read();
        for word in self.words.keys() {
            if fold(word, Casing::of(&self.iso)).starts_with(folded_prefix) {
                each(word);
            }
        }
    }

    fn following(&self, context: &[&str], each: &mut dyn FnMut(&str)) {
        self.read();
        for (chain, word) in self.chains.keys() {
            if chain.iter().map(String::as_str).eq(context.iter().copied()) {
                each(word);
            }
        }
    }

    fn by_count<'s>(
        &'s self,
        folded_prefix: &str,
    ) -> Box<dyn Iterator<Item = (&'s str, u32)> + 's> {
        self.read();
        let casing = Casing::of(&self.iso);
        let fits = |w: &&String| fold(w, casing).starts_with(folded_prefix);
        let mut words: Vec<(&str, u32)> = (self.words.iter())
            .filter(|(w, _)| fits(w))
            .map(|(w, &count)| (w.as_str(), count))
            .collect();
        words.sort_unstable_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(b.0)));
        Box::new(words.into_iter())
    }
}
