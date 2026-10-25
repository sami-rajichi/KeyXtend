//! A learner for the scorer's tests: counts set by hand.

use std::collections::{HashMap, HashSet};

use super::Learned;
use crate::casing::Casing;
use crate::fold::fold;

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
}

impl Fake {
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

    /// Blocks the seed word `word`.
    pub(super) fn block(mut self, word: &str) -> Self {
        self.blocked.insert(word.to_owned());
        self
    }
}

impl Learned for Fake {
    fn count(&self, word: &str) -> u32 {
        self.words.get(word).copied().unwrap_or_default()
    }

    fn chain(&self, context: &[&str], word: &str) -> u32 {
        let context = context.iter().map(|w| (*w).to_owned()).collect();
        let key = (context, word.to_owned());
        self.chains.get(&key).copied().unwrap_or_default()
    }

    fn uses(&self) -> u64 {
        self.uses
    }

    fn blocked(&self, word: &str) -> bool {
        self.blocked.contains(word)
    }

    fn starting(&self, folded_prefix: &str, each: &mut dyn FnMut(&str)) {
        for word in self.words.keys() {
            if fold(word, Casing::root()).starts_with(folded_prefix) {
                each(word);
            }
        }
    }

    fn following(&self, context: &[&str], each: &mut dyn FnMut(&str)) {
        for (chain, word) in self.chains.keys() {
            if chain.iter().map(String::as_str).eq(context.iter().copied()) {
                each(word);
            }
        }
    }
}
