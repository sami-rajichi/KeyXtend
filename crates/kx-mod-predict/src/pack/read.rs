//! The pack reader: it checks the header, the sections and the lexicon's checksum once, then reads in place. A mapped
//! file could change under it, so every offset and length read from the pack is checked again where it is used.

use std::fmt;
use std::ops::Range;
use std::str;

use fst::automaton::{Automaton, Str};
use fst::{IntoStreamer, Map};
use kx_module_api::text::Casing;

use super::layout::{MIN_CHAIN, OFFSET_LEN, RUN_SHIFT, fits_spelling, usize_at};
use super::{Flags, PackFormatError, WordId};
use crate::script::{self, Script};

mod chains;
mod head;
mod keys;

use chains::Chain;
use head::Head;
use keys::{Completions, first_at, successor};

/// The share of the words with a script that a pack's main script holds: all, as its builder dropped the others.
const KEPT: f64 = 1.0;

/// One language's word pack, read in place from its bytes.
pub struct Pack<'a> {
    /// The whole pack.
    bytes: &'a [u8],
    /// The header and section table, read at parse.
    head: Head,
    /// The lexicon, its checksum checked at parse.
    lexicon: Map<&'a [u8]>,
    /// Its main script, found at parse.
    script: Option<Script>,
}

impl fmt::Debug for Pack<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let head = &self.head;
        f.debug_struct("Pack")
            .field("iso", &head.iso)
            .field("words", &head.words)
            .finish_non_exhaustive()
    }
}

impl<'a> Pack<'a> {
    /// Reads the pack in `bytes`, checking its header, every section's place and size, the spellings and the
    /// lexicon's checksum; the big sections are not copied.
    ///
    /// # Errors
    /// `PackFormatError` says what is wrong when the bytes are not a sound pack of this version.
    pub fn parse(bytes: &'a [u8]) -> Result<Self, PackFormatError> {
        let head = Head::read(bytes)?;
        let bad = PackFormatError::BadSection;
        let lexicon = Map::new(head.lexicon.of(bytes)).map_err(|_| bad)?;
        lexicon.as_fst().verify().map_err(|_| bad)?;
        let count = usize::try_from(head.words).map_err(|_| bad)?;
        if lexicon.len() > count || head.freq.len != count {
            return Err(bad);
        }
        check_spellings(head.spellings.of(bytes), count)?;
        let mut pack = Self {
            bytes,
            head,
            lexicon,
            script: None,
        };
        for n in (MIN_CHAIN..).take(pack.head.chains.len()) {
            pack.chain(n).ok_or(bad)?.check()?;
        }
        pack.script = pack.main_script();
        Ok(pack)
    }

    /// The language code, as the pack's header names it.
    #[must_use]
    pub fn iso(&self) -> &str {
        &self.head.iso
    }

    /// How the pack's language cases its words, which its keys were folded with.
    #[must_use]
    pub fn casing(&self) -> &'static Casing {
        Casing::of(self.iso())
    }

    /// How many words the pack holds; ids run from 0 to this, exclusive.
    #[must_use]
    pub fn word_count(&self) -> u32 {
        self.head.words
    }

    /// The longest chain: `layout::NO_CHAINS` without chains, else up to `layout::MAX_CHAIN`.
    #[must_use]
    pub fn max_chain(&self) -> u8 {
        self.head.max_chain
    }

    /// What the header says of the pack.
    #[must_use]
    pub fn flags(&self) -> Flags {
        self.head.flags
    }

    /// Word `id`'s correct spelling; empty for an unknown id or a damaged entry.
    #[must_use]
    pub fn spelling(&self, id: WordId) -> &'a str {
        self.spelling_bytes(id)
            .and_then(|b| str::from_utf8(b).ok())
            .unwrap_or_default()
    }

    /// Word `id`'s frequency byte, Zipf × `STEPS_PER_ZIPF`; 0 for an unknown id.
    #[must_use]
    pub fn freq(&self, id: WordId) -> u8 {
        self.index(id)
            .and_then(|i| self.head.freq.of(self.bytes).get(i).copied())
            .unwrap_or_default()
    }

    /// The ids of every word that folds to exactly `folded`, highest frequency first; empty when none does.
    #[must_use]
    pub fn run(&self, folded: &str) -> Range<WordId> {
        self.lexicon
            .get(folded)
            .and_then(|v| run(v, self.head.words))
            .unwrap_or_default()
    }

    /// The main script its builder kept, found by the builder's rule (`script::main`) over every spelling; `None` for
    /// a list the builder left mixed or with no letter.
    #[must_use]
    pub fn script(&self) -> Option<Script> {
        self.script
    }

    /// The ids of every word whose folded key starts with `folded_prefix`: one range, as ids follow their keys.
    #[must_use]
    pub fn span(&self, folded_prefix: &str) -> Range<WordId> {
        let (prefix, words) = (folded_prefix.as_bytes(), self.head.words);
        let from = first_at(&self.lexicon, words, prefix);
        let to = successor(prefix).map_or(words, |next| first_at(&self.lexicon, words, &next));
        from..to.max(from)
    }

    /// The ids of every word whose folded key starts with `folded_prefix`, key by key in byte order.
    pub fn completions<'s>(&'s self, folded_prefix: &'s str) -> impl Iterator<Item = WordId> + 's {
        let keys = Str::new(folded_prefix).starts_with();
        let stream = self.lexicon.search(keys).into_stream();
        Completions {
            stream,
            run: Range::default(),
            keys_left: self.head.words,
            words: self.head.words,
        }
    }

    /// The words that followed `context`, its 1 to `max_chain − 1` previous ids, highest score first, each with its
    /// score; empty when the context is unknown or too long.
    pub fn followers(&self, context: &[WordId]) -> impl Iterator<Item = (WordId, u8)> + use<'a> {
        let chain = u8::try_from(context.len() + 1)
            .ok()
            .and_then(|n| self.chain(n));
        let found = chain.and_then(|c| Some((c, c.find(context)?)));
        found
            .into_iter()
            .flat_map(|(c, range)| range.filter_map(move |f| c.follower(f)))
    }

    /// The pack's header over other `bytes` of the same length, as a mapped file changed after parse would be.
    #[cfg(all(test, feature = "build"))]
    pub(super) fn altered<'b>(&self, bytes: &'b [u8]) -> Option<Pack<'b>> {
        let lexicon = Map::new(self.head.lexicon.of(bytes)).ok()?;
        Some(Pack {
            bytes,
            head: self.head.clone(),
            lexicon,
            script: self.script,
        })
    }

    /// The builder's main script over every spelling, with `KEPT` as its share.
    fn main_script(&self) -> Option<Script> {
        let ids = 0..self.head.words;
        script::main(ids.map(|id| script::of(self.spelling(id))), KEPT)
    }

    /// `id` as an index into the word sections, when it names a word.
    fn index(&self, id: WordId) -> Option<usize> {
        usize::try_from(id)
            .ok()
            .filter(|&i| i < usize::try_from(self.head.words).unwrap_or_default())
    }

    /// Word `id`'s UTF-8 bytes, between its two offsets.
    fn spelling_bytes(&self, id: WordId) -> Option<&'a [u8]> {
        let sec = self.head.spellings.of(self.bytes);
        let at = self.index(id)?.checked_mul(OFFSET_LEN)?;
        let (start, end) = (
            usize_at(sec, at)?,
            usize_at(sec, at.checked_add(OFFSET_LEN)?)?,
        );
        let text_at = usize::try_from(self.head.words)
            .ok()?
            .checked_add(1)?
            .checked_mul(OFFSET_LEN)?;
        sec.get(text_at..)?.get(start..end)
    }

    /// Chain section `n`, when the pack has it.
    fn chain(&self, n: u8) -> Option<Chain<'a>> {
        let span = self
            .head
            .chains
            .get(usize::from(n.checked_sub(MIN_CHAIN)?))?;
        let (cap, words) = (usize::from(self.head.followers), self.head.words);
        Some(Chain {
            sec: span.of(self.bytes),
            n,
            cap,
            words,
        })
    }
}

/// The run a lexicon `value` packs: its first id in the low half and its count in the high half, within `words`.
fn run(value: u64, words: u32) -> Option<Range<WordId>> {
    let first = u32::try_from(value & u64::from(u32::MAX)).ok()?;
    let count = u32::try_from(value >> RUN_SHIFT).ok()?;
    let end = first.checked_add(count)?;
    (count > 0 && end <= words).then_some(first..end)
}

/// `BadUtf8`, `BadSection` or `OutOfBounds` unless the `Words` section holds `count` spellings that fit
/// (`fits_spelling`) and are valid UTF-8, their offsets running from 0 to the text's end.
fn check_spellings(sec: &[u8], count: usize) -> Result<(), PackFormatError> {
    let out = PackFormatError::OutOfBounds;
    let text_at = count
        .checked_add(1)
        .and_then(|n| n.checked_mul(OFFSET_LEN))
        .ok_or(out)?;
    let text = sec.get(text_at..).ok_or(out)?;
    let offset = |i: usize| {
        i.checked_mul(OFFSET_LEN)
            .and_then(|at| usize_at(sec, at))
            .ok_or(out)
    };
    if offset(0)? != 0 || offset(count)? != text.len() {
        return Err(PackFormatError::BadSection);
    }
    for i in 0..count {
        let word = text.get(offset(i)?..offset(i + 1)?).ok_or(out)?;
        if !fits_spelling(word.len()) {
            return Err(PackFormatError::BadSection);
        }
        str::from_utf8(word).map_err(|_| PackFormatError::BadUtf8)?;
    }
    Ok(())
}
