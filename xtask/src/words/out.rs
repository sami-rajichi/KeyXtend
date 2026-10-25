//! The build's output: each pack's bytes, the files written into the out folder, the packs a past run left there
//! removed, and the folder measured again for the download check.

use std::path::Path;

use kx_mod_predict::{Flags, PackWriter, WriteError};
use kx_module_api::ports::PACK_EXT;

use super::chains::Picked;
use super::list::WordList;
use super::{WordsError, download, io_error};

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
    let refused = |e: WriteError| WordsError::Pack(format!("{iso}: {e}"));
    let mut writer = PackWriter::new(iso, flags, followers).map_err(refused)?;
    for word in &list.words {
        writer
            .add_word(&word.spelling, word.freq)
            .map_err(refused)?;
    }
    for context in &picked.contexts {
        let words = (context.words.iter())
            .map(|&w| spelling(iso, list, w))
            .collect::<Result<Vec<&str>, _>>()?;
        for &(next, score) in &context.followers {
            let next = spelling(iso, list, next)?;
            writer.add_chain(&words, next, score).map_err(refused)?;
        }
    }
    writer.finish().map_err(refused)
}

/// The spelling of the word at place `at` of `iso`'s `list`; a place past the list is a build bug.
pub(crate) fn spelling<'l>(iso: &str, list: &'l WordList, at: u32) -> Result<&'l str, WordsError> {
    let word = usize::try_from(at).ok().and_then(|i| list.words.get(i));
    let missing = || WordsError::Pack(format!("{iso}: no word at list place {at}"));
    word.map(|w| w.spelling.as_str()).ok_or_else(missing)
}

/// Writes each named file into `out`, then removes every other pack file there, saying each one removed.
pub(crate) fn write(
    out: &Path,
    files: &[(String, Vec<u8>)],
    say: &mut dyn FnMut(&str),
) -> Result<(), WordsError> {
    std::fs::create_dir_all(out).map_err(|e| io_error(out, &e))?;
    for (name, bytes) in files {
        let path = out.join(name);
        std::fs::write(&path, bytes).map_err(|e| io_error(&path, &e))?;
    }
    let written: Vec<&str> = files.iter().map(|(name, _)| name.as_str()).collect();
    for name in stale(&packs_in(out)?, &written) {
        let path = out.join(name);
        std::fs::remove_file(&path).map_err(|e| io_error(&path, &e))?;
        say(&format!(
            "stale: removed {name}, which this run did not build"
        ));
    }
    Ok(())
}

/// The `listed` packs no `written` name names. A name that differs only in case is the written file where the folder
/// ignores case (Windows, APFS), so it stays unless the written name is listed beside it.
pub(crate) fn stale<'a>(listed: &'a [String], written: &[&str]) -> Vec<&'a str> {
    let is_listed = |w: &&str| listed.iter().any(|l| l == w);
    let names = |l: &str| {
        written
            .iter()
            .any(|w| *w == l || (w.eq_ignore_ascii_case(l) && !is_listed(w)))
    };
    listed
        .iter()
        .map(String::as_str)
        .filter(|l| !names(l))
        .collect()
}

/// The raw and deflated sizes of every pack file in `out` and of the other files `named`.
pub(crate) fn measure(
    out: &Path,
    named: &[String],
    level: u32,
) -> Result<Vec<(usize, usize)>, WordsError> {
    let mut files = packs_in(out)?;
    files.extend(named.iter().cloned());
    let mut sizes = Vec::with_capacity(files.len());
    for name in files {
        let path = out.join(name);
        let bytes = std::fs::read(&path).map_err(|e| io_error(&path, &e))?;
        sizes.push((bytes.len(), download::deflated(&bytes, level)?));
    }
    Ok(sizes)
}

/// The names of the pack files in `out`, sorted.
fn packs_in(out: &Path) -> Result<Vec<String>, WordsError> {
    let entries = std::fs::read_dir(out).map_err(|e| io_error(out, &e))?;
    let mut names = Vec::new();
    for entry in entries {
        let path = entry.map_err(|e| io_error(out, &e))?.path();
        let ext = path.extension().and_then(|x| x.to_str());
        let is_pack = ext.is_some_and(|x| x.eq_ignore_ascii_case(PACK_EXT)) && path.is_file();
        if let (true, Some(name)) = (is_pack, path.file_name().and_then(|n| n.to_str())) {
            names.push(name.to_string());
        }
    }
    names.sort();
    Ok(names)
}
