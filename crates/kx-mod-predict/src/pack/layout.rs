//! The word-pack format v1 (`.kxw`), little-endian, one definition for the reader and the writer. A pack is a header,
//! a section table, then the sections; every number of the format is named here.

use std::mem::size_of;

/// The first bytes of every pack.
pub const MAGIC: [u8; 4] = *b"KXW1";
/// The format version this crate reads and writes.
pub const VERSION: u16 = 1;
/// The longest language code, in ASCII bytes; its field always has this many bytes, zero-padded.
pub const MAX_ISO: usize = 8;

/// The header's version field (u16), after the magic.
pub const AT_VERSION: usize = MAGIC.len();
/// The language code's length (u8).
pub const AT_ISO_LEN: usize = AT_VERSION + size_of::<u16>();
/// The language code, `MAX_ISO` bytes.
pub const AT_ISO: usize = AT_ISO_LEN + size_of::<u8>();
/// The pack's flags (u8): `CHECKED` and `CHAINS`.
pub const AT_FLAGS: usize = AT_ISO + MAX_ISO;
/// The word count (u32).
pub const AT_WORDS: usize = AT_FLAGS + size_of::<u8>();
/// The highest chain length (u8); `NO_CHAINS` when the pack has none.
pub const AT_MAX_CHAIN: usize = AT_WORDS + size_of::<u32>();
/// The followers kept per context (u8).
pub const AT_FOLLOWERS: usize = AT_MAX_CHAIN + size_of::<u8>();
/// The number of sections (u8).
pub const AT_SECTIONS: usize = AT_FOLLOWERS + size_of::<u8>();
/// The header's length; the section table follows it.
pub const HEAD_LEN: usize = AT_SECTIONS + size_of::<u8>();
/// In a section table entry, the section's offset (u32) from the pack's start, after its kind (u8).
pub const ENTRY_AT_OFFSET: usize = size_of::<u8>();
/// In a section table entry, the section's length (u32).
pub const ENTRY_AT_LEN: usize = ENTRY_AT_OFFSET + size_of::<u32>();
/// One section table entry: kind, offset and length.
pub const ENTRY_LEN: usize = ENTRY_AT_LEN + size_of::<u32>();

/// Flag: the words were kept only where a spelling dictionary knows them, with its capitals.
pub const CHECKED: u8 = 1;
/// Flag: the pack has chain sections.
pub const CHAINS: u8 = 1 << 1;
/// Every flag this version knows; a pack with another is refused.
pub const KNOWN_FLAGS: u8 = CHECKED | CHAINS;
/// A word's flag in the `Flags` section: the word is always capitalised.
pub const CAPITAL: u8 = 1;

/// The `Lexicon` section: an `fst` map from folded key bytes to a run, the first id in the low half of the value and
/// the count in the high half. A key's run is the contiguous ids of every word that folds to it.
pub const LEXICON: u8 = 1;
/// The `Words` section: `word count + 1` u32 offsets into the UTF-8 text that follows them. Word `i` spans offsets
/// `i` to `i + 1`.
pub const WORDS: u8 = 2;
/// The `Freq` section: one byte per word, its wordfreq frequency bucket.
pub const FREQ: u8 = 3;
/// The `Flags` section: one byte per word, `CAPITAL` or zero.
pub const FLAGS: u8 = 4;
/// The kind of chain section `n` is this plus `n`. Its contexts of `n − 1` ids are sorted, so a search halves them.
pub const CHAINS_BASE: u8 = 0x10;

/// The highest chain length of a pack without chains: a lone word.
pub const NO_CHAINS: u8 = 1;
/// The shortest chain: a word and the one before it.
pub const MIN_CHAIN: u8 = 2;
/// The longest chain: a word and the four before it, the most `context_words` allows.
pub const MAX_CHAIN: u8 = 5;
/// Bits per word id inside chain sections.
pub const ID_BITS: u32 = 24;
/// Bytes per word id inside chain sections.
pub const ID_BYTES: usize = (ID_BITS / u8::BITS) as usize;
/// The most words a pack holds, as many as its ids can name.
pub const MAX_WORDS: usize = 1 << ID_BITS;
/// The longest spelling, in UTF-8 bytes.
pub const MAX_SPELLING: usize = 128;

/// Bytes per offset in the `Words` section.
pub const OFFSET_LEN: usize = size_of::<u32>();
/// Where a run's count sits in a lexicon value.
pub const RUN_SHIFT: u32 = u32::BITS;
/// A chain section starts with its context count (u32), then its follower count (u32) at this offset.
pub const CHAIN_AT_TOTAL: usize = size_of::<u32>();
/// The context records follow the two counts.
pub const CHAIN_HEAD: usize = CHAIN_AT_TOTAL + size_of::<u32>();
/// Each context record ends with its first follower's index (u32), after its ids.
pub const START_LEN: usize = size_of::<u32>();
/// Each follower is a u32: its id in the low `ID_BITS` and its score above; highest score first per context.
pub const FOLLOWER_LEN: usize = size_of::<u32>();
/// Where a follower's score sits.
pub const SCORE_SHIFT: u32 = ID_BITS;
/// A follower's id bits.
pub const ID_MASK: u32 = (1 << ID_BITS) - 1;

/// One context record's length in chain section `n`: its `n − 1` ids and its start.
#[must_use]
pub fn record_len(n: u8) -> usize {
    usize::from(n).saturating_sub(1) * ID_BYTES + START_LEN
}

/// True when `code` is a language code a pack may carry: 1 to `MAX_ISO` printable ASCII bytes.
#[must_use]
pub fn fits_iso(code: &[u8]) -> bool {
    (1..=MAX_ISO).contains(&code.len()) && code.iter().all(u8::is_ascii_graphic)
}

/// True when a spelling of `len` UTF-8 bytes fits a pack: 1 to `MAX_SPELLING`.
#[must_use]
pub fn fits_spelling(len: usize) -> bool {
    (1..=MAX_SPELLING).contains(&len)
}

/// `N` bytes at `at`, or `None` past the end.
fn array_at<const N: usize>(bytes: &[u8], at: usize) -> Option<[u8; N]> {
    bytes.get(at..at.checked_add(N)?)?.try_into().ok()
}

/// The u8 at `at`.
#[must_use]
pub fn u8_at(bytes: &[u8], at: usize) -> Option<u8> {
    bytes.get(at).copied()
}

/// The little-endian u16 at `at`.
#[must_use]
pub fn u16_at(bytes: &[u8], at: usize) -> Option<u16> {
    array_at(bytes, at).map(u16::from_le_bytes)
}

/// The little-endian u32 at `at`.
#[must_use]
pub fn u32_at(bytes: &[u8], at: usize) -> Option<u32> {
    array_at(bytes, at).map(u32::from_le_bytes)
}

/// The little-endian `ID_BYTES` id at `at`.
#[must_use]
pub fn id_at(bytes: &[u8], at: usize) -> Option<u32> {
    let mut word = [0; size_of::<u32>()];
    word.get_mut(..ID_BYTES)?
        .copy_from_slice(bytes.get(at..at.checked_add(ID_BYTES)?)?);
    Some(u32::from_le_bytes(word))
}

/// The u32 at `at` as a `usize`.
#[must_use]
pub fn usize_at(bytes: &[u8], at: usize) -> Option<usize> {
    usize::try_from(u32_at(bytes, at)?).ok()
}
