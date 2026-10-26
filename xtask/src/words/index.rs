//! The pack index (ADR-0032): which languages the build shipped and the layout codes each serves, written beside the
//! packs with the credits, and rewritten from a folder's packs by `words index`.

use std::collections::BTreeMap;
use std::path::Path;

use kx_mod_predict::Pack;
use kx_module_api::ports::{PACK_INDEX, PackIndex, pack_file};

use super::{WordsError, io_error, out};

/// The index of `isos`, each serving the layout codes `aliases` gives it, as a named file beside the packs.
pub(crate) fn file(
    isos: &[&str],
    aliases: &BTreeMap<String, Vec<String>>,
) -> Result<(String, Vec<u8>), WordsError> {
    let index = PackIndex::new(isos, aliases);
    let index = index.map_err(|e| WordsError::Pack(format!("{PACK_INDEX}: {e}")))?;
    Ok((PACK_INDEX.to_owned(), index.text().into_bytes()))
}

/// Rewrites the index in `dir` from the packs there, with their `aliases`; each pack must parse and carry the language
/// its file names.
pub(crate) fn run(
    dir: &Path,
    aliases: &BTreeMap<String, Vec<String>>,
    say: &mut dyn FnMut(&str),
) -> Result<(), WordsError> {
    let isos = out::packs_in(dir)?
        .iter()
        .map(|name| lang(dir, name))
        .collect::<Result<Vec<String>, _>>()?;
    let isos: Vec<&str> = isos.iter().map(String::as_str).collect();
    let (name, bytes) = file(&isos, aliases)?;
    let path = dir.join(&name);
    std::fs::write(&path, bytes).map_err(|e| io_error(&path, &e))?;
    say(&format!("index: {} languages in {name}", isos.len()));
    Ok(())
}

/// The language of the pack `name` in `dir`; an error unless it parses and its file is named after that language.
fn lang(dir: &Path, name: &str) -> Result<String, WordsError> {
    let path = dir.join(name);
    let bytes = std::fs::read(&path).map_err(|e| io_error(&path, &e))?;
    let pack = Pack::parse(&bytes).map_err(|e| WordsError::Pack(format!("{name}: {e}")))?;
    let iso = pack.iso();
    if pack_file(iso).as_deref() != Some(name) {
        return Err(WordsError::Pack(format!("{name} holds language {iso:?}")));
    }
    Ok(iso.to_owned())
}
