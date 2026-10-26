//! The manifest `words.toml`: every source file with its hash, the rules' thresholds, each language's files and
//! text, and the credits.

mod check;
mod origin;
mod text;

use std::collections::BTreeMap;

use kx_mod_predict::fold;
use kx_module_api::ports::fits_iso;
use serde::Deserialize;

pub(crate) use self::origin::Origin;
pub(crate) use self::text::{
    Credits, LangText, MeterRules, PER_MILLE, PERCENT, Packs, Section, Tatoeba, TextRules,
};

use super::WordsError;

/// The placeholder for the language code in the small lists' path.
const ISO_SLOT: &str = "{iso}";
/// A SHA-256 is 32 bytes, written as two lowercase hex digits each.
pub(crate) const HASH_HEX: usize = 64;
/// The error of a manifest that builds no pack.
pub(crate) const NO_LANGS: &str = "[lang] and [small] name no language to build";

/// The whole manifest.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Manifest {
    /// How words are cleaned and reported.
    pub(crate) rules: Rules,
    /// How a missing source is downloaded.
    pub(crate) fetch: FetchConfig,
    /// How text is read and counted into word chains.
    pub(crate) text: TextRules,
    /// The files written beside the packs.
    pub(crate) credits: Credits,
    /// The packs' download budget.
    pub(crate) packs: Packs,
    /// How the meter types the held-back sentences and judges the packs.
    pub(crate) meter: MeterRules,
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
    /// The apostrophe of elided and joined words, from the fold table, never from `words.toml`.
    #[serde(skip)]
    pub(crate) apostrophe: char,
    /// A rejected word merges only into a fold twin at most this many centibels rarer.
    pub(crate) max_gap_cb: u32,
    /// The share of a list's words that makes a script the list's own.
    pub(crate) main_script_share: f64,
    /// How often text must show a twin spelling away from sentence starts before it wins.
    pub(crate) min_case_seen: u64,
    /// How many times more often a twin differing only in case must be seen than the word's own spelling.
    pub(crate) twin_ratio: f64,
    /// A word is a piece when its apostrophe words are seen this many times more often than it stands alone.
    pub(crate) piece_ratio: f64,
    /// The fewest sightings of a word's apostrophe words for it to be judged a piece.
    pub(crate) piece_min_seen: u64,
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
    /// The most words kept, before the words budget cuts the rarest.
    max_words: usize,
    /// The most KB each pack may take without its chains.
    words_budget_kb: u64,
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
    /// The most words kept, before the words budget cuts the rarest.
    max_words: usize,
    /// The most KB its pack may take without its chains.
    words_budget_kb: u64,
    /// Its text for word chains, if any.
    text: Option<LangText>,
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
    /// The language code.
    pub(crate) iso: String,
    /// The wordfreq list.
    pub(crate) list: String,
    /// The dictionary, if the language has one.
    pub(crate) dict: Option<DictFiles<'a>>,
    /// The most words kept, before the words budget cuts the rarest.
    pub(crate) max_words: usize,
    /// The most KB its pack may take without its chains.
    pub(crate) words_budget_kb: u64,
    /// Its text for word chains, if any.
    pub(crate) text: Option<&'a LangText>,
}

impl Manifest {
    /// Parses and checks a manifest.
    pub(crate) fn parse(text: &str) -> Result<Self, WordsError> {
        let mut manifest: Self = toml::from_str(text).map_err(|e| bad(&e.to_string()))?;
        manifest.rules.apostrophe =
            fold::apostrophe().ok_or_else(|| bad("the fold table names no apostrophe"))?;
        check::rules(&manifest.rules)?;
        check::fetch(&manifest.fetch)?;
        check::text(&manifest.text)?;
        check::packs(&manifest.packs)?;
        check::meter(&manifest.meter)?;
        manifest.check_sources()?;
        manifest.check_langs()?;
        let isos = manifest.isos();
        check::langs(&isos)?;
        check::aliases(&manifest.packs, &isos)?;
        check::credits(&manifest.credits, &manifest.sources)?;
        Ok(manifest)
    }

    /// Every language a pack is built for: the full ones, then each other one with a small list, by code.
    pub(crate) fn isos(&self) -> Vec<String> {
        let (head, tail) = self.small.list.split_once(ISO_SLOT).unwrap_or_default();
        let small = self.sources.keys().filter_map(|rel| {
            let iso = rel.strip_prefix(head)?.strip_suffix(tail)?;
            (fits_iso(iso) && !self.lang.contains_key(iso)).then(|| iso.to_string())
        });
        self.lang.keys().cloned().chain(small).collect()
    }

    /// The plan for `iso`: its full list and dictionary, or its small list.
    pub(crate) fn plan(&self, iso: &str) -> Result<Plan<'_>, WordsError> {
        if let Some(lang) = self.lang.get(iso) {
            let dict = DictFiles {
                aff: &lang.aff,
                dic: &lang.dic,
            };
            return Ok(Plan {
                iso: iso.to_string(),
                list: lang.list.clone(),
                dict: Some(dict),
                max_words: lang.max_words,
                words_budget_kb: lang.words_budget_kb,
                text: lang.text.as_ref(),
            });
        }
        let list = self.small.list.replace(ISO_SLOT, iso);
        if !fits_iso(iso) || !self.sources.contains_key(&list) {
            return Err(WordsError::Lang(iso.to_string()));
        }
        Ok(Plan {
            iso: iso.to_string(),
            list,
            dict: None,
            max_words: self.small.max_words,
            words_budget_kb: self.small.words_budget_kb,
            text: None,
        })
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

    /// Refuses a language whose files are not sources, whose small list is a source no plan reads (unless it is the
    /// language's own `list`), or whose limit is zero.
    fn check_langs(&self) -> Result<(), WordsError> {
        let small = &self.small;
        if !small.list.contains(ISO_SLOT) || small.max_words == 0 || small.words_budget_kb == 0 {
            return Err(bad(&format!(
                "[small] needs {ISO_SLOT} in its list, some max_words and a words budget"
            )));
        }
        for (iso, lang) in &self.lang {
            let mut files = vec![lang.list.as_str(), &lang.aff, &lang.dic];
            files.extend(lang.text.iter().flat_map(LangText::files));
            let known = files.iter().all(|f| self.sources.contains_key(*f));
            let own_small = small.list.replace(ISO_SLOT, iso);
            let unread = own_small != lang.list && self.sources.contains_key(&own_small);
            let text_ok = lang.text.as_ref().is_none_or(check::lang_text);
            let limits = lang.max_words > 0 && lang.words_budget_kb > 0;
            if !fits_iso(iso) || !known || unread || !limits || !text_ok {
                return Err(bad(&format!(
                    "[lang.{iso}] needs a pack code, known files, no small list, some max_words, a words budget and \
                     sound text limits"
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
