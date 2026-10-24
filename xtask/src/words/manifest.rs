//! The manifest `words.toml`: every source file with its hash, the cleaning rules and each language's files.

use std::collections::BTreeMap;

use kx_mod_predict::pack::layout::fits_iso;
use serde::Deserialize;

use super::WordsError;
use crate::devtools::paths::is_relative_plain;

/// The placeholder for the language code in the small lists' path.
const ISO_SLOT: &str = "{iso}";
/// A SHA-256 is 32 bytes, written as two lowercase hex digits each.
pub(crate) const HASH_HEX: usize = 64;
/// The separator of the manifest's paths.
const SEP: char = '/';

/// The whole manifest.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Manifest {
    /// How words are cleaned and reported.
    pub(crate) rules: Rules,
    /// How a missing source is downloaded.
    pub(crate) fetch: FetchConfig,
    /// The languages with only a small list.
    small: Small,
    /// The languages with a full list and a dictionary, by code.
    lang: BTreeMap<String, Lang>,
    /// The URL prefixes the sources come from, by name.
    bases: BTreeMap<String, String>,
    /// Every source file, by its path under the source folder.
    pub(crate) sources: BTreeMap<String, Source>,
}

/// How words are cleaned and reported.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Rules {
    /// A word holding one of these is dropped.
    pub(crate) drop_chars: Vec<char>,
    /// The apostrophe an elided word takes back.
    pub(crate) apostrophe: char,
    /// Frequency bytes per Zipf unit.
    pub(crate) steps_per_zipf: u32,
    /// How many of the most frequent words the coverage line checks.
    pub(crate) coverage_top: usize,
    /// How many rejected words it prints.
    pub(crate) coverage_show: usize,
}

/// How a missing source is downloaded with the system's curl.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct FetchConfig {
    /// Env var naming the folder that holds curl.
    pub(crate) root_env: String,
    /// curl, under that folder.
    pub(crate) curl: String,
    /// The program run where the env var is unset.
    pub(crate) fallback: String,
    /// curl's arguments before the output file and the URL.
    pub(crate) args: Vec<String>,
    /// The extension of a download before its check.
    pub(crate) part: String,
}

/// The languages with only a small list.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Small {
    /// The list's path, with `ISO_SLOT` for the code.
    list: String,
    /// The most words kept.
    max_words: usize,
}

/// A language with a full list and a dictionary.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Lang {
    /// The wordfreq list.
    list: String,
    /// The Hunspell affix file.
    aff: String,
    /// The Hunspell word file.
    dic: String,
    /// The most words kept.
    max_words: usize,
    /// The tokens that take back their apostrophe.
    #[serde(default)]
    elisions: Vec<String>,
    /// The hamza rule, for Arabic.
    hamza: Option<Hamza>,
}

/// The alef forms of the hamza rule.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Hamza {
    /// The bare alef.
    pub(crate) bare: char,
    /// The alef forms with a hamza or a madda.
    pub(crate) marked: Vec<char>,
    /// A hamza form more than this many centibels rarer than the bare word takes nothing from it.
    pub(crate) max_gap_cb: u32,
}

/// One source file.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Source {
    /// The base it is downloaded from.
    base: Option<String>,
    /// Its folder under that base.
    dir: Option<String>,
    /// The archive it is unzipped from instead.
    zip: Option<String>,
    /// Its SHA-256, in lowercase hex.
    pub(crate) sha256: String,
    /// Its size in bytes.
    pub(crate) size: u64,
}

/// Where a source comes from.
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum Origin<'a> {
    /// Downloaded from this URL.
    Url(String),
    /// Unzipped from this source.
    Zip(&'a str),
}

/// A language's dictionary files.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct DictFiles<'a> {
    /// The affix file.
    pub(crate) aff: &'a str,
    /// The word file.
    pub(crate) dic: &'a str,
}

/// What building one language's word list needs.
#[derive(Debug)]
pub(crate) struct Plan<'a> {
    /// The wordfreq list.
    pub(crate) list: String,
    /// The dictionary, if the language has one.
    pub(crate) dict: Option<DictFiles<'a>>,
    /// The most words kept.
    pub(crate) max_words: usize,
    /// The tokens that take back their apostrophe.
    pub(crate) elisions: &'a [String],
    /// The hamza rule, if any.
    pub(crate) hamza: Option<&'a Hamza>,
}

impl Manifest {
    /// Parses and checks a manifest.
    pub(crate) fn parse(text: &str) -> Result<Self, WordsError> {
        let manifest: Self = toml::from_str(text).map_err(|e| bad(&e.to_string()))?;
        manifest.check_sources()?;
        manifest.check_langs()?;
        Ok(manifest)
    }

    /// The plan for `iso`: its full list and dictionary, or its small list.
    pub(crate) fn plan(&self, iso: &str) -> Result<Plan<'_>, WordsError> {
        if let Some(lang) = self.lang.get(iso) {
            let dict = DictFiles {
                aff: &lang.aff,
                dic: &lang.dic,
            };
            return Ok(Plan {
                list: lang.list.clone(),
                dict: Some(dict),
                max_words: lang.max_words,
                elisions: &lang.elisions,
                hamza: lang.hamza.as_ref(),
            });
        }
        let list = self.small.list.replace(ISO_SLOT, iso);
        if !fits_iso(iso.as_bytes()) || !self.sources.contains_key(&list) {
            return Err(WordsError::Lang(iso.to_string()));
        }
        Ok(Plan {
            list,
            dict: None,
            max_words: self.small.max_words,
            elisions: &[],
            hamza: None,
        })
    }

    /// Where the source `rel` comes from; `None` for a path that is not a source.
    pub(crate) fn origin(&self, rel: &str) -> Option<Origin<'_>> {
        let source = self.sources.get(rel)?;
        if let Some(zip) = &source.zip {
            return Some(Origin::Zip(zip));
        }
        let base = self.bases.get(source.base.as_ref()?)?;
        let name = rel.rsplit(SEP).next()?;
        let dir = source.dir.as_deref().unwrap_or_default();
        Some(Origin::Url(format!("{base}{dir}{name}")))
    }

    /// Refuses a source with a bad path, hash or origin.
    fn check_sources(&self) -> Result<(), WordsError> {
        for (rel, source) in &self.sources {
            let hex = |c: char| c.is_ascii_digit() || ('a'..='f').contains(&c);
            if !is_relative_plain(rel) {
                return Err(bad(&format!(
                    "{rel:?} is not a relative path of plain names"
                )));
            }
            if source.sha256.len() != HASH_HEX || !source.sha256.chars().all(hex) {
                return Err(bad(&format!(
                    "{rel}: the SHA-256 is not {HASH_HEX} lowercase hex digits"
                )));
            }
            let downloaded = source
                .base
                .as_ref()
                .is_some_and(|b| self.bases.contains_key(b));
            let unzipped = source.zip.as_ref().is_some_and(|z| self.is_download(z));
            let dir_ok = source.dir.is_none() || source.base.is_some();
            if downloaded == unzipped || !dir_ok {
                return Err(bad(&format!(
                    "{rel}: name one known base, or one downloaded zip"
                )));
            }
        }
        Ok(())
    }

    /// True when `rel` is a source downloaded from a base.
    fn is_download(&self, rel: &str) -> bool {
        self.sources.get(rel).is_some_and(|s| s.base.is_some())
    }

    /// Refuses a language whose files are not sources, or whose limits or rules are empty.
    fn check_langs(&self) -> Result<(), WordsError> {
        if !self.small.list.contains(ISO_SLOT) || self.small.max_words == 0 {
            return Err(bad(&format!(
                "[small] needs {ISO_SLOT} in its list and some max_words"
            )));
        }
        for (iso, lang) in &self.lang {
            let files = [&lang.list, &lang.aff, &lang.dic];
            let known = files.iter().all(|f| self.sources.contains_key(*f));
            let hamza_ok = lang
                .hamza
                .as_ref()
                .is_none_or(|h| !h.marked.is_empty() && !h.marked.contains(&h.bare));
            if !fits_iso(iso.as_bytes()) || !known || lang.max_words == 0 || !hamza_ok {
                return Err(bad(&format!(
                    "[lang.{iso}] needs a pack code, known files, some max_words and a valid hamza rule"
                )));
            }
        }
        Ok(())
    }
}

/// A manifest error.
fn bad(why: &str) -> WordsError {
    WordsError::Manifest(why.to_string())
}
