//! Secret bytes a frame lends when written and keeps when read, wiped when dropped: passwords, notebook parts and sound.

use std::fmt;
use std::ops::Deref;
use zeroize::Zeroizing;

/// Bytes a frame lends when it is written and keeps when it is read; kept bytes are wiped when dropped.
#[derive(Clone)]
pub enum Wiped<'a> {
    /// Bytes the writer lends.
    Lent(&'a [u8]),
    /// Bytes read from a frame.
    Kept(Zeroizing<Vec<u8>>),
}

impl Deref for Wiped<'_> {
    type Target = [u8];

    fn deref(&self) -> &[u8] {
        match self {
            Self::Lent(bytes) => bytes,
            Self::Kept(bytes) => bytes,
        }
    }
}

impl PartialEq for Wiped<'_> {
    fn eq(&self, other: &Self) -> bool {
        **self == **other
    }
}

impl Eq for Wiped<'_> {}

impl fmt::Debug for Wiped<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("Wiped").field(&self.len()).finish()
    }
}
