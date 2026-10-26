//! The pack index (ADR-0032): the file the build writes beside the packs, naming each language it shipped and the
//! other layout codes each pack serves, so the platform tells a missing pack from a language that never had one.

use super::fits_iso;
use std::collections::{BTreeMap, BTreeSet};
use thiserror::Error;

/// Ends each line of the index; a reader also takes `\r\n`.
const LINE_END: char = '\n';
/// Parts a pack's code from the layout codes it also serves, on its line: `sh hr bs`.
const SEP: char = ' ';

/// Why the index could not be read.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Error)]
pub enum IndexError {
    /// No folder holds an index.
    #[error("the word pack index is missing")]
    Missing,
    /// The index is there but cannot be read.
    #[error("the word pack index cannot be read")]
    Unreadable,
    /// A code is not one `fits_iso` takes, a layout code is a pack or served twice, or the text is not UTF-8.
    #[error("the word pack index is damaged")]
    Damaged,
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
    /// `Damaged` when a code fails `fits_iso`, or a layout code is a pack or served by two.
    pub fn new(
        packs: &[&str],
        aliases: &BTreeMap<String, Vec<String>>,
    ) -> Result<Self, IndexError> {
        let served = |pack: &str| aliases.get(pack).into_iter().flatten().cloned().collect();
        let packs = packs.iter().map(|p| ((*p).to_owned(), served(p))).collect();
        Self::checked(packs)
    }

    /// The index in `bytes`: one pack per line, then the codes it serves, each after one `SEP`.
    ///
    /// # Errors
    /// `Damaged` when the bytes are not UTF-8, a part is empty, or `new` would refuse the codes.
    pub fn read(bytes: &[u8]) -> Result<Self, IndexError> {
        let text = std::str::from_utf8(bytes).map_err(|_| IndexError::Damaged)?;
        let mut packs: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
        for line in text.lines() {
            let mut parts = line.split(SEP).map(str::to_owned);
            let pack = parts.next().unwrap_or_default();
            packs.entry(pack).or_default().extend(parts);
        }
        Self::checked(packs)
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
        if let Some((pack, _)) = self.packs.get_key_value(iso) {
            return Some(pack);
        }
        let serves = self.packs.iter().find(|(_, served)| served.contains(iso));
        serves.map(|(pack, _)| pack.as_str())
    }

    /// Adds `other`'s packs and the codes they serve, as when several folders hold an index.
    pub fn merge(&mut self, other: Self) {
        for (pack, served) in other.packs {
            self.packs.entry(pack).or_default().extend(served);
        }
    }

    /// `packs` as an index, when every code fits and no layout code is a pack or served twice.
    fn checked(packs: BTreeMap<String, BTreeSet<String>>) -> Result<Self, IndexError> {
        let mut seen = BTreeSet::new();
        for (pack, served) in &packs {
            let sound = |code: &String| fits_iso(code) && !packs.contains_key(code);
            if !fits_iso(pack) || !served.iter().all(|c| sound(c) && seen.insert(c)) {
                return Err(IndexError::Damaged);
            }
        }
        Ok(Self { packs })
    }
}

#[cfg(test)]
mod tests;
