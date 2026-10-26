//! The learner's byte form, little-endian: a magic, a version, the language code, then the words, the chain tables
//! and the blocklist, and nothing else. The actor seals it; reading checks every length against what is left.

use std::str;

use kx_module_api::is_hidden;

use super::chains::Chains;
use super::clean::{largest_cap, most};
use super::forget::{Blocked, Mark};
use super::table::{Id, Word, Words};
use super::{LearnConfig, LearnFormatError, Learner, fits_text};
use crate::pack::layout::{MAX_CHAIN, MIN_CHAIN};
use crate::reader::Reader;
use crate::track::could_learn;

/// The first bytes of every learned file.
pub const MAGIC: [u8; 4] = *b"KXL1";
/// The format version this crate reads and writes.
pub const VERSION: u16 = 1;
/// The longest spelling, in UTF-8 bytes, as its length is one byte.
pub(crate) const MAX_TEXT: usize = u8::MAX as usize;
/// The most chain tables a byte form holds: one per length from `MIN_CHAIN` to `MAX_CHAIN`.
const MAX_TABLES: u8 = MAX_CHAIN - MIN_CHAIN + 1;
/// The most `max_chains` a learner takes: at its peak, a table's length still fits the byte form's u32.
pub const MAX_LEARNED_CHAINS: usize = largest_cap(u32::MAX as usize);

impl Learner {
    /// The learner's byte form, which `from_bytes` reads back equal; every spelling fits `MAX_TEXT`.
    #[must_use]
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut out = MAGIC.to_vec();
        out.extend(VERSION.to_le_bytes());
        put_text(&mut out, &self.iso);
        put_len(&mut out, self.words.len());
        for (id, word) in self.words.iter() {
            out.extend(id.to_le_bytes());
            put_text(&mut out, &word.text);
            put_use(&mut out, word.count, word.day);
        }
        out.push(u8::try_from(self.chains.len()).unwrap_or(MAX_TABLES));
        for table in &self.chains {
            put_len(&mut out, table.len());
            for at in 0..table.len() {
                let (key, count, day) = table.entry(at);
                for id in key {
                    out.extend(id.to_le_bytes());
                }
                put_use(&mut out, count, day);
            }
        }
        put_len(&mut out, self.blocked.len());
        for (text, mark) in self.blocked.iter() {
            put_text(&mut out, text);
            put_use(&mut out, mark.uses, mark.day);
        }
        out
    }

    /// The learner in `bytes`, with `cfg`: tables the settings do not keep are left out, and caps lowered since are
    /// cleaned down to.
    ///
    /// # Errors
    /// `LearnFormatError` says what is wrong when `bytes` is not a sound byte form of this version.
    pub fn from_bytes(bytes: &[u8], cfg: LearnConfig) -> Result<Self, LearnFormatError> {
        let mut r = Reader::new(bytes, LearnFormatError::Truncated);
        if r.array()? != MAGIC {
            return Err(LearnFormatError::BadMagic);
        }
        let version = r.u16()?;
        if version != VERSION {
            return Err(LearnFormatError::BadVersion(version));
        }
        let iso = r.text().map_err(|e| match e {
            LearnFormatError::Truncated => e,
            _ => LearnFormatError::BadIso,
        })?;
        let mut learner = Self::new(iso, cfg)?;
        let (words, old, dropped) = read_words(&mut r, &learner)?;
        learner.words = words;
        read_chains(&mut r, &mut learner, old.as_deref(), &dropped)?;
        learner.blocked = read_blocked(&mut r, &learner.words)?;
        if !r.done() {
            return Err(LearnFormatError::BadEntry);
        }
        learner.blocked.trim(learner.cfg.max_words);
        learner.clean();
        Ok(learner)
    }
}

/// The words read back, their old ids when renumbered, and the ids of the words dropped.
type WordsRead = (Words, Option<Vec<Id>>, Vec<Id>);

/// The words, each under its id, their old ids when renumbered, and the ids of the words the learning rule
/// (`could_learn`) drops; `BadEntry` when two share an id or a spelling.
fn read_words(r: &mut Read<'_>, learner: &Learner) -> Result<WordsRead, LearnFormatError> {
    let (mut words, mut dropped) = (Vec::new(), Vec::new());
    for _ in 0..r.u32()? {
        let id = r.u16()?;
        let text = r.text()?;
        let (count, day) = r.counted()?;
        if !could_learn(text) {
            dropped.push(id);
            continue;
        }
        let text = text.into();
        words.push((id, Word { text, count, day }));
    }
    let (cap, casing) = (learner.cfg.max_words, learner.casing());
    let old = renumber(&mut words, most(cap))?;
    let words = Words::load(cap, casing, words).ok_or(LearnFormatError::BadEntry)?;
    Ok((words, old, dropped))
}

/// Numbers `words` 0, 1, … in id order when an id reaches `slots`, as after a `max_words` lowered since the save, so
/// the table holds no slot ahead of its words; gives their old ids then. `BadEntry` when two share an id.
fn renumber(words: &mut [(Id, Word)], slots: usize) -> Result<Option<Vec<Id>>, LearnFormatError> {
    if words.iter().all(|w| usize::from(w.0) < slots) {
        return Ok(None);
    }
    words.sort_unstable_by_key(|w| w.0);
    let old: Vec<Id> = words.iter().map(|w| w.0).collect();
    if old.windows(2).any(|p| p.first() == p.last()) {
        return Err(LearnFormatError::BadEntry);
    }
    for (id, word) in (0..=Id::MAX).zip(words.iter_mut()) {
        word.0 = id;
    }
    Ok(Some(old))
}

/// Word `id`'s id after `renumber`, given the old ids it renumbered, if any; `None` for an id no word held.
fn moved(old: Option<&[Id]>, id: Id) -> Option<Id> {
    let Some(old) = old else {
        return Some(id);
    };
    old.binary_search(&id)
        .ok()
        .and_then(|at| Id::try_from(at).ok())
}

/// The chain tables into `learner`, keeping those its settings hold, with ids `renumber` moved and none naming a
/// `dropped` word; `BadEntry` for too many tables, or a chain out of order or naming a missing word.
fn read_chains(
    r: &mut Read<'_>,
    learner: &mut Learner,
    old: Option<&[Id]>,
    dropped: &[Id],
) -> Result<(), LearnFormatError> {
    let tables = r.u8()?;
    if tables > MAX_TABLES {
        return Err(LearnFormatError::BadEntry);
    }
    let widths = (MIN_CHAIN..).take(usize::from(tables)).map(usize::from);
    for (at, width) in widths.enumerate() {
        let mut table = Chains::new(width, learner.cfg.max_chains);
        for _ in 0..r.u32()? {
            let key = (0..width)
                .map(|_| r.u16())
                .collect::<Result<Vec<Id>, _>>()?;
            let (count, day) = r.counted()?;
            if key.iter().any(|id| dropped.contains(id)) {
                continue;
            }
            let key: Option<Vec<Id>> = key.iter().map(|&id| moved(old, id)).collect();
            let known = key.filter(|k| k.iter().all(|&id| learner.words.get(id).is_some()));
            if !known.is_some_and(|key| table.push(&key, count, day)) {
                return Err(LearnFormatError::BadEntry);
            }
        }
        if let Some(slot) = learner.chains.get_mut(at) {
            *slot = table;
        }
    }
    Ok(())
}

/// The blocklist, without the entries `could_block` refuses; `BadEntry` when out of order, or for a word also in
/// `words`.
fn read_blocked(r: &mut Read<'_>, words: &Words) -> Result<Blocked, LearnFormatError> {
    let mut blocked = Blocked::default();
    let mut last = None;
    for _ in 0..r.u32()? {
        let text = r.text()?;
        let mark = Mark {
            uses: r.u16()?,
            day: r.u32()?,
        };
        if last.is_some_and(|l| l >= text) || words.find(text).is_some() {
            return Err(LearnFormatError::BadEntry);
        }
        if could_block(text) {
            blocked.put(text.into(), mark);
        }
        last = Some(text);
    }
    Ok(blocked)
}

/// True when the blocklist may keep the pack spelling `text` it reads back: it fits the byte form and holds no hidden
/// mark. An entry only hides a word and never types one, so the typed-word rule does not apply.
fn could_block(text: &str) -> bool {
    fits_text(text) && !text.chars().any(is_hidden)
}

/// Appends `text` after its one-byte length.
fn put_text(out: &mut Vec<u8>, text: &str) {
    out.push(u8::try_from(text.len()).unwrap_or(u8::MAX));
    out.extend(text.as_bytes());
}

/// Appends a count of entries; the caps keep it within a u32.
fn put_len(out: &mut Vec<u8>, len: usize) {
    out.extend(u32::try_from(len).unwrap_or(u32::MAX).to_le_bytes());
}

/// Appends an entry's count and last-used day.
fn put_use(out: &mut Vec<u8>, count: u16, day: u32) {
    out.extend(count.to_le_bytes());
    out.extend(day.to_le_bytes());
}

/// The learner's reader: texts and counted entries on top of the shared one.
type Read<'b> = Reader<'b, LearnFormatError>;

impl<'b> Read<'b> {
    /// The next text after its one-byte length; `BadEntry` when empty or not UTF-8.
    fn text(&mut self) -> Result<&'b str, LearnFormatError> {
        let len = usize::from(self.u8()?);
        let text = str::from_utf8(self.take(len)?).map_err(|_| LearnFormatError::BadEntry)?;
        Some(text)
            .filter(|t| !t.is_empty())
            .ok_or(LearnFormatError::BadEntry)
    }

    /// The next count and last-used day; `BadEntry` for a count of 0, as an entry holds at least one use.
    fn counted(&mut self) -> Result<(u16, u32), LearnFormatError> {
        let (count, day) = (self.u16()?, self.u32()?);
        if count == 0 {
            return Err(LearnFormatError::BadEntry);
        }
        Ok((count, day))
    }
}
