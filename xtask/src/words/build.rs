//! `words build`: for each language, one pass over its Tatoeba and Wikipedia text counts words and chains, the list
//! is finished with those counts, the chains are picked, and the pack is written; then the credits.

use std::collections::BTreeSet;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};

use kx_mod_predict::{Flags, PackWriter};
use kx_module_api::ports::pack_file;

use super::cands::Candidates;
use super::chains::{self, Picked};
use super::count::{Counted, Counter};
use super::list::{self, WordList};
use super::manifest::{LangText, Manifest};
use super::report::report;
use super::vocab::Vocab;
use super::{WordsError, candidates, credits, fetch, io_error, tatoeba, wiki};
use crate::devtools::paths::{BYTES_PER_KB, join_rel};

/// Wikipedia's weight: the unit that Tatoeba's weight is counted in.
const WIKI_WEIGHT: u32 = 1;
/// The extension of a held-back sentences file.
const HELD_EXT: &str = "txt";

/// Where `build` reads and writes.
#[derive(Debug)]
pub(crate) struct Paths {
    /// The source folder.
    pub(crate) src: PathBuf,
    /// The folder the packs and credits go to.
    pub(crate) out: PathBuf,
    /// The folder the held-back sentences go to.
    pub(crate) held: PathBuf,
}

/// What reading one language's text found.
#[derive(Debug, Default)]
struct Read {
    /// Tatoeba lines read.
    lines: u64,
    /// Of them, held back.
    held: u64,
    /// Bytes of Wikipedia text read.
    wiki_bytes: u64,
}

/// Builds every language's pack into `paths.out`, then the credits, saying one line per fact.
pub(crate) fn run(
    manifest: &Manifest,
    paths: &Paths,
    say: &mut dyn FnMut(&str),
) -> Result<(), WordsError> {
    std::fs::create_dir_all(&paths.out).map_err(|e| io_error(&paths.out, &e))?;
    let mut authors = BTreeSet::new();
    for iso in manifest.isos() {
        let (pack, lines) = lang(manifest, &iso, paths, &mut authors)?;
        let name = pack_file(&iso).ok_or_else(|| WordsError::Lang(iso.clone()))?;
        let path = paths.out.join(name);
        std::fs::write(&path, pack).map_err(|e| io_error(&path, &e))?;
        for line in &lines {
            say(line);
        }
    }
    credits::write(manifest, &paths.src, &paths.out, &authors)
}

/// The pack of `iso` and the lines that report it, adding the authors of the Tatoeba sentences counted.
fn lang(
    manifest: &Manifest,
    iso: &str,
    paths: &Paths,
    authors: &mut BTreeSet<String>,
) -> Result<(Vec<u8>, Vec<String>), WordsError> {
    let cands = candidates(manifest, iso, &paths.src)?;
    let (list, picked, text) = match manifest.plan(iso)?.text {
        None => {
            let list = list::finish(cands, None, &manifest.rules);
            (list, Picked::default(), None)
        }
        Some(text) => {
            let (list, picked, line) = counted(manifest, iso, text, cands, paths, authors)?;
            (list, picked, Some(line))
        }
    };
    let pack = write_pack(iso, &list, &picked, manifest.text.followers)?;
    let mut lines = report(iso, &list);
    lines.extend(text);
    let (words, contexts, chains) = (list.words.len(), picked.contexts.len(), picked.chains);
    let (chain_bytes, bytes) = (picked.bytes, pack.len());
    lines.push(format!(
        "pack {iso}: {words} words, {contexts} contexts, {chains} chains ({chain_bytes} chain bytes), {bytes} bytes"
    ));
    Ok((pack, lines))
}

/// `iso`'s list finished with its text's counts, its picked chains, and the line that says what the text held.
fn counted(
    manifest: &Manifest,
    iso: &str,
    text: &LangText,
    cands: Candidates,
    paths: &Paths,
    authors: &mut BTreeSet<String>,
) -> Result<(WordList, Picked, String), WordsError> {
    let (rules, t) = (&manifest.rules, &manifest.text);
    let vocab = Vocab::new(&cands.cands);
    let mut counter = Counter::new(&vocab, cands.cands.len(), t, rules.apostrophe);
    let read = read_text(manifest, iso, text, paths, &mut counter, authors)?;
    let found = counter.finish();
    let line = text_line(iso, &read, &found);
    let list = list::finish(cands, Some(&found.text), rules);
    let budget = usize::try_from(text.chain_budget_kb * BYTES_PER_KB).unwrap_or(usize::MAX);
    let (table, seen) = (found.chains.table, &found.seen);
    let picked = chains::pick(table, seen, &list.of_cand, t.followers, budget, t.max_chain);
    Ok((list, picked, line))
}

/// Counts `iso`'s Tatoeba sentences, holding some back into the held folder, then its Wikipedia text up to its cap.
fn read_text(
    manifest: &Manifest,
    iso: &str,
    text: &LangText,
    paths: &Paths,
    counter: &mut Counter<'_>,
    authors: &mut BTreeSet<String>,
) -> Result<Read, WordsError> {
    let files: Vec<&str> = text.files().collect();
    fetch::verify(manifest, &paths.src, &files)?;
    let mut read = Read::default();
    let t = &manifest.text;
    if let Some(tatoeba) = &text.tatoeba {
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
            counter.add(e.text, t.tatoeba_weight);
            if e.user != t.null {
                authors.insert(e.user.to_string());
            }
        };
        let src = source(&paths.src, &tatoeba.file)?;
        read.lines = tatoeba::read(src, tatoeba.max_sentences, &mut each)
            .map_err(|why| bad(&tatoeba.file, &why))?;
        let flushed = failed.map_or_else(|| held.flush(), Err);
        flushed.map_err(|e| io_error(&held_path, &e))?;
    }
    for file in text.wiki.iter().flat_map(|w| w.files.iter()) {
        let left = text
            .wiki
            .as_ref()
            .map_or(0, |w| w.max_bytes)
            .saturating_sub(read.wiki_bytes);
        let mut each = |doc: &str| counter.add(doc, WIKI_WEIGHT);
        read.wiki_bytes += wiki::read(source(&paths.src, file)?, left, &mut each)
            .map_err(|why| bad(file, &why))?;
    }
    Ok(read)
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

/// The line that says what `iso`'s text held.
fn text_line(iso: &str, read: &Read, counted: &Counted) -> String {
    let (lines, held, bytes) = (read.lines, read.held, read.wiki_bytes);
    let (tokens, prunes, floor) = (
        counted.text.total,
        counted.chains.prunes,
        counted.chains.floor,
    );
    let live = counted.chains.table.len();
    format!(
        "text {iso}: {tokens} weighted tokens from {lines} Tatoeba lines ({held} held back) and {bytes} bytes of \
         Wikipedia; {live} chains counted, the table pruned {prunes} times up to a floor of {floor}"
    )
}

/// The pack of `iso`'s `list` and `picked` chains, keeping `followers` per context.
pub(crate) fn write_pack(
    iso: &str,
    list: &WordList,
    picked: &Picked,
    followers: u8,
) -> Result<Vec<u8>, WordsError> {
    let flags = Flags {
        checked: list.coverage.is_some(),
        chains: !picked.contexts.is_empty(),
    };
    let refused = |e: kx_mod_predict::WriteError| WordsError::Pack(format!("{iso}: {e}"));
    let mut writer = PackWriter::new(iso, flags, followers).map_err(refused)?;
    for word in &list.words {
        writer
            .add_word(&word.spelling, word.freq, word.capital)
            .map_err(refused)?;
    }
    let spelling = |w: u32| {
        usize::try_from(w)
            .ok()
            .and_then(|w| list.words.get(w))
            .map(|w| w.spelling.as_str())
    };
    for context in &picked.contexts {
        let words: Vec<&str> = context.words.iter().filter_map(|&w| spelling(w)).collect();
        for &(next, score) in &context.followers {
            let next = spelling(next).unwrap_or_default();
            writer.add_chain(&words, next, score).map_err(refused)?;
        }
    }
    writer.finish().map_err(refused)
}
