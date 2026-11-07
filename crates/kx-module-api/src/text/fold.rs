//! Folding for loose matching (spec §6 "Matching", ADR-0032 decision 5): `ecole` finds `école` and `ان` finds `أن`,
//! while the chip still types the stored spelling. Every mapping lives in `fold.toml`.

use std::collections::{BTreeMap, HashMap};
use std::sync::LazyLock;

use serde::Deserialize;
use thiserror::Error;
use zeroize::Zeroize;

use super::Casing;
use crate::Zeroizing;

/// The fold tables, compiled in so they are never read from disk.
const TABLE: &str = include_str!("fold.toml");
/// The table section that folds each apostrophe form to the one apostrophe words keep.
const APOSTROPHE: &str = "apostrophe";

/// The tables, parsed once; a broken table would fold case only, and a test keeps it sound.
static FOLDS: LazyLock<Folds> = LazyLock::new(|| Folds::parse(TABLE).unwrap_or_default());

/// `word` in lower case as `casing` says, with the characters `fold.toml` lists dropped or replaced unless the casing
/// keeps them: accents, harakat and tatweel go, the alef forms and ى fold, and `’` becomes `'`. Folding twice changes
/// nothing.
#[must_use]
pub fn fold(word: &str, casing: &Casing) -> String {
    FOLDS.apply(word, casing)
}

/// `text` folded as `fold_any` folds it, into `out` in place of what it held: `out` is wiped, then sized to the folded
/// length before the first character goes in, so it never moves and no unwiped copy is left.
pub fn fold_any_into(text: &str, out: &mut Zeroizing<String>) {
    FOLDS.apply_into(text, out);
}

/// The apostrophe that elided and joined words keep (`l'`, `don't`), which every other form folds to; `None` only
/// for a broken table, which a test rules out.
#[must_use]
pub fn apostrophe() -> Option<char> {
    FOLDS.apostrophe
}

/// True when `c` is the apostrophe or one of its forms, so the build tool and the tracker split words alike.
#[must_use]
pub fn is_apostrophe(c: char) -> bool {
    let Some(apostrophe) = FOLDS.apostrophe else {
        return false;
    };
    c == apostrophe
        || FOLDS
            .map
            .get(&c)
            .is_some_and(|to| folds_to(to.as_deref(), apostrophe))
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
    /// The apostrophe section drops a form, maps none, or maps them to more than one character.
    #[error("the apostrophe section must map every form to one character")]
    Apostrophe,
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

/// Each listed character and what it becomes, and the apostrophe its section names.
#[derive(Debug, Default)]
struct Folds {
    /// Each listed character and what it becomes: `None` drops it.
    map: HashMap<char, Option<String>>,
    /// The one character the apostrophe section maps every form to.
    apostrophe: Option<char>,
}

impl Folds {
    /// Reads and checks a fold table from `text`.
    fn parse(text: &str) -> Result<Self, FoldTableError> {
        let scripts: BTreeMap<String, Script> =
            toml::from_str(text).map_err(|e| FoldTableError::Parse(e.to_string()))?;
        let apostrophe = scripts.get(APOSTROPHE).map(apostrophe_of).transpose()?;
        let mut map = HashMap::new();
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
                if map.insert(c, to).is_some() {
                    return Err(FoldTableError::Twice(c));
                }
            }
        }
        let folds = Self { map, apostrophe };
        folds.check_settled()?;
        Ok(folds)
    }

    /// Refuses a replacement that is not lower case or holds a listed character, as folding it again would change it.
    fn check_settled(&self) -> Result<(), FoldTableError> {
        let mut chained = self.map.iter().filter_map(|(c, to)| {
            let to = to.as_deref()?;
            let moves = to
                .chars()
                .any(|t| !is_lower(t) || self.map.contains_key(&t));
            moves.then_some(*c)
        });
        chained
            .next()
            .map_or(Ok(()), |c| Err(FoldTableError::Chained(c)))
    }

    /// `word` lower-cased by `casing`, then each character dropped, replaced or kept.
    fn apply(&self, word: &str, casing: &Casing) -> String {
        let mut out = String::with_capacity(word.len());
        let mut buf = [0; char::MAX_LEN_UTF8];
        for c in casing.lower(word).chars() {
            out.push_str(self.piece(c, casing, &mut buf));
        }
        out
    }

    /// `text` lower-cased one character at a time, as Unicode's default casing does but for a final sigma, which the
    /// table folds to sigma anyway; then folded into `out`, wiped and sized first.
    fn apply_into(&self, text: &str, out: &mut String) {
        let root = Casing::root();
        let lowered = || text.chars().flat_map(char::to_lowercase);
        let mut buf = [0; char::MAX_LEN_UTF8];
        let len = lowered().map(|c| self.piece(c, root, &mut buf).len()).sum();
        out.zeroize();
        out.reserve_exact(len);
        for c in lowered() {
            out.push_str(self.piece(c, root, &mut buf));
        }
        buf.zeroize();
    }

    /// What `c`, already in lower case, folds to under `casing`: itself, written in `buf`, its replacement, or nothing.
    fn piece<'a>(
        &'a self,
        c: char,
        casing: &Casing,
        buf: &'a mut [u8; char::MAX_LEN_UTF8],
    ) -> &'a str {
        match self.map.get(&c).filter(|_| !casing.keeps(c)) {
            None => c.encode_utf8(buf),
            Some(Some(to)) => to,
            Some(None) => "",
        }
    }
}

/// The one character every form of the apostrophe section maps to; `Apostrophe` when it drops any, maps none, or
/// maps to text that is not that one character.
fn apostrophe_of(script: &Script) -> Result<char, FoldTableError> {
    let first = script.map.values().next().and_then(|to| to.chars().next());
    let one = first.filter(|&c| script.map.values().all(|to| folds_to(Some(to), c)));
    one.filter(|_| script.drop.is_empty())
        .ok_or(FoldTableError::Apostrophe)
}

/// True when `to` is exactly the character `c`.
fn folds_to(to: Option<&str>, c: char) -> bool {
    let mut chars = to.unwrap_or_default().chars();
    chars.next() == Some(c) && chars.next().is_none()
}

/// True when lower-casing leaves `c` as it is.
fn is_lower(c: char) -> bool {
    c.to_lowercase().eq([c])
}

#[cfg(test)]
mod tests;
