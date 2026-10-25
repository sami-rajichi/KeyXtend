//! The spelling dictionary (spec §6 "Engine"): the forms it accepts for a token (`london` gives `London`, `jusqu`
//! gives `jusqu'`), its own prefixes, and its stems joined by an apostrophe (`aujourd'hui`).

use std::collections::{BTreeSet, HashSet};
use std::path::Path;

use kx_mod_predict::Casing;
use spellbook::Dictionary;

use super::{WordsError, io_error};

/// The affix-file keyword of a prefix line; suffixes are left out, as Arabic ones (`انا`, `ان`) are words too.
const PREFIX_KEY: &str = "PFX";
/// The cross-product marks of an affix header line (`PFX A Y 1`).
const CROSS: [&str; 2] = ["Y", "N"];
/// Fields of an affix header line: keyword, flag, cross-product mark and count.
const HEADER_FIELDS: usize = 4;
/// The field of an affix entry that holds the added text (`PFX A 0 ال .`).
const ADD_FIELD: usize = 3;
/// What an affix entry writes for "nothing".
const EMPTY: &str = "0";
/// Separates a stem or an affix from its flags.
const FLAGS: char = '/';

/// A Hunspell dictionary read by `spellbook`, with the stems and prefixes its rules need.
pub(crate) struct Dict {
    /// The dictionary.
    words: Dictionary,
    /// The stems that start upper case or hold the apostrophe: the ones forms and joins look up.
    stems: BTreeSet<String>,
    /// The text each prefix adds.
    prefixes: HashSet<String>,
    /// The apostrophe of elided and joined words.
    apostrophe: char,
}

impl Dict {
    /// The dictionary in the affix text `aff` and the word text `dic`.
    pub(crate) fn parse(aff: &str, dic: &str, apostrophe: char) -> Result<Self, String> {
        let words = Dictionary::new(aff, dic).map_err(|e| e.to_string())?;
        let marked =
            |s: &&str| s.contains(apostrophe) || s.chars().next().is_some_and(char::is_uppercase);
        let stems = dic
            .lines()
            .skip(1)
            .map(stem)
            .filter(marked)
            .map(String::from)
            .collect();
        Ok(Self {
            words,
            stems,
            prefixes: aff.lines().filter_map(prefix).map(String::from).collect(),
            apostrophe,
        })
    }

    /// The dictionary in the files `aff` and `dic`.
    pub(crate) fn load(aff: &Path, dic: &Path, apostrophe: char) -> Result<Self, WordsError> {
        let read = |path: &Path| std::fs::read_to_string(path).map_err(|e| io_error(path, &e));
        Self::parse(&read(aff)?, &read(dic)?, apostrophe)
            .map_err(|why| WordsError::Dict(format!("{}: {why}", dic.display())))
    }

    /// Each form of `word` the dictionary accepts, its own first: as it is, its capital as `casing` writes it, then
    /// its elision. A capital beside an accepted word counts only when it is a stem of its own (`I` beside `i`).
    pub(crate) fn forms(&self, word: &str, casing: &Casing) -> Vec<String> {
        let mut forms = Vec::new();
        let plain = self.words.check(word);
        if plain {
            forms.push(word.to_string());
        }
        let own = |c: &String| !plain || self.stems.contains(c);
        forms.extend(capitalise(word, casing).filter(|c| self.words.check(c) && own(c)));
        let elided = format!("{word}{}", self.apostrophe);
        if self.stems.contains(&elided) && self.words.check(&elided) {
            forms.push(elided);
        }
        forms
    }

    /// True when `word` is the text one of the dictionary's prefixes adds.
    pub(crate) fn is_prefix(&self, word: &str) -> bool {
        self.prefixes.contains(word)
    }

    /// Each accepted stem holding one apostrophe between two pieces, as (stem, left piece, right piece).
    pub(crate) fn joins(&self) -> impl Iterator<Item = (&str, &str, &str)> {
        self.stems.iter().filter_map(|stem| {
            let (left, right) = stem.split_once(self.apostrophe)?;
            let one = !left.is_empty() && !right.is_empty() && !right.contains(self.apostrophe);
            (one && self.words.check(stem)).then_some((stem.as_str(), left, right))
        })
    }
}

/// The stem of a `.dic` line: the text before its flags or its morphology.
fn stem(line: &str) -> &str {
    let word = line.split_whitespace().next().unwrap_or_default();
    word.split(FLAGS).next().unwrap_or_default()
}

/// The text a prefix entry line adds; `None` for a header, an empty prefix or another line.
fn prefix(line: &str) -> Option<&str> {
    let fields: Vec<&str> = line.split_whitespace().collect();
    let key = fields.first()?;
    let header = fields.len() == HEADER_FIELDS
        && fields.get(2).is_some_and(|c| CROSS.contains(c))
        && fields.get(3).is_some_and(|n| n.parse::<usize>().is_ok());
    if *key != PREFIX_KEY || header {
        return None;
    }
    let add = fields.get(ADD_FIELD)?.split(FLAGS).next()?;
    (add != EMPTY && !add.is_empty()).then_some(add)
}

/// `word` with its first letter in capitals as `casing` writes it; `None` when that changes nothing, as in Arabic.
fn capitalise(word: &str, casing: &Casing) -> Option<String> {
    let capital = casing.capital(word);
    (capital != word).then_some(capital)
}
