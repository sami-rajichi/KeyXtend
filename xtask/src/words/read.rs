//! Reads one language's text into the counter: Tatoeba twice, first for its words' shares and then counted with a
//! share held back, then Wikipedia up to its cap.

use std::collections::BTreeSet;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;

use kx_mod_predict::split::Splitter;

use super::build::Paths;
use super::cands::{Candidates, Tally};
use super::count::Counter;
use super::manifest::{LangText, Manifest, Tatoeba};
use super::skew::Shares;
use super::vocab::Vocab;
use super::{WordsError, io_error, tatoeba, wiki};
use crate::devtools::paths::join_rel;

/// Wikipedia's weight: the unit that Tatoeba's weight is counted in.
const WIKI_WEIGHT: u32 = 1;
/// The extension of a held-back sentences file.
const HELD_EXT: &str = "txt";

/// What reading one language's text found.
#[derive(Debug, Default)]
pub(crate) struct Read {
    /// Tatoeba lines read.
    pub(crate) lines: u64,
    /// Of them, held back.
    pub(crate) held: u64,
    /// Bytes of Wikipedia text read.
    pub(crate) wiki_bytes: u64,
    /// The words whose Tatoeba chains were scaled, with their skew.
    pub(crate) skewed: Tally,
}

/// Counts `iso`'s text into `counter` from the checked sources, adding the authors of the Tatoeba sentences counted.
pub(crate) fn text(
    manifest: &Manifest,
    iso: &str,
    text: &LangText,
    (cands, vocab): (&Candidates, &Vocab),
    paths: &Paths,
    counter: &mut Counter<'_>,
    authors: &mut BTreeSet<String>,
) -> Result<Read, WordsError> {
    let mut read = Read::default();
    if let Some(tatoeba) = &text.tatoeba {
        let t = &manifest.text;
        let split = Splitter {
            ends: &t.sentence_ends,
            apostrophe: manifest.rules.apostrophe,
        };
        let mut shares = Shares::new(vocab, cands.cands.len(), split, t.max_seen_sentences);
        each_counted(paths, tatoeba, t.held_back_per_mille, &mut |e| {
            shares.add(e.text);
        })?;
        let show = manifest.rules.show;
        let (skew, skewed) = shares.factors(&cands.cands, &cands.tokens, t.skew_ratio, show);
        counter.scale(skew);
        read.skewed = skewed;
        tatoeba_counted(manifest, iso, tatoeba, paths, counter, authors, &mut read)?;
    }
    for file in text.wiki.iter().flat_map(|w| w.files.iter()) {
        let cap = text.wiki.as_ref().map_or(0, |w| w.max_bytes);
        let left = cap.saturating_sub(read.wiki_bytes);
        let mut each = |doc: &str| {
            counter.add(doc, WIKI_WEIGHT, false);
        };
        read.wiki_bytes += wiki::read(source(&paths.src, file)?, left, &mut each)
            .map_err(|why| bad(file, &why))?;
    }
    Ok(read)
}

/// Counts `iso`'s Tatoeba sentences, scaled, writing the held-back ones into the held folder.
fn tatoeba_counted(
    manifest: &Manifest,
    iso: &str,
    tatoeba: &Tatoeba,
    paths: &Paths,
    counter: &mut Counter<'_>,
    authors: &mut BTreeSet<String>,
    read: &mut Read,
) -> Result<(), WordsError> {
    let t = &manifest.text;
    std::fs::create_dir_all(&paths.held).map_err(|e| io_error(&paths.held, &e))?;
    let held_path = paths.held.join(format!("{iso}.{HELD_EXT}"));
    let held_file = File::create(&held_path).map_err(|e| io_error(&held_path, &e))?;
    let mut held = BufWriter::new(held_file);
    let mut failed = None;
    let mut each = |e: tatoeba::Entry<'_>| {
        if tatoeba::held(e.id, t.held_back_per_mille) {
            read.held += 1;
            failed = failed.take().or_else(|| writeln!(held, "{}", e.text).err());
            return;
        }
        let added = counter.add(e.text, t.tatoeba_weight, true);
        if added && e.user != t.null {
            authors.insert(e.user.to_string());
        }
    };
    let src = source(&paths.src, &tatoeba.file)?;
    read.lines = tatoeba::read(src, tatoeba.max_sentences, &mut each)
        .map_err(|why| bad(&tatoeba.file, &why))?;
    let flushed = failed.map_or_else(|| held.flush(), Err);
    flushed.map_err(|e| io_error(&held_path, &e))
}

/// Gives `each` every Tatoeba entry that is not held back, up to the file's cap.
fn each_counted(
    paths: &Paths,
    tatoeba: &Tatoeba,
    per_mille: u64,
    each: &mut dyn FnMut(tatoeba::Entry<'_>),
) -> Result<(), WordsError> {
    let mut kept = |e: tatoeba::Entry<'_>| {
        if !tatoeba::held(e.id, per_mille) {
            each(e);
        }
    };
    let src = source(&paths.src, &tatoeba.file)?;
    tatoeba::read(src, tatoeba.max_sentences, &mut kept).map_err(|why| bad(&tatoeba.file, &why))?;
    Ok(())
}

/// The source file `rel` under `src`, opened.
fn source(src: &Path, rel: &str) -> Result<File, WordsError> {
    let path = join_rel(src.to_path_buf(), rel);
    File::open(&path).map_err(|e| io_error(&path, &e))
}

/// A text source's error, naming the file.
fn bad(rel: &str, why: &str) -> WordsError {
    WordsError::Io(format!("{rel}: {why}"))
}
