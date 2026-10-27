//! Reads Tatoeba's sentence files (`<lang>_sentences_detailed.tsv.bz2`) as a stream: one line per entry, its fields
//! id, language, text, username, added and modified, split by tabs.

use std::io::Read;

use sha2::{Digest, Sha256};

use super::lines::Lines;
use super::manifest::PER_MILLE;

/// Separates a line's fields.
const TAB: char = '\t';
/// The field of the sentence's id.
const ID: usize = 0;
/// The field of its text.
const TEXT: usize = 2;
/// The field of its author's username.
const USER: usize = 3;
/// The bytes of the hash read as the share's number.
const HASH_BYTES: usize = 8;

/// One entry of the file.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Entry<'a> {
    /// Its id.
    pub(crate) id: &'a str,
    /// Its text, one or more sentences.
    pub(crate) text: &'a str,
    /// Its author's username.
    pub(crate) user: &'a str,
}

/// Gives `each` up to `max` entries of the bzip2 stream `reader`; returns how many lines were read.
pub(crate) fn read(
    reader: impl Read,
    max: u64,
    each: &mut dyn FnMut(Entry<'_>),
) -> Result<u64, String> {
    let mut lines = Lines::new(reader);
    let mut read = 0;
    while read < max {
        let Some(buf) = lines.line()? else {
            break;
        };
        read += 1;
        let line = String::from_utf8_lossy(buf);
        let fields: Vec<&str> = line.trim_end().split(TAB).collect();
        let (Some(id), Some(text), Some(user)) =
            (fields.get(ID), fields.get(TEXT), fields.get(USER))
        else {
            return Err(format!("line {read} has fewer than {} fields", USER + 1));
        };
        each(Entry { id, text, user });
    }
    Ok(read)
}

/// True when entry `id` is held back for the meter: the first bytes of its SHA-256, as a number, fall in the first
/// `per_mille` of each thousand, so the same ids are always held.
pub(crate) fn held(id: &str, per_mille: u64) -> bool {
    let hash = Sha256::digest(id.as_bytes());
    let mut first = [0; HASH_BYTES];
    first.copy_from_slice(hash.get(..HASH_BYTES).unwrap_or(&[0; HASH_BYTES]));
    u64::from_le_bytes(first) % PER_MILLE < per_mille
}
