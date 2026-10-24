//! The `words` subcommand: checks the downloaded sources, turns wordfreq's lists into cleaned, dictionary-checked
//! word lists, counts word chains in Tatoeba and Wikipedia, and writes the packs with their credits (spec §6,
//! ADR-0032 decisions 1–3). It never ships; it builds data files.

mod build;
mod cands;
mod chains;
mod clean;
mod count;
mod credits;
mod dict;
mod fetch;
#[cfg(test)]
mod fixture;
mod list;
mod manifest;
mod merge;
mod report;
mod script;
mod split;
mod tatoeba;
#[cfg(test)]
mod tests;
mod text;
mod vocab;
mod wiki;
mod wordfreq;

use std::fmt;
use std::path::{Path, PathBuf};

use self::cands::Candidates;
use self::dict::Dict;
use self::manifest::Manifest;
use crate::devtools::paths::join_rel;
use crate::workspace;

/// The manifest, compiled in: sources, hashes and rules.
const MANIFEST: &str = include_str!("../words.toml");

/// Why a `words` command failed.
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum WordsError {
    /// `words.toml` is invalid.
    Manifest(String),
    /// A file could not be read or written.
    Io(String),
    /// A word list is not in wordfreq's format.
    Wordfreq(String),
    /// A dictionary does not parse.
    Dict(String),
    /// No word list is known for this language.
    Lang(String),
    /// curl could not download a file.
    Download(String),
    /// These source files failed their check.
    Bad(Vec<String>),
    /// A pack could not be written.
    Pack(String),
    /// The workspace could not be found.
    Workspace(String),
}

impl fmt::Display for WordsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Manifest(why) => write!(f, "words.toml: {why}"),
            Self::Io(why) => write!(f, "file error: {why}"),
            Self::Wordfreq(why) => write!(f, "wordfreq list: {why}"),
            Self::Dict(why) => write!(f, "dictionary: {why}"),
            Self::Lang(iso) => write!(f, "no word list is known for {iso:?}"),
            Self::Download(why) => write!(f, "download failed: {why}"),
            Self::Bad(files) => write!(f, "these files failed their check: {}", files.join("; ")),
            Self::Pack(why) => write!(f, "pack: {why}"),
            Self::Workspace(why) => write!(f, "workspace: {why}"),
        }
    }
}

/// A file error that names the file.
fn io_error(path: &Path, err: &std::io::Error) -> WordsError {
    WordsError::Io(format!("{}: {err}", path.display()))
}

/// Runs `cargo xtask words fetch`: checks every source under `src`, downloading the missing ones.
pub(crate) fn fetch(src: &str) -> Result<(), WordsError> {
    let manifest = Manifest::parse(MANIFEST)?;
    let mut get = |url: &str, size: u64, out: &Path| fetch::curl(&manifest.fetch, url, size, out);
    let mut say = |line: &str| println!("{line}");
    fetch::run(&manifest, Path::new(src), &mut get, &mut say)
}

/// Runs `cargo xtask words list`: prints the counts and one line per rule of `iso`'s word list, without text.
pub(crate) fn list(iso: &str, src: &str) -> Result<(), WordsError> {
    let manifest = Manifest::parse(MANIFEST)?;
    let cands = candidates(&manifest, iso, Path::new(src))?;
    let words = list::finish(cands, None, &manifest.rules);
    for line in report::report(iso, &words) {
        println!("{line}");
    }
    Ok(())
}

/// Runs `cargo xtask words build`: builds every language's pack and the credits into `out`, by default the
/// manifest's folder under the workspace root; the held-back sentences go under cargo's target folder.
pub(crate) fn build(src: &str, out: Option<&str>) -> Result<(), WordsError> {
    let manifest = Manifest::parse(MANIFEST)?;
    let lost = |e: workspace::LoadError| WordsError::Workspace(e.to_string());
    let json = workspace::metadata_json().map_err(lost)?;
    let root = workspace::root(&json).map_err(lost)?;
    let target = workspace::target_dir(&json).map_err(lost)?;
    let out = out.map_or_else(|| join_rel(root, &manifest.text.out), PathBuf::from);
    let paths = build::Paths {
        src: PathBuf::from(src),
        out,
        held: join_rel(target, &manifest.text.held),
    };
    build::run(&manifest, &paths, &mut |line| println!("{line}"))
}

/// The candidates of `iso` from the files under `src`, after checking their hashes; text rules come after.
pub(crate) fn candidates(
    manifest: &Manifest,
    iso: &str,
    src: &Path,
) -> Result<Candidates, WordsError> {
    let plan = manifest.plan(iso)?;
    let at = |rel: &str| join_rel(src.to_path_buf(), rel);
    let mut files = vec![plan.list.as_str()];
    files.extend(plan.dict.iter().flat_map(|d| [d.aff, d.dic]));
    fetch::verify(manifest, src, &files)?;
    let entries = wordfreq::read(&at(&plan.list))?;
    let apostrophe = manifest.rules.apostrophe;
    let dict = plan
        .dict
        .map(|d| Dict::load(&at(d.aff), &at(d.dic), apostrophe))
        .transpose()?;
    Ok(cands::candidates(
        entries,
        &plan,
        dict.as_ref(),
        &manifest.rules,
    ))
}

/// A word or token count as a float, for the share and ratio rules.
#[allow(
    clippy::cast_precision_loss,
    reason = "Word and token counts are far below 2^53."
)]
fn float(n: u64) -> f64 {
    n as f64
}

/// `path` with `.ext` added after its whole name: `a.msgpack.gz` becomes `a.msgpack.gz.part`.
fn with_added_ext(path: &Path, ext: &str) -> PathBuf {
    let mut name = path.as_os_str().to_os_string();
    name.push(".");
    name.push(ext);
    PathBuf::from(name)
}
