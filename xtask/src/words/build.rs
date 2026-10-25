//! `words build`: once every source checks out, each language's text is counted, its list finished and its chains
//! picked into a pack; then the credits and the index of the packs, all written once the download fits, and packs a
//! past run left are removed.

use std::collections::BTreeSet;
use std::path::PathBuf;

use kx_mod_predict::Casing;
use kx_module_api::ports::pack_file;

use super::cands::Candidates;
use super::chains::{self, Picked};
use super::count::{Counted, Counter};
use super::list::{self, WordList};
use super::manifest::{LangText, Manifest};
use super::read::{self, Read};
use super::report::{report, tally_line};
use super::vocab::Vocab;
use super::{WordsError, candidates, credits, download, fetch, index, out};
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

/// Checks every source, builds every language's pack and the credits, saying one line per fact, and writes them
/// into `paths.out` once their deflated total fits the download budget; the folder is then measured again.
pub(crate) fn run(
    manifest: &Manifest,
    paths: &Paths,
    say: &mut dyn FnMut(&str),
) -> Result<(), WordsError> {
    let sources = credits::sources(manifest);
    let rels: Vec<&str> = sources.iter().map(String::as_str).collect();
    fetch::verify(manifest, &paths.src, &rels)?;
    let level = manifest.packs.level;
    let mut authors = BTreeSet::new();
    let (mut files, mut sizes) = (Vec::new(), Vec::new());
    let isos = manifest.isos();
    for iso in &isos {
        let (pack, deflated, lines) = lang(manifest, iso, paths, &mut authors)?;
        for line in &lines {
            say(line);
        }
        let name = pack_file(iso).ok_or_else(|| WordsError::Lang(iso.clone()))?;
        sizes.push((pack.len(), deflated));
        files.push((name, pack));
    }
    let mut credits = credits::files(manifest, &paths.src, &authors)?;
    let listed: Vec<&str> = isos.iter().map(String::as_str).collect();
    credits.push(index::file(&listed)?);
    for (_, bytes) in &credits {
        sizes.push((bytes.len(), download::deflated(bytes, level)?));
    }
    download::check(&sizes, &manifest.packs)?;
    let named: Vec<String> = credits.iter().map(|c| c.0.clone()).collect();
    files.extend(credits);
    out::write(&paths.out, &files, say)?;
    say(&download::check(
        &out::measure(&paths.out, &named, level)?,
        &manifest.packs,
    )?);
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
    let pack = out::write_pack(iso, &list, &picked, manifest.text.followers)?;
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
    let vocab = Vocab::new(&cands.cands, Casing::of(iso));
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
    let budget = text.chain_budget_kb.saturating_mul(BYTES_PER_KB);
    let budget = usize::try_from(budget).unwrap_or(usize::MAX);
    let (table, seen) = (found.chains.table, &found.seen);
    let picked = chains::pick(table, seen, &list.of_cand, t.followers, budget, t.max_chain);
    lines.push(top_line(iso, &list, &picked, manifest)?);
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
fn top_line(
    iso: &str,
    list: &WordList,
    picked: &Picked,
    manifest: &Manifest,
) -> Result<String, WordsError> {
    let t = &manifest.text;
    let spelling = |w: u32| out::spelling(iso, list, w);
    let mut shown = Vec::new();
    for (context, next) in chains::top(picked, t.report_contexts, t.report_followers) {
        let next = next
            .into_iter()
            .map(spelling)
            .collect::<Result<Vec<_>, _>>()?;
        shown.push(format!("{} → {}", spelling(context)?, next.join(", ")));
    }
    Ok(format!("top {iso}: {}", shown.join("; ")))
}
