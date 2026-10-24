//! The spelling dictionary check (spec §6 "Engine"): a word is kept as it is, kept with its capital (`london` becomes
//! `London`), or dropped (`ete`).

use std::path::Path;

use spellbook::Dictionary;

use super::{WordsError, io_error};

/// What the dictionary says of one word.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Verdict {
    /// Accepted as it is.
    Plain,
    /// Accepted only with its first letter capitalised, as given here.
    Capital(String),
    /// Not accepted.
    Rejected,
}

/// A Hunspell dictionary read by `spellbook`.
pub(crate) struct Dict(Dictionary);

impl Dict {
    /// The dictionary in the affix text `aff` and the word text `dic`.
    pub(crate) fn parse(aff: &str, dic: &str) -> Result<Self, String> {
        Dictionary::new(aff, dic)
            .map(Self)
            .map_err(|e| e.to_string())
    }

    /// The dictionary in the files `aff` and `dic`.
    pub(crate) fn load(aff: &Path, dic: &Path) -> Result<Self, WordsError> {
        let read = |path: &Path| std::fs::read_to_string(path).map_err(|e| io_error(path, &e));
        Self::parse(&read(aff)?, &read(dic)?)
            .map_err(|why| WordsError::Dict(format!("{}: {why}", dic.display())))
    }

    /// The verdict on `word`: as it is first, then with its first letter capitalised.
    pub(crate) fn check(&self, word: &str) -> Verdict {
        if self.0.check(word) {
            return Verdict::Plain;
        }
        match capitalise(word) {
            Some(capital) if self.0.check(&capital) => Verdict::Capital(capital),
            _ => Verdict::Rejected,
        }
    }
}

/// `word` with its first letter in upper case; `None` when that changes nothing, as in Arabic.
fn capitalise(word: &str) -> Option<String> {
    let mut chars = word.chars();
    let first = chars.next()?;
    let capital: String = first.to_uppercase().chain(chars).collect();
    (capital != word).then_some(capital)
}
