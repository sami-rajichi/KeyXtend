//! The files written beside the packs: `CREDITS.txt` from the template `words-credits.txt`, with every source's URL
//! and the licence sections copied word for word from the sources; the Tatoeba authors; and the licence.

use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use super::manifest::{Manifest, Origin, Section};
use super::{WordsError, io_error};
use crate::devtools::paths::join_rel;

/// The credits template's file name beside `src`: one literal for the compiled-in copy and its errors.
macro_rules! template_name {
    () => {
        "words-credits.txt"
    };
}

/// The credits template, compiled in.
const TEMPLATE: &str = include_str!(concat!("../../", template_name!()));
/// Opens a placeholder in the template.
const OPEN: char = '{';
/// Closes a placeholder.
const CLOSE: char = '}';
/// Starts each line of a file list.
const BULLET: &str = "- ";
/// Parts the items of an inline list.
const LIST_SEP: &str = ", ";

/// Every source file the build reads, each once: each language's files, then those the credits copy from.
pub(crate) fn sources(manifest: &Manifest) -> Result<Vec<String>, WordsError> {
    let files = Files::of(manifest)?;
    let mut all = files.lists;
    let read = [files.dictionaries, files.tatoeba, files.wiki].concat();
    all.extend(read.into_iter().map(str::to_string));
    let c = &manifest.credits;
    for file in [&c.wordfreq.file, &c.scowl.file, &c.citations.file] {
        if !all.contains(file) {
            all.push(file.clone());
        }
    }
    Ok(all)
}

/// The credits, the authors and the licence as named files, reading the licence sections under `src` (checked by
/// `sources`); `authors` are the usernames of the Tatoeba sentences counted.
pub(crate) fn files(
    manifest: &Manifest,
    src: &Path,
    authors: &BTreeSet<String>,
) -> Result<Vec<(String, Vec<u8>)>, WordsError> {
    let c = &manifest.credits;
    let section = |s: &Section| copy(&join_rel(src.to_path_buf(), &s.file), s);
    let files = Files::of(manifest)?;
    let values = BTreeMap::from([
        ("licence", c.licence.clone()),
        ("authors", c.authors.clone()),
        ("wordfreq_files", list(manifest, &files.lists)),
        ("wordfreq_readme", url(manifest, &c.wordfreq.file)),
        ("wordfreq_licence", section(&c.wordfreq)?),
        ("wordfreq_citations", section(&c.citations)?),
        ("wiki_files", list(manifest, &files.wiki)),
        ("wiki_editions", files.editions.join(LIST_SEP)),
        ("wiki_dumps", dumps(manifest, &files.wiki)),
        ("tatoeba_files", list(manifest, &files.tatoeba)),
        ("dictionary_files", list(manifest, &files.dictionaries)),
        ("scowl_notice", section(&c.scowl)?),
    ]);
    let mut names: Vec<&str> = authors.iter().map(String::as_str).collect();
    names.push("");
    Ok(vec![
        (c.file.clone(), fill(TEMPLATE, &values)?.into_bytes()),
        (c.authors.clone(), names.join("\n").into_bytes()),
        (
            c.licence.clone(),
            format!("{}\n", c.licence_text.join("\n")).into_bytes(),
        ),
    ])
}

/// The source files the packs come from, by kind.
#[derive(Debug, Default)]
struct Files<'a> {
    /// The wordfreq lists.
    lists: Vec<String>,
    /// The Wikipedia dump files.
    wiki: Vec<&'a str>,
    /// The Tatoeba files.
    tatoeba: Vec<&'a str>,
    /// The dictionaries' files.
    dictionaries: Vec<&'a str>,
    /// The codes of the languages whose Wikipedia text was read.
    editions: Vec<String>,
}

impl<'a> Files<'a> {
    /// Every file `manifest`'s languages read, by kind; a language without a plan is refused, never skipped.
    fn of(manifest: &'a Manifest) -> Result<Self, WordsError> {
        let mut files = Self::default();
        for iso in manifest.isos() {
            let plan = manifest.plan(&iso)?;
            files.lists.push(plan.list);
            files
                .dictionaries
                .extend(plan.dict.iter().flat_map(|d| [d.aff, d.dic]));
            let Some(text) = plan.text else {
                continue;
            };
            files
                .tatoeba
                .extend(text.tatoeba.iter().map(|t| t.file.as_str()));
            let wiki = text.wiki.iter().flat_map(|w| w.files.iter());
            let before = files.wiki.len();
            files.wiki.extend(wiki.map(String::as_str));
            if files.wiki.len() > before {
                files.editions.push(iso);
            }
        }
        Ok(files)
    }
}

/// The URL `rel` is downloaded from, or the URL of the archive it is unzipped from.
fn url(manifest: &Manifest, rel: &str) -> String {
    match manifest.origin(rel) {
        Some(Origin::Url(url)) => url,
        Some(Origin::Zip(zip)) => url(manifest, zip),
        None => rel.to_string(),
    }
}

/// The distinct base folders the Wikipedia `files` come from, which name their dumps' dates.
fn dumps(manifest: &Manifest, files: &[&str]) -> String {
    let mut seen = BTreeSet::new();
    let folders = files.iter().filter_map(|f| manifest.base_folder(f));
    let distinct: Vec<&str> = folders.filter(|d| seen.insert(*d)).collect();
    distinct.join(LIST_SEP)
}

/// One line per distinct URL of `files`, in order.
fn list(manifest: &Manifest, files: &[impl AsRef<str>]) -> String {
    let mut seen = BTreeSet::new();
    let urls = files
        .iter()
        .map(|f| url(manifest, f.as_ref()))
        .filter(|u| seen.insert(u.clone()));
    urls.map(|u| format!("{BULLET}{u}"))
        .collect::<Vec<_>>()
        .join("\n")
}

/// The lines of the file at `path` that `section` names, word for word, without trailing blank lines; a mark no
/// line starts with is refused.
pub(crate) fn copy(path: &Path, section: &Section) -> Result<String, WordsError> {
    let text = std::fs::read_to_string(path).map_err(|e| io_error(path, &e))?;
    let missing =
        |mark: &str| WordsError::Io(format!("{}: no line starts with {mark:?}", path.display()));
    let start = (text.lines().position(|l| l.starts_with(&section.from)))
        .ok_or_else(|| missing(&section.from))?;
    let mut kept: Vec<&str> = text.lines().skip(start).collect();
    if let Some(to) = &section.to {
        let end = (kept.iter().skip(1).position(|l| l.starts_with(to.as_str())))
            .ok_or_else(|| missing(to))?;
        kept.truncate(end + 1);
    }
    while kept.last().is_some_and(|l| l.trim().is_empty()) {
        kept.pop();
    }
    Ok(kept.join("\n"))
}

/// `template` with each `{name}` replaced by its value; an unknown or unclosed name is refused.
pub(crate) fn fill(template: &str, values: &BTreeMap<&str, String>) -> Result<String, WordsError> {
    let mut out = String::with_capacity(template.len());
    let mut rest = template;
    while let Some(at) = rest.find(OPEN) {
        out.push_str(rest.get(..at).unwrap_or_default());
        let tail = rest.get(at + OPEN.len_utf8()..).unwrap_or_default();
        let end = tail.find(CLOSE).ok_or_else(|| bad_slot(tail))?;
        let name = tail.get(..end).unwrap_or_default();
        out.push_str(values.get(name).ok_or_else(|| bad_slot(name))?);
        rest = tail.get(end + CLOSE.len_utf8()..).unwrap_or_default();
    }
    out.push_str(rest);
    Ok(out)
}

/// The error for a placeholder the template cannot fill.
fn bad_slot(name: &str) -> WordsError {
    WordsError::Manifest(format!(
        "{}: no value for the placeholder {name:?}",
        template_name!()
    ))
}
