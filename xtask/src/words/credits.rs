//! The files written beside the packs: `CREDITS.txt` from the template `words-credits.txt`, with every source's URL
//! and the licence sections copied word for word from the sources; the Tatoeba authors; and the licence.

use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use super::manifest::{Manifest, Origin, Section};
use super::{WordsError, fetch, io_error};
use crate::devtools::paths::join_rel;

/// The credits template, compiled in.
const TEMPLATE: &str = include_str!("../../words-credits.txt");
/// Opens a placeholder in the template.
const OPEN: char = '{';
/// Closes a placeholder.
const CLOSE: char = '}';
/// Starts each line of a file list.
const BULLET: &str = "- ";

/// Writes the credits, the authors and the licence into `out`, reading the licence sections under `src`; `authors`
/// are the usernames of the Tatoeba sentences counted.
pub(crate) fn write(
    manifest: &Manifest,
    src: &Path,
    out: &Path,
    authors: &BTreeSet<String>,
) -> Result<(), WordsError> {
    let c = &manifest.credits;
    fetch::verify(
        manifest,
        src,
        &[c.wordfreq.file.as_str(), c.scowl.file.as_str()],
    )?;
    let section = |s: &Section| copy(&join_rel(src.to_path_buf(), &s.file), s);
    let files = Files::of(manifest);
    let values = BTreeMap::from([
        ("licence", c.licence.clone()),
        ("authors", c.authors.clone()),
        ("wordfreq_files", list(manifest, &files.lists)),
        ("wordfreq_readme", url(manifest, &c.wordfreq.file)),
        ("wordfreq_licence", section(&c.wordfreq)?),
        ("wiki_files", list(manifest, &files.wiki)),
        ("tatoeba_files", list(manifest, &files.tatoeba)),
        ("dictionary_files", list(manifest, &files.dictionaries)),
        ("scowl_notice", section(&c.scowl)?),
    ]);
    let mut names: Vec<&str> = authors.iter().map(String::as_str).collect();
    names.push("");
    let put = |name: &str, text: &str| {
        let path = out.join(name);
        std::fs::write(&path, text).map_err(|e| io_error(&path, &e))
    };
    put(&c.file, &fill(TEMPLATE, &values)?)?;
    put(&c.authors, &names.join("\n"))?;
    put(&c.licence, &format!("{}\n", c.licence_text.join("\n")))
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
}

impl<'a> Files<'a> {
    /// Every file `manifest`'s languages read, by kind.
    fn of(manifest: &'a Manifest) -> Self {
        let mut files = Self::default();
        for iso in manifest.isos() {
            let Ok(plan) = manifest.plan(&iso) else {
                continue;
            };
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
            files.wiki.extend(
                text.wiki
                    .iter()
                    .flat_map(|w| w.files.iter().map(String::as_str)),
            );
        }
        files
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

/// The lines of the file at `path` that `section` names, word for word, without trailing blank lines.
fn copy(path: &Path, section: &Section) -> Result<String, WordsError> {
    let text = std::fs::read_to_string(path).map_err(|e| io_error(path, &e))?;
    let mut lines = text.lines().skip_while(|l| !l.starts_with(&section.from));
    let first = lines.next().ok_or_else(|| {
        WordsError::Io(format!(
            "{}: no line starts with {:?}",
            path.display(),
            section.from
        ))
    })?;
    let mut kept: Vec<&str> = std::iter::once(first)
        .chain(lines.take_while(|l| !l.starts_with(&section.to)))
        .collect();
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
        "words-credits.txt: no value for the placeholder {name:?}"
    ))
}
