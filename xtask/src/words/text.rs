//! Counts from real text, which twins and pieces need: filled by the text counter, read here through fold keys.

use std::collections::HashMap;

use kx_mod_predict::fold;

/// The counts of one fold key in the text.
#[derive(Debug, Default)]
pub(crate) struct KeyCounts {
    /// How often a word of this key stands alone, not glued to another by an apostrophe or hyphen.
    pub(crate) alone: u64,
    /// How often each spelling of this key is seen away from sentence starts.
    pub(crate) spellings: HashMap<String, u64>,
}

/// Word counts from real text.
#[derive(Debug, Default)]
pub(crate) struct TextCounts {
    /// How many tokens the text held.
    pub(crate) total: u64,
    /// The counts of each fold key.
    pub(crate) keys: HashMap<String, KeyCounts>,
}

impl TextCounts {
    /// How often a word folding like `word` stands alone.
    pub(crate) fn alone(&self, word: &str) -> u64 {
        self.keys.get(&fold(word)).map_or(0, |k| k.alone)
    }

    /// How often `spelling` itself is seen away from sentence starts.
    pub(crate) fn seen(&self, spelling: &str) -> u64 {
        self.keys
            .get(&fold(spelling))
            .and_then(|k| k.spellings.get(spelling))
            .copied()
            .unwrap_or_default()
    }
}
