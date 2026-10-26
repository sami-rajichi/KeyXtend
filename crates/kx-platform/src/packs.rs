//! Word packs read straight from disk (ADR-0032): each language's `pack_file` from the first folder that has it, with
//! the folder order, the name and the size rules every OS shares. An adapter only opens and maps a file (`PackDisk`).

use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use kx_module_api::ports::{
    IndexError, MAX_PACK_BYTES, PACK_INDEX, PackError, PackIndex, PackView, WordPacks, fits_iso,
    pack_file,
};

/// The OS side of word packs: open a pack file, then give a view whose bytes stay unchanged while it lives
/// (`PackView::bytes`).
pub trait PackDisk: Send + Sync {
    /// One open pack file.
    type File;
    /// One mapped pack.
    type View: PackView + 'static;

    /// The plain file at `path`, opened for reading, and its length; `Ok(None)` when none is there.
    ///
    /// # Errors
    /// `Unreadable` when it is no plain file or cannot be opened.
    fn open(&self, path: &Path) -> Result<Option<(Self::File, u64)>, PackError>;

    /// A view of the first `len` bytes of `file` that stay unchanged while it lives, by a map that keeps writers out
    /// or a copy.
    ///
    /// # Errors
    /// `Unreadable` when the OS refuses the map.
    fn map(&self, file: Self::File, len: usize) -> Result<Self::View, PackError>;
}

/// The word packs in a list of folders, the first folder first, over an OS's `PackDisk`.
/// It has no `Debug`, as its folders name the user.
#[derive(Clone)]
pub struct DiskPacks<D> {
    disk: D,
    dirs: Vec<PathBuf>,
    /// What the folders' indexes list, read at the first `pack_of`.
    shipped: OnceLock<PackIndex>,
}

impl<D: PackDisk> DiskPacks<D> {
    /// The packs in `dirs`, looked up in that order on `disk`.
    #[must_use]
    pub fn new(disk: D, dirs: Vec<PathBuf>) -> Self {
        Self {
            disk,
            dirs,
            shipped: OnceLock::new(),
        }
    }
}

impl<D: PackDisk> WordPacks for DiskPacks<D> {
    fn open(&self, iso: &str) -> Result<Option<Box<dyn PackView>>, PackError> {
        let view = find(&self.disk, &self.dirs, iso)?;
        Ok(view.map(|v| Box::new(v) as Box<dyn PackView>))
    }

    fn pack_of(&self, iso: &str) -> Option<String> {
        let shipped = self.shipped.get_or_init(|| shipped(&self.disk, &self.dirs));
        let pack = fits_iso(iso).then(|| shipped.pack_of(iso)).flatten();
        pack.map(str::to_owned)
    }
}

/// What the indexes in `dirs` list together; one that does not read is logged and adds nothing, and a relative folder
/// is never read.
fn shipped<D: PackDisk>(disk: &D, dirs: &[PathBuf]) -> PackIndex {
    let (mut all, mut found) = (PackIndex::default(), false);
    for dir in dirs.iter().filter(|d| d.is_absolute()) {
        match index(disk, &dir.join(PACK_INDEX)) {
            Ok(None) => {}
            Ok(Some(index)) => {
                all.merge(index);
                found = true;
            }
            Err(error) => {
                found = true;
                tracing::warn!(%error, "word pack index not read");
            }
        }
    }
    if !found {
        tracing::warn!(error = %IndexError::Missing, "word pack index not read");
    }
    all
}

/// What the index at `path` lists; `Ok(None)` when no file is there, and an empty file is unreadable, as a pack is.
fn index<D: PackDisk>(disk: &D, path: &Path) -> Result<Option<PackIndex>, IndexError> {
    let unreadable = |_| IndexError::Unreadable;
    let Some((file, len)) = disk.open(path).map_err(unreadable)? else {
        return Ok(None);
    };
    let view = disk.map(file, view_len(len).map_err(unreadable)?);
    PackIndex::read(view.map_err(unreadable)?.bytes()).map(Some)
}

/// A pack file's `len` as its view's length; `Unreadable` when the file is empty or over `MAX_PACK_BYTES`.
fn view_len(len: u64) -> Result<usize, PackError> {
    if len == 0 || len > MAX_PACK_BYTES {
        return Err(PackError::Unreadable);
    }
    usize::try_from(len).map_err(|_| PackError::Unreadable)
}

/// `iso`'s pack from the first of `dirs` that has it, whose answer stands even when it is an error.
/// A bad code or a relative folder never reaches the disk, so the working folder never supplies a pack.
fn find<D: PackDisk>(disk: &D, dirs: &[PathBuf], iso: &str) -> Result<Option<D::View>, PackError> {
    let Some(name) = pack_file(iso) else {
        return Ok(None);
    };
    for dir in dirs.iter().filter(|d| d.is_absolute()) {
        if let Some((file, len)) = disk.open(&dir.join(&name))? {
            return disk.map(file, view_len(len)?).map(Some);
        }
    }
    Ok(None)
}

#[cfg(test)]
mod tests;
