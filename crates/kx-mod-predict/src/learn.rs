//! The learner (spec §6 "Learning", ADR-0032 decision 4, after AOSP `LatinIME`): per language, counts and last-used
//! days for the words typed and the chains they end, capped, cleaned, purged and halved, with forgotten seed words.

use std::fmt;

use kx_module_api::ports::fits_iso;

use crate::casing::Casing;
use crate::pack::Pack;
use crate::pack::layout::{MAX_CHAIN, MIN_CHAIN};

mod bytes;
mod chains;
mod clean;
mod forget;
mod resolve;
mod table;
mod view;

use bytes::MAX_TEXT;
pub use bytes::{MAGIC, MAX_CHAINS, VERSION};
use chains::Chains;
use clean::{CLEAN_AT_TENTHS, HEADROOM, TENTHS, stale, too_high};
use forget::{Blocked, Undo, Use};
use resolve::seed_spelling;
use table::{Id, Word, Words};
pub use view::View;

/// The most `max_words` a learner takes: at its peak, every word id still fits in 16 bits.
pub const MAX_LEARNED: usize = Id::MAX as usize / CLEAN_AT_TENTHS * TENTHS;
/// The most `min_uses` a learner takes: one learn adds at most that many uses, which the halving headroom holds.
pub const MAX_MIN_USES: u32 = HEADROOM as u32;

/// What the learner needs from the settings; the module fills it from `[predict]`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LearnConfig {
    /// The uses a forgotten seed word needs to come back (`min_uses`).
    pub min_uses: u32,
    /// The most words kept (`max_words`).
    pub max_words: usize,
    /// The most chains kept of each length (`max_chains`).
    pub max_chains: usize,
    /// Days unused after which an entry is dropped (`forget_days`).
    pub forget_days: u32,
    /// The most previous words a chain holds (`context_words`).
    pub context_words: usize,
}

/// Why a learner could not be made or read.
#[derive(Copy, Clone, Debug, PartialEq, Eq, thiserror::Error)]
pub enum LearnFormatError {
    /// The bytes do not start with the learned file's magic.
    #[error("the file is not a learned-words file")]
    BadMagic,
    /// The byte form's version is not this crate's.
    #[error("learned-words version {0} is not supported")]
    BadVersion(u16),
    /// The language code is not one a pack could carry.
    #[error("the language code is not one a learned file can carry")]
    BadIso,
    /// `min_uses` is above `MAX_MIN_USES`.
    #[error("min_uses is above the most the learner takes")]
    BadMinUses,
    /// The bytes end before what they announce.
    #[error("the learned-words file ends early")]
    Truncated,
    /// An entry is malformed, repeated, out of order, names a missing word, or bytes follow the end.
    #[error("a learned-words entry is malformed")]
    BadEntry,
}

/// One language's learned words and chains.
#[derive(Clone)]
pub struct Learner {
    /// The language code.
    iso: String,
    /// The settings.
    cfg: LearnConfig,
    /// The words.
    words: Words,
    /// The chain tables, shortest first: `MIN_CHAIN` words, then one more each.
    chains: Vec<Chains>,
    /// The seed words the user forgot.
    blocked: Blocked,
    /// What the last forget took, until anything else changes.
    undo: Option<Undo>,
}

impl fmt::Debug for Learner {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let chains: Vec<usize> = self.chains.iter().map(Chains::len).collect();
        f.debug_struct("Learner")
            .field("iso", &self.iso)
            .field("words", &self.words.len())
            .field("chains", &chains)
            .field("blocked", &self.blocked.len())
            .finish_non_exhaustive()
    }
}

impl PartialEq for Learner {
    /// Equal when the language, the settings and everything learned are; a pending undo does not count.
    fn eq(&self, other: &Self) -> bool {
        (self.iso == other.iso && self.cfg == other.cfg)
            && (self.words == other.words && self.chains == other.chains)
            && self.blocked == other.blocked
    }
}

impl Learner {
    /// An empty learner for language `iso`. Settings past a limit stop there: `max_words` at `MAX_LEARNED`,
    /// `max_chains` at `MAX_CHAINS`, `context_words` at the longest chain a pack holds.
    ///
    /// # Errors
    /// `BadIso` when `iso` is not a code a pack could carry; `BadMinUses` when `min_uses` passes `MAX_MIN_USES`.
    pub fn new(iso: &str, cfg: LearnConfig) -> Result<Self, LearnFormatError> {
        if !fits_iso(iso) {
            return Err(LearnFormatError::BadIso);
        }
        if cfg.min_uses > MAX_MIN_USES {
            return Err(LearnFormatError::BadMinUses);
        }
        let cfg = LearnConfig {
            max_words: cfg.max_words.min(MAX_LEARNED),
            max_chains: cfg.max_chains.min(MAX_CHAINS),
            context_words: cfg.context_words.min(usize::from(MAX_CHAIN - 1)),
            ..cfg
        };
        let widths = (MIN_CHAIN..).take(cfg.context_words);
        let chains = widths.map(|w| Chains::new(usize::from(w), cfg.max_chains));
        Ok(Self {
            iso: iso.to_owned(),
            cfg,
            words: Words::new(cfg.max_words),
            chains: chains.collect(),
            blocked: Blocked::default(),
            undo: None,
        })
    }

    /// The language code.
    #[must_use]
    pub fn iso(&self) -> &str {
        &self.iso
    }

    /// Adds one use of `word` and of each chain it ends after `context`, oldest first, on `day`; `seed` maps an
    /// informal spelling to the pack's, and entries stored as typed that it spells so merge into it. A pack of
    /// another language is ignored, and so is a word over `MAX_TEXT`.
    pub fn learn(&mut self, seed: Option<&Pack<'_>>, word: &str, context: &[&str], day: u32) {
        self.undo = None;
        if word.is_empty() || word.len() > MAX_TEXT {
            return;
        }
        let seed = self.own(seed);
        if let Some(id) = self.add_word(seed, word, day) {
            self.add_chains(seed, id, context, day);
        }
        self.clean();
    }

    /// Drops every entry unused for more than `forget_days` by `day`, and the chains naming a dropped word. The
    /// blocklist stays, as a forgotten seed word comes back only when typed.
    pub fn purge(&mut self, day: u32) {
        self.undo = None;
        let days = self.cfg.forget_days;
        let gone = self.words.retain(|_, w| !stale(w.day, day, days));
        for table in &mut self.chains {
            table.retain(|_, key, last| !stale(last, day, days) && !names(&gone, key));
        }
    }

    /// The learner as the scorer reads it, with `seed` mapping typed words to the pack's spellings.
    #[must_use]
    pub fn view<'a>(&'a self, seed: Option<&'a Pack<'a>>) -> View<'a> {
        View::new(self, self.own(seed))
    }

    /// How the language cases its words, which its keys are folded with.
    fn casing(&self) -> &'static Casing {
        Casing::of(&self.iso)
    }

    /// `seed`, when it is the pack of this learner's language.
    fn own<'s, 'p>(&self, seed: Option<&'s Pack<'p>>) -> Option<&'s Pack<'p>> {
        seed.filter(|pack| pack.iso() == self.iso)
    }

    /// The table of chains `width` words long, when the settings keep it.
    fn table(&self, width: usize) -> Option<&Chains> {
        self.chains.get(width.checked_sub(usize::from(MIN_CHAIN))?)
    }

    /// Adds one use of `word` on `day` to the entry `resolve` maps it to, after merging its strays; a forgotten seed
    /// word counts on the blocklist instead until it comes back. `None` while it stays hidden.
    fn add_word(&mut self, seed: Option<&Pack<'_>>, word: &str, day: u32) -> Option<Id> {
        let (text, found) = self.resolve(seed, word);
        let uses = match self.blocked.used(text, 1, day, self.cfg.min_uses) {
            Use::Free => 1,
            Use::Held => return None,
            Use::Back(uses) => uses,
        };
        let casing = self.casing();
        let Some(id) = self.absorb(seed, text, found) else {
            let word = Word {
                text: text.into(),
                count: uses,
                day,
            };
            return self.words.insert(casing, word);
        };
        let held = self.words.get(id).map(|w| (&*w.text != text, w.count));
        if held.is_some_and(|h| h.0) && casing.lower(text) == text {
            self.words.respell(id, text);
        }
        if held.is_some_and(|h| too_high(h.1, uses)) {
            self.halve();
        }
        self.words.bump(id, uses, day);
        Some(id)
    }

    /// Halves every count of every table at once, so the scorer's chain-to-word shares hold; `min_uses` is the floor.
    fn halve(&mut self) {
        let floor = u16::try_from(self.cfg.min_uses).unwrap_or(HEADROOM);
        self.words.halve(floor);
        self.chains.iter_mut().for_each(|table| table.halve(floor));
    }

    /// Cleans the word table past its peak, with the chains naming a dropped word, then each chain table.
    fn clean(&mut self) {
        if let Some(gone) = self.words.clean() {
            for table in &mut self.chains {
                table.retain(|_, key, _| !names(&gone, key));
            }
        }
        self.chains.iter_mut().for_each(Chains::clean);
    }

    /// Adds one use of each chain word `id` ends after `context`, back to the first word the learner does not hold.
    fn add_chains(&mut self, seed: Option<&Pack<'_>>, id: Id, context: &[&str], day: u32) {
        let mut key = vec![id];
        for word in context.iter().rev().take(self.chains.len()) {
            let Some(before) = self.lookup(seed, word) else {
                return;
            };
            key.insert(0, before);
            let at = key.len() - usize::from(MIN_CHAIN);
            if (self.chains.get(at)).is_some_and(|t| too_high(t.count(&key), 1)) {
                self.halve();
            }
            if let Some(table) = self.chains.get_mut(at) {
                table.bump(&key, day);
            }
        }
    }
}

/// True when `key` names a word `gone` flags.
fn names(gone: &[bool], key: &[Id]) -> bool {
    key.iter()
        .any(|&id| gone.get(usize::from(id)).copied().unwrap_or_default())
}

#[cfg(test)]
mod props;
#[cfg(test)]
pub(crate) mod tests;
