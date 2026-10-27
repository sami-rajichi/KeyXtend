//! The saved emails' byte form, little-endian: a magic, a version, the count, then each email after its length, most
//! recent first, and nothing else. The module seals it; reading checks every length against what is left.

use std::str;

use super::{Emails, looks_like};
use crate::reader::Reader;

/// The first bytes of every saved emails file.
pub const MAGIC: [u8; 4] = *b"KXE1";
/// The format version this crate reads and writes.
pub const VERSION: u16 = 1;

/// Why saved emails could not be read.
#[derive(Copy, Clone, Debug, PartialEq, Eq, thiserror::Error)]
pub enum EmailFormatError {
    /// The bytes do not start with the emails file's magic.
    #[error("the file is not a saved emails file")]
    BadMagic,
    /// The byte form's version is not this crate's.
    #[error("saved emails version {0} is not supported")]
    BadVersion(u16),
    /// The bytes end before what they announce.
    #[error("the saved emails file ends early")]
    Truncated,
    /// Bytes follow the last entry.
    #[error("the saved emails file has bytes after its last entry")]
    Trailing,
}

impl Emails {
    /// The emails' byte form, which `from_bytes` reads back equal; an email too long for its length is left out.
    #[must_use]
    pub fn to_bytes(&self) -> Vec<u8> {
        let kept: Vec<&[u8]> = self
            .list
            .iter()
            .map(String::as_bytes)
            .filter(|e| u16::try_from(e.len()).is_ok())
            .collect();
        let mut out = MAGIC.to_vec();
        out.extend(VERSION.to_le_bytes());
        out.extend(u32::try_from(kept.len()).unwrap_or(u32::MAX).to_le_bytes());
        for email in kept {
            out.extend(u16::try_from(email.len()).unwrap_or(u16::MAX).to_le_bytes());
            out.extend(email);
        }
        out
    }

    /// The emails in `bytes`, at most `max`, the most recent kept; an entry that is not UTF-8 or no email is dropped,
    /// and the others stay.
    ///
    /// # Errors
    /// `EmailFormatError` says what is wrong when `bytes` is not a sound byte form of this version.
    pub fn from_bytes(bytes: &[u8], max: usize) -> Result<Self, EmailFormatError> {
        let mut r = Reader::new(bytes, EmailFormatError::Truncated);
        if r.take(MAGIC.len())? != MAGIC {
            return Err(EmailFormatError::BadMagic);
        }
        let version = r.u16()?;
        if version != VERSION {
            return Err(EmailFormatError::BadVersion(version));
        }
        let count = r.u32()?;
        let mut list = Vec::new();
        for _ in 0..count {
            let len = usize::from(r.u16()?);
            let text = str::from_utf8(r.take(len)?).ok().filter(|t| looks_like(t));
            list.extend(text.map(str::to_owned));
        }
        if !r.done() {
            return Err(EmailFormatError::Trailing);
        }
        Ok(Self::new(&list, max))
    }
}

#[cfg(test)]
mod tests;
