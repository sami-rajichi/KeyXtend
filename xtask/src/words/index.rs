//! The pack index (ADR-0032): which languages the build shipped, written beside the packs with the credits, and
//! rewritten from a folder's packs by `words index`.

use std::path::Path;

use kx_mod_predict::Pack;
use kx_module_api::ports::{PACK_INDEX, index_text, pack_file};

use super::{WordsError, io_error, out};

/// The index of `isos`, as a named file beside the packs.
pub(crate) fn file(isos: &[&str]) -> Result<(String, Vec<u8>), WordsError> {
    let text = index_text(isos).map_err(|e| WordsError::Pack(format!("{PACK_INDEX}: {e}")))?;
    Ok((PACK_INDEX.to_owned(), text.into_bytes()))
}

/// Rewrites the index in `dir` from the packs there, each of which must parse and carry the language its file names.
pub(crate) fn run(dir: &Path, say: &mut dyn FnMut(&str)) -> Result<(), WordsError> {
    let isos = out::packs_in(dir)?
        .iter()
        .map(|name| lang(dir, name))
        .collect::<Result<Vec<String>, _>>()?;
    let isos: Vec<&str> = isos.iter().map(String::as_str).collect();
    let (name, bytes) = file(&isos)?;
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
