//! Word packs (ADR-0032 decision 2): one file per language, read straight from a memory map. `layout` defines the
//! format, `read` reads it without copying, and `write` (feature `build`) makes it for `cargo xtask words`.

use thiserror::Error;

pub mod layout;
mod read;
#[cfg(feature = "build")]
mod write;

pub use read::Pack;
#[cfg(feature = "build")]
pub use write::{PackWriter, WriteError};

/// A word's place in its pack's word table, sorted by folded key, then by frequency.
pub type WordId = u32;

/// What the header says of the whole pack.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub struct Flags {
    /// The words were kept only where a spelling dictionary knows them, with its capitals.
    pub checked: bool,
    /// The pack has word chains to guess the next word.
    pub chains: bool,
}

impl Flags {
    /// The flags as the header's byte holds them.
    #[must_use]
    pub fn to_byte(self) -> u8 {
        let checked = if self.checked { layout::CHECKED } else { 0 };
        let chains = if self.chains { layout::CHAINS } else { 0 };
        checked | chains
    }

    /// The flags in `byte`; `None` when it holds a flag this version does not know.
    #[must_use]
    pub fn from_byte(byte: u8) -> Option<Self> {
        let checked = byte & layout::CHECKED != 0;
        let chains = byte & layout::CHAINS != 0;
        (byte & !layout::KNOWN_FLAGS == 0).then_some(Self { checked, chains })
    }
}

/// Why a pack was refused.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Error)]
pub enum PackFormatError {
    /// The file does not start with the pack magic.
    #[error("the file is not a word pack")]
    BadMagic,
    /// The pack's format version is not this crate's.
    #[error("word pack version {0} is not supported")]
    BadVersion(u16),
    /// A header field is out of its range.
    #[error("a word pack header field is out of range")]
    BadHeader,
    /// An offset or a length runs past the file or its section.
    #[error("a word pack offset or length runs past its end")]
    OutOfBounds,
    /// A section is unknown, repeated, missing, or does not fit what the header says.
    #[error("a word pack section is unknown, repeated, missing or malformed")]
    BadSection,
    /// A spelling is not valid UTF-8.
    #[error("a word pack spelling is not valid UTF-8")]
    BadUtf8,
}

#[cfg(test)]
mod tests;
