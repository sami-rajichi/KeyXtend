//! Counts from real text, which twins, pieces and stems need: filled by the text counter, read here through fold
//! keys. Every count is weighted by its source.

use std::collections::HashMap;

use kx_mod_predict::{Casing, fold};

/// The counts of one fold key in the text.
#[derive(Debug, Default)]
pub(crate) struct KeyCounts {
    /// How often a word of this key stands alone, not glued to another by an apostrophe or hyphen.
    pub(crate) alone: u64,
    /// How often each spelling of this key stands alone away from sentence starts.
    pub(crate) spellings: HashMap<String, u64>,
}

/// Word counts from real text.
#[derive(Debug)]
pub(crate) struct TextCounts {
    /// How many tokens the text held.
    pub(crate) total: u64,
    /// The counts of each fold key.
    pub(crate) keys: HashMap<String, KeyCounts>,
    /// How often tokens were taken as each spelling, by spelling, case or fold.
    pub(crate) found: HashMap<String, u64>,
    /// How the language cases words, which the keys were folded with.
    pub(crate) casing: &'static Casing,
}

impl TextCounts {
    /// How often a word folding like `word` stands alone.
    pub(crate) fn alone(&self, word: &str) -> u64 {
        self.keys
            .get(&fold(word, self.casing))
            .map_or(0, |k| k.alone)
    }

    /// How often tokens were taken as `spelling` itself, anywhere: `ç'` and `c'` share a fold key, not this count.
    pub(crate) fn count(&self, spelling: &str) -> u64 {
        self.found.get(spelling).copied().unwrap_or_default()
    }

    /// How often `spelling` itself stands alone away from sentence starts.
    pub(crate) fn seen(&self, spelling: &str) -> u64 {
        self.keys
            .get(&fold(spelling, self.casing))
            .and_then(|k| k.spellings.get(spelling))
            .copied()
            .unwrap_or_default()
    }
}
