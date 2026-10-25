//! The header and the section table, read and checked once at parse.

use super::super::layout::{
    AT_FLAGS, AT_FOLLOWERS, AT_ISO, AT_ISO_LEN, AT_MAX_CHAIN, AT_SECTIONS, AT_VERSION, AT_WORDS,
    CHAINS_BASE, ENTRY_AT_LEN, ENTRY_AT_OFFSET, ENTRY_LEN, FREQ, HEAD_LEN, ISO_FIELD, LEXICON,
    MAGIC, MAX_CHAIN, MAX_WORDS, MIN_CHAIN, NO_CHAINS, VERSION, WORDS, u8_at, u16_at, u32_at,
    usize_at,
};
use super::super::{Flags, PackFormatError};
use kx_module_api::ports::fits_iso;

/// A section's place in the pack, checked to lie inside the file.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) struct Span {
    /// Its first byte, from the pack's start.
    pub at: usize,
    /// Its length in bytes.
    pub len: usize,
}

impl Span {
    /// The section's bytes in `bytes`; empty if it no longer fits.
    pub fn of(self, bytes: &[u8]) -> &[u8] {
        let end = self.at.saturating_add(self.len);
        bytes.get(self.at..end).unwrap_or_default()
    }
}

/// What the header and the section table say.
#[derive(Clone, Debug)]
pub(super) struct Head {
    /// The language code.
    pub iso: String,
    /// The pack's flags.
    pub flags: Flags,
    /// The word count.
    pub words: u32,
    /// The highest chain length.
    pub max_chain: u8,
    /// The followers kept per context.
    pub followers: u8,
    /// The `Lexicon` section.
    pub lexicon: Span,
    /// The `Words` section.
    pub spellings: Span,
    /// The `Freq` section.
    pub freq: Span,
    /// The chain sections, from `MIN_CHAIN` to `max_chain`.
    pub chains: Vec<Span>,
}

/// The sections the table names, before the required ones are checked.
#[derive(Default)]
struct Found {
    lexicon: Option<Span>,
    spellings: Option<Span>,
    freq: Option<Span>,
    chains: Vec<Option<Span>>,
}

impl Found {
    /// Places section `kind` at `span`; `BadSection` when the kind is unknown or seen before.
    fn put(&mut self, kind: u8, span: Span) -> Result<(), PackFormatError> {
        let slot = match kind {
            LEXICON => &mut self.lexicon,
            WORDS => &mut self.spellings,
            FREQ => &mut self.freq,
            _ => kind
                .checked_sub(CHAINS_BASE)
                .and_then(|n| n.checked_sub(MIN_CHAIN))
                .and_then(|i| self.chains.get_mut(usize::from(i)))
                .ok_or(PackFormatError::BadSection)?,
        };
        match slot.replace(span) {
            Some(_) => Err(PackFormatError::BadSection),
            None => Ok(()),
        }
    }
}

impl Head {
    /// Reads the header and the section table of `bytes`.
    pub fn read(bytes: &[u8]) -> Result<Self, PackFormatError> {
        if bytes.get(..MAGIC.len()) != Some(MAGIC.as_slice()) {
            return Err(PackFormatError::BadMagic);
        }
        let version = u16_at(bytes, AT_VERSION).ok_or(PackFormatError::OutOfBounds)?;
        if version != VERSION {
            return Err(PackFormatError::BadVersion(version));
        }
        let byte = |at| u8_at(bytes, at).ok_or(PackFormatError::OutOfBounds);
        let (max_chain, followers) = (byte(AT_MAX_CHAIN)?, byte(AT_FOLLOWERS)?);
        let words = u32_at(bytes, AT_WORDS).ok_or(PackFormatError::OutOfBounds)?;
        let flags = Flags::from_byte(byte(AT_FLAGS)?).ok_or(PackFormatError::BadHeader)?;
        let chains_fit = if flags.chains {
            (MIN_CHAIN..=MAX_CHAIN).contains(&max_chain) && followers > 0
        } else {
            max_chain == NO_CHAINS
        };
        let words_fit = usize::try_from(words).is_ok_and(|n| n <= MAX_WORDS);
        if !chains_fit || !words_fit {
            return Err(PackFormatError::BadHeader);
        }
        let iso = iso(bytes)?;
        let found = table(bytes, byte(AT_SECTIONS)?)?;
        let missing = PackFormatError::BadSection;
        Ok(Self {
            iso,
            flags,
            words,
            max_chain,
            followers,
            lexicon: found.lexicon.ok_or(missing)?,
            spellings: found.spellings.ok_or(missing)?,
            freq: found.freq.ok_or(missing)?,
            chains: chains(found.chains, max_chain)?,
        })
    }
}

/// The language code: one that `fits_iso` takes, and only zeros after it in its field.
fn iso(bytes: &[u8]) -> Result<String, PackFormatError> {
    let out = PackFormatError::OutOfBounds;
    let len = usize::from(u8_at(bytes, AT_ISO_LEN).ok_or(out)?);
    let field = bytes.get(AT_ISO..AT_ISO + ISO_FIELD).ok_or(out)?;
    let (code, padding) = field
        .split_at_checked(len)
        .ok_or(PackFormatError::BadHeader)?;
    let code = std::str::from_utf8(code).map_err(|_| PackFormatError::BadHeader)?;
    if !fits_iso(code) || padding.iter().any(|&b| b != 0) {
        return Err(PackFormatError::BadHeader);
    }
    Ok(code.to_owned())
}

/// The `count` sections of the table, each inside the file and after the table.
fn table(bytes: &[u8], count: u8) -> Result<Found, PackFormatError> {
    let count = usize::from(count);
    let table_end = HEAD_LEN + count * ENTRY_LEN;
    let chain_kinds = usize::from(MAX_CHAIN - MIN_CHAIN) + 1;
    let mut found = Found {
        chains: vec![None; chain_kinds],
        ..Found::default()
    };
    for entry in (0..count).map(|i| HEAD_LEN + i * ENTRY_LEN) {
        let out = PackFormatError::OutOfBounds;
        let kind = u8_at(bytes, entry).ok_or(out)?;
        let at = usize_at(bytes, entry + ENTRY_AT_OFFSET).ok_or(out)?;
        let len = usize_at(bytes, entry + ENTRY_AT_LEN).ok_or(out)?;
        if at.checked_add(len).is_none_or(|end| end > bytes.len()) {
            return Err(out);
        }
        if at < table_end {
            return Err(PackFormatError::BadSection);
        }
        found.put(kind, Span { at, len })?;
    }
    Ok(found)
}

/// The chain sections from `MIN_CHAIN` to `max_chain`; `BadSection` when one is missing or one is beyond.
fn chains(found: Vec<Option<Span>>, max_chain: u8) -> Result<Vec<Span>, PackFormatError> {
    let mut kept = Vec::new();
    for (n, span) in (MIN_CHAIN..=MAX_CHAIN).zip(found) {
        match (span, n <= max_chain) {
            (Some(span), true) => kept.push(span),
            (None, false) => {}
            _ => return Err(PackFormatError::BadSection),
        }
    }
    Ok(kept)
}
