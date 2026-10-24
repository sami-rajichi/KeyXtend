//! Folding for loose matching (spec §6 "Matching", ADR-0032 decision 5): `ecole` finds `école` and `ان` finds `أن`,
//! while the chip still types the stored spelling. Every mapping lives in `fold.toml`.

use std::collections::{BTreeMap, HashMap};
use std::sync::LazyLock;

use serde::Deserialize;
use thiserror::Error;

/// The fold tables, compiled in so they are never read from disk.
const TABLE: &str = include_str!("fold.toml");

/// The tables, parsed once; a broken table would fold case only, and a test keeps it sound.
static FOLDS: LazyLock<Folds> = LazyLock::new(|| Folds::parse(TABLE).unwrap_or_default());

/// `word` in lower case, with the characters `fold.toml` lists dropped or replaced: accents, harakat and tatweel go,
/// and the alef forms and ى fold. Folding a folded word changes nothing.
#[must_use]
pub fn fold(word: &str) -> String {
    FOLDS.apply(word)
}

/// Why a fold table is unusable.
#[derive(Clone, Debug, PartialEq, Eq, Error)]
enum FoldTableError {
    /// The table is not TOML of the expected shape, or a key is not one character.
    #[error("the fold table does not parse: {0}")]
    Parse(String),
    /// A character is listed twice, in one script or across scripts.
    #[error("U+{:04X} is listed twice", u32::from(*.0))]
    Twice(char),
    /// A character maps to itself.
    #[error("U+{:04X} maps to itself", u32::from(*.0))]
    Itself(char),
    /// A key is not lower case, so a lower-cased word never meets it.
    #[error("U+{:04X} is not lower case", u32::from(*.0))]
    Upper(char),
    /// A character maps to text that would fold again, so folding twice would differ.
    #[error("U+{:04X} maps to text that folds again", u32::from(*.0))]
    Chained(char),
}

/// One script's section of the table.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Script {
    /// The characters removed.
    #[serde(default)]
    drop: Vec<char>,
    /// The characters replaced, each with its replacement.
    #[serde(default)]
    map: BTreeMap<char, String>,
}

/// Each listed character and what it becomes: `None` drops it.
#[derive(Debug, Default)]
struct Folds(HashMap<char, Option<String>>);

impl Folds {
    /// Reads and checks a fold table from `text`.
    fn parse(text: &str) -> Result<Self, FoldTableError> {
        let scripts: BTreeMap<String, Script> =
            toml::from_str(text).map_err(|e| FoldTableError::Parse(e.to_string()))?;
        let mut all = HashMap::new();
        for script in scripts.into_values() {
            let drops = script.drop.into_iter().map(|c| (c, None));
            let maps = script.map.into_iter().map(|(c, to)| (c, Some(to)));
            for (c, to) in drops.chain(maps) {
                if !is_lower(c) {
                    return Err(FoldTableError::Upper(c));
                }
                if to.as_deref() == Some(c.encode_utf8(&mut [0; 4])) {
                    return Err(FoldTableError::Itself(c));
                }
                if all.insert(c, to).is_some() {
                    return Err(FoldTableError::Twice(c));
                }
            }
        }
        let folds = Self(all);
        folds.check_settled()?;
        Ok(folds)
    }

    /// Refuses a replacement that is not lower case or holds a listed character, as folding it again would change it.
    fn check_settled(&self) -> Result<(), FoldTableError> {
        let mut chained = self.0.iter().filter_map(|(c, to)| {
            let to = to.as_deref()?;
            let moves = to.chars().any(|t| !is_lower(t) || self.0.contains_key(&t));
            moves.then_some(*c)
        });
        chained
            .next()
            .map_or(Ok(()), |c| Err(FoldTableError::Chained(c)))
    }

    /// `word` lower-cased, then each character dropped, replaced or kept.
    fn apply(&self, word: &str) -> String {
        let mut out = String::with_capacity(word.len());
        for c in word.to_lowercase().chars() {
            match self.0.get(&c) {
                None => out.push(c),
                Some(Some(to)) => out.push_str(to),
                Some(None) => {}
            }
        }
        out
    }
}

/// True when lower-casing leaves `c` as it is.
fn is_lower(c: char) -> bool {
    c.to_lowercase().eq([c])
}

#[cfg(test)]
mod tests;
