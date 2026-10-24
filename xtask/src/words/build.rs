//! `words build`: for each language, its Tatoeba and Wikipedia text is counted, the list is finished with those
//! counts, the chains are picked and the pack is written; then the credits, and every file once the download fits.

use std::collections::BTreeSet;
use std::path::PathBuf;

use kx_mod_predict::{Flags, PackWriter};
use kx_module_api::ports::pack_file;

use super::cands::Candidates;
use super::chains::{self, Picked};
use super::count::{Counted, Counter};
use super::list::{self, WordList};
use super::manifest::{LangText, Manifest};
use super::read::{self, Read};
use super::report::{report, tally_line};
use super::vocab::Vocab;
use super::{WordsError, candidates, credits, download, io_error};
use crate::devtools::paths::BYTES_PER_KB;

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

/// One file to write: its name, its bytes and their deflated size.
type File = (String, Vec<u8>, usize);

/// Builds every language's pack and the credits, saying one line per fact, then writes them into `paths.out` once
/// their deflated total fits the download budget.
pub(crate) fn run(
    manifest: &Manifest,
    paths: &Paths,
    say: &mut dyn FnMut(&str),
) -> Result<(), WordsError> {
    let level = manifest.packs.level;
    let mut authors = BTreeSet::new();
    let mut files: Vec<File> = Vec::new();
    for iso in manifest.isos() {
        let (pack, deflated, lines) = lang(manifest, &iso, paths, &mut authors)?;
        for line in &lines {
            say(line);
        }
        let name = pack_file(&iso).ok_or_else(|| WordsError::Lang(iso.clone()))?;
        files.push((name, pack, deflated));
    }
    for (name, bytes) in credits::files(manifest, &paths.src, &authors)? {
        let deflated = download::deflated(&bytes, level)?;
        files.push((name, bytes, deflated));
    }
    let sizes: Vec<(usize, usize)> = files.iter().map(|f| (f.1.len(), f.2)).collect();
    say(&download::check(&sizes, &manifest.packs)?);
    std::fs::create_dir_all(&paths.out).map_err(|e| io_error(&paths.out, &e))?;
    for (name, bytes, _) in files {
        let path = paths.out.join(name);
        std::fs::write(&path, bytes).map_err(|e| io_error(&path, &e))?;
    }
    Ok(())
}

/// The pack of `iso`, its deflated size and the lines that report it, adding the authors of the Tatoeba sentences
/// counted.
fn lang(
    manifest: &Manifest,
    iso: &str,
    paths: &Paths,
    authors: &mut BTreeSet<String>,
) -> Result<(Vec<u8>, usize, Vec<String>), WordsError> {
    let cands = candidates(manifest, iso, &paths.src)?;
    let (list, picked, text) = match manifest.plan(iso)?.text {
        None => (
            list::finish(cands, None, &manifest.rules)?,
            Picked::default(),
            Vec::new(),
        ),
        Some(text) => counted(manifest, iso, text, cands, paths, authors)?,
    };
    let pack = write_pack(iso, &list, &picked, manifest.text.followers)?;
    let deflated = download::deflated(&pack, manifest.packs.level)?;
    let mut lines = report(iso, &list);
    lines.extend(text);
    let (words, contexts, chains) = (list.words.len(), picked.contexts.len(), picked.chains);
    let (chain_bytes, bytes) = (picked.bytes, pack.len());
    lines.push(format!(
        "pack {iso}: {words} words, {contexts} contexts, {chains} chains ({chain_bytes} chain bytes), {bytes} bytes, \
         {deflated} deflated"
    ));
    Ok((pack, deflated, lines))
}

/// `iso`'s list finished with its text's counts, its picked chains, and the lines that say what the text held.
fn counted(
    manifest: &Manifest,
    iso: &str,
    text: &LangText,
    cands: Candidates,
    paths: &Paths,
    authors: &mut BTreeSet<String>,
) -> Result<(WordList, Picked, Vec<String>), WordsError> {
    let (rules, t) = (&manifest.rules, &manifest.text);
    let vocab = Vocab::new(&cands.cands);
    let mut counter = Counter::new(&vocab, cands.cands.len(), t, rules.apostrophe);
    let read = read::text(
        manifest,
        iso,
        text,
        (&cands, &vocab),
        paths,
        &mut counter,
        authors,
    )?;
    let found = counter.finish();
    let mut lines = text_lines(iso, &read, &found);
    let list = list::finish(cands, Some(&found.text), rules)?;
    let budget = usize::try_from(text.chain_budget_kb * BYTES_PER_KB).unwrap_or(usize::MAX);
    let (table, seen) = (found.chains.table, &found.seen);
    let picked = chains::pick(table, seen, &list.of_cand, t.followers, budget, t.max_chain);
    lines.push(top_line(iso, &list, &picked, manifest));
    Ok((list, picked, lines))
}

/// The lines that say what `iso`'s text held: its sizes and table, the repeats skipped and the Tatoeba words scaled.
fn text_lines(iso: &str, read: &Read, counted: &Counted) -> Vec<String> {
    let (lines, held, bytes) = (read.lines, read.held, read.wiki_bytes);
    let tokens = counted.text.total;
    let (live, prunes, floor) = (
        counted.chains.table.len(),
        counted.chains.prunes,
        counted.chains.floor,
    );
    let r = &counted.repeats;
    let (skipped, sentences, kept) = (r.skipped, r.read, r.kept());
    vec![
        format!(
            "text {iso}: {tokens} weighted tokens from {lines} Tatoeba lines ({held} held back) and {bytes} bytes of \
             Wikipedia; {live} chains counted, the table pruned {prunes} times up to a floor of {floor}"
        ),
        format!(
            "repeats {iso}: {skipped} of {sentences} sentences skipped as repeats, {kept} remembered"
        ),
        tally_line("skew", iso, &read.skewed),
    ]
}

/// The line naming the most seen one-word contexts of `iso` with their first followers.
fn top_line(iso: &str, list: &WordList, picked: &Picked, manifest: &Manifest) -> String {
    let t = &manifest.text;
    let spelling = |w: u32| {
        let word = usize::try_from(w).ok().and_then(|w| list.words.get(w));
        word.map_or("", |w| w.spelling.as_str())
    };
    let shown: Vec<String> = chains::top(picked, t.report_contexts, t.report_followers)
        .into_iter()
        .map(|(context, next)| {
            let next: Vec<&str> = next.into_iter().map(spelling).collect();
            format!("{} → {}", spelling(context), next.join(", "))
        })
        .collect();
    format!("top {iso}: {}", shown.join("; "))
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
