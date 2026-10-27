//! The word packs in the stage (ADR-0032): the index, each pack it lists and the credits go from the folder
//! `cargo xtask words` writes to into `PACK_DIR` beside the program; the check refuses a stage missing any of them.

use std::collections::BTreeSet;
use std::ffi::OsStr;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};

use kx_module_api::ports::{PACK_DIR, PACK_INDEX, PackIndex, pack_file};

use super::{Copy, StageError};
use crate::devtools::{DevError, paths};

/// The folder the packs come from and the credit files that ship beside them.
#[derive(Debug)]
pub(crate) struct Words {
    /// The folder `cargo xtask words` writes the packs to.
    pub dir: PathBuf,
    /// The credit files written beside the packs.
    pub credits: Vec<String>,
}

impl Words {
    /// The packs folder under the workspace `root` and the credits, as the words manifest names them.
    pub(crate) fn load(root: &Path) -> Result<Self, DevError> {
        let (out, credits) =
            crate::words::shipped().map_err(|e| DevError::Config(e.to_string()))?;
        let dir = paths::join_rel(root.to_path_buf(), &out);
        Ok(Self { dir, credits })
    }
}

/// The copies into `PACK_DIR`: the index, each pack it lists and the credits.
pub(crate) fn plan(words: &Words) -> Result<Vec<Copy>, DevError> {
    let names = names(&words.dir, &words.credits, &words.dir)?;
    let copy = |name: String| Copy {
        from: words.dir.join(&name),
        to: Path::new(PACK_DIR).join(name),
    };
    Ok(names.into_iter().map(copy).collect())
}

/// Checks the staged packs folder: its index reads, each pack it lists and every credit is there, and nothing else is.
/// Names match exactly, as the platform opens them, so a case variant is refused on any disk.
pub(crate) fn check(stage: &Path, credits: &[String]) -> Result<(), DevError> {
    let (dir, shown) = (stage.join(PACK_DIR), Path::new(PACK_DIR));
    let want = names(&dir, credits, shown)?;
    if let Some(gone) = want.iter().find(|n| !dir.join(n).is_file()) {
        return Err(StageError::NoWords(paths::text(&shown.join(gone))).into());
    }
    let listed: BTreeSet<PathBuf> = want.iter().map(PathBuf::from).collect();
    let files = paths::files(&dir)?;
    match files.into_iter().find(|f| !listed.contains(f)) {
        Some(extra) => Err(StageError::Unplanned(paths::text(&shown.join(extra))).into()),
        None => Ok(()),
    }
}

/// True when the stage-relative `file` lies in `PACK_DIR`, named exactly as `check` reads it.
pub(crate) fn inside(file: &Path) -> bool {
    let mut parts = file.iter();
    parts.next() == Some(OsStr::new(PACK_DIR)) && parts.next().is_some()
}

/// The files a packs folder `dir` holds: the index, each pack it lists, then `credits`; a refusal names them under
/// `shown`. A missing index reads as missing, any other read error keeps its own text, and one the index's rules refuse
/// (one listing no pack too) is a bad index.
fn names(dir: &Path, credits: &[String], shown: &Path) -> Result<Vec<String>, DevError> {
    let index = paths::text(&shown.join(PACK_INDEX));
    let bytes = std::fs::read(dir.join(PACK_INDEX)).map_err(|e| match e.kind() {
        ErrorKind::NotFound => DevError::from(StageError::NoWords(index.clone())),
        _ => DevError::Io(format!("{index}: {e}")),
    })?;
    let isos = PackIndex::read(&bytes).map_err(|_| StageError::BadIndex(index))?;
    let packs = isos.packs().filter_map(pack_file);
    let names = std::iter::once(PACK_INDEX.to_string()).chain(packs);
    Ok(names.chain(credits.iter().cloned()).collect())
}

#[cfg(test)]
mod tests;
