//! The pack index (ADR-0032): the file the build writes beside the packs, naming each language it shipped, so the
//! platform tells a missing pack from a language that never had one.

use super::fits_iso;
use std::collections::BTreeSet;
use thiserror::Error;

/// Ends each line of the index; a reader also takes `\r\n`.
const LINE_END: char = '\n';

/// Why the index could not be read.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Error)]
pub enum IndexError {
    /// No folder holds an index.
    #[error("the word pack index is missing")]
    Missing,
    /// The index is there but cannot be read.
    #[error("the word pack index cannot be read")]
    Unreadable,
    /// A line is not a language code `fits_iso` takes, or the text is not UTF-8.
    #[error("the word pack index is damaged")]
    Damaged,
}

/// The index of `isos`: each code once, sorted, one per line.
///
/// # Errors
/// `Damaged` when a code fails `fits_iso`.
pub fn index_text(isos: &[&str]) -> Result<String, IndexError> {
    if !isos.iter().all(|iso| fits_iso(iso)) {
        return Err(IndexError::Damaged);
    }
    let sorted: BTreeSet<&str> = isos.iter().copied().collect();
    let mut text = String::new();
    for iso in sorted {
        text.push_str(iso);
        text.push(LINE_END);
    }
    Ok(text)
}

/// The languages the index `bytes` lists.
///
/// # Errors
/// `Damaged` when the bytes are not UTF-8 or a line is not a code `fits_iso` takes.
pub fn read_index(bytes: &[u8]) -> Result<BTreeSet<String>, IndexError> {
    let text = std::str::from_utf8(bytes).map_err(|_| IndexError::Damaged)?;
    text.lines()
        .map(|line| {
            fits_iso(line)
                .then(|| line.to_owned())
                .ok_or(IndexError::Damaged)
        })
        .collect()
}

#[cfg(test)]
mod tests;
