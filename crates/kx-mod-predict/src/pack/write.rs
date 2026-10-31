//! The pack writer, for `cargo xtask words` only (feature `build`): it folds and sorts the words, keeps each
//! context's best followers, and lays the sections out as `layout` says.

use std::collections::{BTreeMap, HashMap};
use std::fmt;

use kx_module_api::ports::{MAX_ISO_LEN, MIN_ISO_LEN, fits_iso};
use kx_module_api::text::{Casing, fold};
use thiserror::Error;

use super::layout::{
    CHAINS_BASE, FREQ, LEXICON, MAX_CHAIN, MAX_SPELLING, MAX_WORDS, MIN_CHAIN, NO_CHAINS, WORDS,
    fits_spelling,
};
use super::{Flags, WordId};

mod encode;

use encode::Header;

/// Why a pack could not be written.
#[derive(Clone, Debug, PartialEq, Eq, Error)]
pub enum WriteError {
    /// The language code fails `fits_iso`.
    #[error("a language code is {MIN_ISO_LEN} to {MAX_ISO_LEN} lower-case letters, not a device")]
    BadIso,
    /// A spelling is empty, longer than `MAX_SPELLING` bytes, or folds to nothing.
    #[error("a spelling is 1 to {MAX_SPELLING} bytes and keeps a letter once folded")]
    BadSpelling,
    /// A chain names a word that was not added first.
    #[error("a chain names a word the pack does not hold: {0}")]
    UnknownWord(String),
    /// A chain's context is empty or longer than the longest chain allows.
    #[error("a chain's context holds 1 to {} words", MAX_CHAIN - 1)]
    BadChain,
    /// A pack with chains would keep no follower per context.
    #[error("a pack with chains keeps at least one follower per context")]
    NoFollowers,
    /// The `chains` flag disagrees with the chains added.
    #[error("the chains flag disagrees with the chains added")]
    ChainsFlag,
    /// More words than the ids can name, sections too big for their offsets, or a pack over `MAX_PACK_BYTES`.
    #[error("the pack is too big for its ids, its offsets or the largest mapped pack")]
    TooBig,
    /// The header written does not end where `layout` says; a bug, never the data.
    #[error("the pack header does not match its layout")]
    Header,
    /// The `fst` builder refused the lexicon.
    #[error("the lexicon could not be built: {0}")]
    Lexicon(String),
}

/// One word as added.
struct Word {
    /// Its correct spelling.
    spelling: String,
    /// Its folded key.
    key: String,
    /// Its frequency byte.
    freq: u8,
}

/// Builds one pack: add the words, then the chains between them, then `finish`.
pub struct PackWriter {
    /// The language code.
    iso: String,
    /// The flags the pack must carry.
    flags: Flags,
    /// The followers kept per context.
    followers: u8,
    /// The words in the order added.
    words: Vec<Word>,
    /// Each spelling's place in `words`.
    index: HashMap<String, usize>,
    /// Each context, as places in `words`, with each next word's place and its best score.
    chains: HashMap<Vec<usize>, HashMap<usize, u8>>,
}

impl fmt::Debug for PackWriter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (iso, words, contexts) = (&self.iso, self.words.len(), self.chains.len());
        f.debug_struct("PackWriter")
            .field("iso", iso)
            .field("words", &words)
            .field("contexts", &contexts)
            .finish_non_exhaustive()
    }
}

impl PackWriter {
    /// A writer for language `iso` whose pack carries `flags` and keeps the best `followers` per context.
    ///
    /// # Errors
    /// `BadIso` for a bad language code; `NoFollowers` when `flags.chains` is set and `followers` is 0.
    pub fn new(iso: &str, flags: Flags, followers: u8) -> Result<Self, WriteError> {
        if !fits_iso(iso) {
            return Err(WriteError::BadIso);
        }
        if flags.chains && followers == 0 {
            return Err(WriteError::NoFollowers);
        }
        let (words, index, chains) = (Vec::new(), HashMap::new(), HashMap::new());
        Ok(Self {
            iso: iso.to_owned(),
            flags,
            followers,
            words,
            index,
            chains,
        })
    }

    /// Adds a word, spelt with its capitals; a spelling added again keeps the higher frequency.
    ///
    /// # Errors
    /// `BadSpelling` when it is empty, over `MAX_SPELLING` bytes, or folds to nothing.
    pub fn add_word(&mut self, spelling: &str, freq: u8) -> Result<(), WriteError> {
        let key = fold(spelling, Casing::of(&self.iso));
        if key.is_empty() || !fits_spelling(spelling.len()) {
            return Err(WriteError::BadSpelling);
        }
        if let Some(word) = self
            .index
            .get(spelling)
            .and_then(|&i| self.words.get_mut(i))
        {
            word.freq = word.freq.max(freq);
            return Ok(());
        }
        self.index.insert(spelling.to_owned(), self.words.len());
        self.words.push(Word {
            spelling: spelling.to_owned(),
            key,
            freq,
        });
        Ok(())
    }

    /// Adds `next` after the words of `context` with `score`; the same chain added again keeps the higher score.
    ///
    /// # Errors
    /// `BadChain` for an empty or too long context; `UnknownWord` for a word not added first.
    pub fn add_chain(&mut self, context: &[&str], next: &str, score: u8) -> Result<(), WriteError> {
        if context.is_empty() || context.len() >= usize::from(MAX_CHAIN) {
            return Err(WriteError::BadChain);
        }
        let places: Vec<usize> = context
            .iter()
            .map(|w| self.place(w))
            .collect::<Result<_, _>>()?;
        let next = self.place(next)?;
        let best = self
            .chains
            .entry(places)
            .or_default()
            .entry(next)
            .or_insert(score);
        *best = (*best).max(score);
        Ok(())
    }

    /// The pack's bytes.
    ///
    /// # Errors
    /// `ChainsFlag` when the flags disagree with the chains added; `TooBig` past the id, offset or
    /// `MAX_PACK_BYTES` limits; `Lexicon` or `Header` when the pack cannot be laid out.
    pub fn finish(self) -> Result<Vec<u8>, WriteError> {
        if self.flags.chains == self.chains.is_empty() {
            return Err(WriteError::ChainsFlag);
        }
        if self.words.len() > MAX_WORDS {
            return Err(WriteError::TooBig);
        }
        let (sorted, ids) = self.sorted()?;
        let tables = self.tables(&ids);
        let mut sections = vec![
            (LEXICON, encode::lexicon(&sorted)?),
            (WORDS, encode::spellings(&sorted)?),
            (FREQ, sorted.iter().map(|w| w.freq).collect()),
        ];
        let max_chain = tables.keys().next_back().copied().unwrap_or(NO_CHAINS);
        let empty = encode::Table::new();
        for n in MIN_CHAIN..=max_chain {
            let table = tables.get(&n).unwrap_or(&empty);
            sections.push((CHAINS_BASE + n, encode::chains(table)?));
        }
        let words = u32::try_from(sorted.len()).map_err(|_| WriteError::TooBig)?;
        let header = Header {
            iso: &self.iso,
            flags: self.flags,
            words,
            max_chain,
            followers: self.followers,
        };
        encode::assemble(&header, &sections)
    }

    /// The place in `words` of `spelling`.
    fn place(&self, spelling: &str) -> Result<usize, WriteError> {
        let unknown = || WriteError::UnknownWord(spelling.to_owned());
        self.index.get(spelling).copied().ok_or_else(unknown)
    }

    /// The words sorted by folded key, then frequency, highest first, then spelling; and each added word's id.
    fn sorted(&self) -> Result<(Vec<&Word>, Vec<WordId>), WriteError> {
        let mut order: Vec<(usize, &Word)> = self.words.iter().enumerate().collect();
        order.sort_by(|(_, a), (_, b)| {
            a.key
                .cmp(&b.key)
                .then(b.freq.cmp(&a.freq))
                .then(a.spelling.cmp(&b.spelling))
        });
        let mut ids = vec![0; order.len()];
        for (id, (place, _)) in order.iter().enumerate() {
            if let Some(slot) = ids.get_mut(*place) {
                *slot = WordId::try_from(id).map_err(|_| WriteError::TooBig)?;
            }
        }
        Ok((order.into_iter().map(|(_, w)| w).collect(), ids))
    }

    /// The chains by length, each context's followers sorted by score, highest first, and cut to `followers`.
    fn tables(&self, ids: &[WordId]) -> BTreeMap<u8, encode::Table> {
        let id = |place: &usize| ids.get(*place).copied();
        let mut tables: BTreeMap<u8, encode::Table> = BTreeMap::new();
        for (context, nexts) in &self.chains {
            let Ok(n) = u8::try_from(context.len() + 1) else {
                continue;
            };
            let key: Option<Vec<WordId>> = context.iter().map(id).collect();
            let mut kept: Vec<(WordId, u8)> = nexts
                .iter()
                .filter_map(|(p, s)| Some((id(p)?, *s)))
                .collect();
            kept.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
            kept.truncate(usize::from(self.followers));
            if let Some(key) = key {
                tables.entry(n).or_default().insert(key, kept);
            }
        }
        tables
    }
}
