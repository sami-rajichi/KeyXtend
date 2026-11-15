//! The folder reveal's rule (ADR-0046): a folder on a fixed local drive, with no reparse point in it or above it, held
//! open while Explorer explores it, a verb only folders have. Pure over `Plain`; only `win` calls Windows.

use std::path::{Path, PathBuf};

use kx_module_api::ports::{FolderError, FolderPath};

/// A drive path's forward separator, which the shell may read apart from `BACKSLASH`.
const SLASH: char = '/';
/// The separator every check and the shell are handed.
const BACKSLASH: &str = "\\";

/// What a path is, read without following a link.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub(crate) enum Found {
    /// A folder that exists and is no reparse point.
    #[default]
    Folder,
    /// A link, a junction or another reparse point, which may lead anywhere.
    Linked,
    /// Nothing, a file, or anything else.
    Other,
}

/// What a path is from what a look without links tells: whether it is a folder and whether it is a reparse point.
pub(crate) const fn found(is_dir: bool, reparse: bool) -> Found {
    match (is_dir, reparse) {
        (_, true) => Found::Linked,
        (true, false) => Found::Folder,
        (false, false) => Found::Other,
    }
}

/// The OS calls behind the folder reveal: Windows in `win`, a recorder in tests.
pub(crate) trait Plain {
    /// The running file manager's shell object.
    type Shell;
    /// A plain folder held open, so it cannot be renamed or removed until dropped.
    type Hold;

    /// True when `folder` lies on a fixed local drive, never a network, removable or optical one.
    fn local(&self, folder: &Path) -> bool;

    /// What `folder` is, read without following a link.
    fn found(&self, folder: &Path) -> Found;

    /// Holds `folder` open when it is a plain folder, read without following a link; else what it is.
    fn hold(&self, folder: &Path) -> Result<Self::Hold, Found>;

    /// The running file manager's shell object, or the code that refused it.
    fn shell(&self) -> Result<Self::Shell, i32>;

    /// Asks `shell` to explore `folder`, or gives the code that refused it.
    fn explore(&self, shell: &Self::Shell, folder: &Path) -> Result<(), i32>;
}

/// Shows `folder` only when it is plain: `BadPath` off a fixed local drive, then `Linked` or `Missing` for any part
/// from the drive down, all before the shell is asked; else `Refused` when the shell says no.
pub(crate) fn show_plain<O: Plain>(os: &O, folder: &FolderPath) -> Result<(), FolderError> {
    let text = folder.expose().to_string_lossy().replace(SLASH, BACKSLASH);
    let path = PathBuf::from(text);
    if !os.local(&path) {
        return Err(FolderError::BadPath);
    }
    let mut above: Vec<&Path> = path.ancestors().skip(1).collect();
    above.reverse();
    if let Some(odd) = above
        .into_iter()
        .map(|p| os.found(p))
        .find(|f| *f != Found::Folder)
    {
        return Err(refusal(odd));
    }
    let _held = os.hold(&path).map_err(refusal)?;
    let shell = os.shell().map_err(FolderError::Refused)?;
    os.explore(&shell, &path).map_err(FolderError::Refused)
}

/// The error for a part that is not a plain folder.
const fn refusal(found: Found) -> FolderError {
    match found {
        Found::Linked => FolderError::Linked,
        Found::Folder | Found::Other => FolderError::Missing,
    }
}

#[cfg(test)]
mod tests;
