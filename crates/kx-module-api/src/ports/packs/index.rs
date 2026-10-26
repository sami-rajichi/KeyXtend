//! The pack index (ADR-0032): the file the build writes beside the packs, naming each language it shipped and the
//! other layout codes each pack serves, so the platform tells a missing pack from a language that never had one.

use super::{MAX_INDEX_BYTES, fits_iso};
use std::collections::{BTreeMap, BTreeSet};
use thiserror::Error;

/// Ends each line of the index; a reader also takes `\r\n`.
const LINE_END: char = '\n';
/// Parts a pack's code from the layout codes it also serves, on its line: `sh hr`.
const SEP: char = ' ';

/// Why the index could not be read.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Error)]
pub enum IndexError {
    /// No folder holds an index.
    #[error("the word pack index is missing")]
    Missing,
    /// The index is there but cannot be read, or is over `MAX_INDEX_BYTES`.
    #[error("the word pack index cannot be read")]
    Unreadable,
    /// A code is not one `fits_iso` takes, a layout code is a pack or served by two, or the text is not UTF-8.
    #[error("the word pack index is damaged")]
    Damaged,
    /// The index lists no pack, as an empty file does; the build never writes one.
    #[error("the word pack index lists no pack")]
    Empty,
}

/// What the index lists: each shipped pack's code, with the layout codes it serves besides its own.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct PackIndex {
    packs: BTreeMap<String, BTreeSet<String>>,
}

impl PackIndex {
    /// The index of `packs`, each serving the layout codes `aliases` names for it; aliases of an unlisted pack go.
    ///
    /// # Errors
    /// `Damaged` when a code breaks the index's rules (`add`), `Empty` when `packs` is empty.
    pub fn new(
        packs: &[&str],
        aliases: &BTreeMap<String, Vec<String>>,
    ) -> Result<Self, IndexError> {
        let mut index = Self::default();
        for pack in packs {
            let served = aliases.get(*pack).into_iter().flatten();
            index.add(pack, served.map(String::as_str))?;
        }
        index.listed()
    }

    /// The index in `bytes`: one pack per line, then the codes it serves, each after one `SEP`. Each code is checked
    /// as it is read, and the first bad one stops the read.
    ///
    /// # Errors
    /// `Unreadable` past `MAX_INDEX_BYTES`, `Damaged` when the bytes are not UTF-8 or a part breaks the rules (`add`),
    /// `Empty` when no pack is listed.
    pub fn read(bytes: &[u8]) -> Result<Self, IndexError> {
        if !u64::try_from(bytes.len()).is_ok_and(|len| len <= MAX_INDEX_BYTES) {
            return Err(IndexError::Unreadable);
        }
        let text = std::str::from_utf8(bytes).map_err(|_| IndexError::Damaged)?;
        let mut index = Self::default();
        for line in text.lines() {
            let mut parts = line.split(SEP);
            let pack = parts.next().unwrap_or_default();
            index.add(pack, parts)?;
        }
        index.listed()
    }

    /// The index's text: each pack once, sorted, one per line, with the codes it serves after it.
    #[must_use]
    pub fn text(&self) -> String {
        let mut text = String::new();
        for (pack, served) in &self.packs {
            text.push_str(pack);
            for code in served {
                text.push(SEP);
                text.push_str(code);
            }
            text.push(LINE_END);
        }
        text
    }

    /// Each pack's code, sorted.
    pub fn packs(&self) -> impl Iterator<Item = &str> {
        self.packs.keys().map(String::as_str)
    }

    /// The pack that serves layout code `iso`: its own when shipped, else the one that lists it; `None` when none does.
    #[must_use]
    pub fn pack_of(&self, iso: &str) -> Option<&str> {
        match self.packs.get_key_value(iso) {
            Some((pack, _)) => Some(pack),
            None => self.server(iso),
        }
    }

    /// Adds `other`'s packs and the codes they serve, as when several folders hold an index.
    ///
    /// # Errors
    /// `Damaged`, leaving this index as it was, when the two together break the rules (`add`).
    pub fn merge(&mut self, other: &Self) -> Result<(), IndexError> {
        let mut both = self.clone();
        for (pack, served) in &other.packs {
            both.add(pack, served.iter().map(String::as_str))?;
        }
        *self = both;
        Ok(())
    }

    /// Adds `pack` serving `served`, refusing the first code that fails `fits_iso`, a pack that another serves, or a
    /// layout code that is a pack or another pack serves; a pack or code given again changes nothing.
    fn add<'a>(
        &mut self,
        pack: &str,
        served: impl IntoIterator<Item = &'a str>,
    ) -> Result<(), IndexError> {
        if !fits_iso(pack) || self.server(pack).is_some() {
            return Err(IndexError::Damaged);
        }
        self.packs.entry(pack.to_owned()).or_default();
        for code in served {
            let taken =
                self.packs.contains_key(code) || self.server(code).is_some_and(|p| p != pack);
            if !fits_iso(code) || taken {
                return Err(IndexError::Damaged);
            }
            self.packs
                .entry(pack.to_owned())
                .or_default()
                .insert(code.to_owned());
        }
        Ok(())
    }

    /// The pack that serves layout code `code` besides its own, if any.
    fn server(&self, code: &str) -> Option<&str> {
        let serves = self.packs.iter().find(|(_, served)| served.contains(code));
        serves.map(|(pack, _)| pack.as_str())
    }

    /// This index, unless it lists no pack.
    fn listed(self) -> Result<Self, IndexError> {
        if self.packs.is_empty() {
            Err(IndexError::Empty)
        } else {
            Ok(self)
        }
    }
}

#[cfg(test)]
mod tests;
