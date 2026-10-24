//! The manifest `words.toml`: every source file with its hash, the rules' thresholds and each language's files.

mod check;

use std::collections::BTreeMap;

use kx_mod_predict::pack::layout::fits_iso;
use serde::Deserialize;

use super::WordsError;
use crate::devtools::paths::SEP;

/// The placeholder for the language code in the small lists' path.
const ISO_SLOT: &str = "{iso}";
/// A SHA-256 is 32 bytes, written as two lowercase hex digits each.
pub(crate) const HASH_HEX: usize = 64;

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

/// How words are cleaned and reported: every threshold the word facts use.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Rules {
    /// A word holding one of these is dropped.
    pub(crate) drop_chars: Vec<char>,
    /// The apostrophe of elided and joined words.
    pub(crate) apostrophe: char,
    /// A rejected word merges only into a fold twin at most this many centibels rarer.
    pub(crate) max_gap_cb: u32,
    /// The share of a list's words that makes a script the list's own.
    pub(crate) main_script_share: f64,
    /// How often text must show a twin spelling away from sentence starts before it wins.
    pub(crate) min_case_seen: u64,
    /// A word standing alone this many times less often than wordfreq predicts is a piece.
    pub(crate) piece_ratio: f64,
    /// The fewest expected sightings in the text for a word to be judged a piece.
    pub(crate) piece_min_expected: f64,
    /// How many of the most frequent words the coverage line checks.
    pub(crate) coverage_top: usize,
    /// How many words every report line prints.
    pub(crate) show: usize,
}

/// How a missing source is downloaded with curl.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct FetchConfig {
    /// Env var naming the folder that holds the system's curl.
    pub(crate) root_env: String,
    /// curl, under that folder.
    pub(crate) curl: String,
    /// The program run when that curl is not there, found on the PATH.
    pub(crate) fallback: String,
    /// curl's first arguments.
    pub(crate) args: Vec<String>,
    /// The flag before the file's size limit.
    pub(crate) size_flag: String,
    /// The flag before the output file.
    pub(crate) out_flag: String,
    /// The flag before the URL.
    pub(crate) url_flag: String,
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
}

impl Manifest {
    /// Parses and checks a manifest.
    pub(crate) fn parse(text: &str) -> Result<Self, WordsError> {
        let manifest: Self = toml::from_str(text).map_err(|e| bad(&e.to_string()))?;
        check::rules(&manifest.rules)?;
        check::fetch(&manifest.fetch)?;
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

    /// Refuses a non-https base, and a source with a bad path, hash or origin.
    fn check_sources(&self) -> Result<(), WordsError> {
        check::bases(self.bases.values())?;
        for (rel, source) in &self.sources {
            check::source_path(rel, &source.sha256)?;
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

    /// Refuses a language whose files are not sources, or whose limit is zero.
    fn check_langs(&self) -> Result<(), WordsError> {
        if !self.small.list.contains(ISO_SLOT) || self.small.max_words == 0 {
            return Err(bad(&format!(
                "[small] needs {ISO_SLOT} in its list and some max_words"
            )));
        }
        for (iso, lang) in &self.lang {
            let files = [&lang.list, &lang.aff, &lang.dic];
            let known = files.iter().all(|f| self.sources.contains_key(*f));
            if !fits_iso(iso.as_bytes()) || !known || lang.max_words == 0 {
                return Err(bad(&format!(
                    "[lang.{iso}] needs a pack code, known files and some max_words"
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
