//! The saved emails' byte form, little-endian: a magic, a version, the count, then each email after its length, most
//! recent first, and nothing else. The module seals it; reading checks every length against what is left.

use std::str;

use super::{Emails, looks_like};

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
    /// An entry is not UTF-8 or no email, or bytes follow the end.
    #[error("a saved email entry is malformed")]
    BadEntry,
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

    /// The emails in `bytes`, at most `max`, the most recent kept.
    ///
    /// # Errors
    /// `EmailFormatError` says what is wrong when `bytes` is not a sound byte form of this version.
    pub fn from_bytes(bytes: &[u8], max: usize) -> Result<Self, EmailFormatError> {
        let mut r = Reader { bytes };
        if r.take(MAGIC.len())? != MAGIC {
            return Err(EmailFormatError::BadMagic);
        }
        let version = u16::from_le_bytes(r.array()?);
        if version != VERSION {
            return Err(EmailFormatError::BadVersion(version));
        }
        let count = u32::from_le_bytes(r.array()?);
        let mut list = Vec::new();
        for _ in 0..count {
            let len = usize::from(u16::from_le_bytes(r.array()?));
            let text = str::from_utf8(r.take(len)?).map_err(|_| EmailFormatError::BadEntry)?;
            if !looks_like(text) {
                return Err(EmailFormatError::BadEntry);
            }
            list.push(text.to_owned());
        }
        if !r.bytes.is_empty() {
            return Err(EmailFormatError::BadEntry);
        }
        Ok(Self::new(&list, max))
    }
}

/// Reads the byte form front to back; every read checks what is left.
struct Reader<'b> {
    /// The bytes not read yet.
    bytes: &'b [u8],
}

impl<'b> Reader<'b> {
    /// The next `n` bytes; `Truncated` past the end.
    fn take(&mut self, n: usize) -> Result<&'b [u8], EmailFormatError> {
        let (head, tail) = (self.bytes.split_at_checked(n)).ok_or(EmailFormatError::Truncated)?;
        self.bytes = tail;
        Ok(head)
    }

    /// The next `N` bytes.
    fn array<const N: usize>(&mut self) -> Result<[u8; N], EmailFormatError> {
        self.take(N)?
            .try_into()
            .map_err(|_| EmailFormatError::Truncated)
    }
}

#[cfg(test)]
mod tests;
