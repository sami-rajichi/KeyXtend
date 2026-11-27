//! The Snips folder on disk (ADR-0056): its unsaved part's pictures and videos with their ages and copy marks for the
//! sweep, marks and deletes there alone, and its drive's room. A hidden run has none (ADR-0037), and no path prints.

use crate::models::Disk;
use crate::unsaved_dir;
use kx_module_api::ports::{FileKind, FolderPath, SaveError, SnipFile, SnipFolder, UnsavedFile};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, SystemTime};

/// What a copy mark's name adds to its video's: `3.mp4` is marked by `3.mp4.copied`, an empty file.
const MARK: &str = ".copied";

/// The room a drive has, as the system tells it.
pub trait Room: Send + Sync {
    /// Bytes free to us on the drive that holds `folder`.
    ///
    /// # Errors
    /// The system's error, `NotFound` for a folder that is gone.
    fn free(&self, folder: &Path) -> io::Result<u64>;
}

impl<D: Disk + ?Sized> Room for D {
    fn free(&self, folder: &Path) -> io::Result<u64> {
        Disk::free(self, folder)
    }
}

/// The Snips folder at its place in the data folder, or none in a hidden run.
pub struct DiskSnips {
    snips: Option<FolderPath>,
    room: Arc<dyn Room>,
}

impl DiskSnips {
    /// The Snips folder `snips`, `None` in a hidden run, on a drive whose room `room` tells.
    #[must_use]
    pub fn new(snips: Option<FolderPath>, room: Arc<dyn Room>) -> Self {
        Self { snips, room }
    }
}

impl SnipFolder for DiskSnips {
    fn snips(&self) -> Option<FolderPath> {
        self.snips.clone()
    }

    fn unsaved(&self) -> Result<Vec<UnsavedFile>, SaveError> {
        let part = unsaved_part(self).ok_or(SaveError::NoFolder)?;
        let entries = fs::read_dir(part.expose()).map_err(|e| read_fault(&e))?;
        let now = SystemTime::now();
        let held = entries.filter_map(Result::ok);
        Ok(held.filter_map(|entry| aged(&entry, now)).collect())
    }

    fn mark_copied(&self, video: &SnipFile) -> Result<(), SaveError> {
        let part = unsaved_part(self).ok_or(SaveError::NoFolder)?;
        if video.kind() != FileKind::Mp4 || !video.lies_in(&part) {
            return Err(SaveError::BadPath);
        }
        if !plain(video.expose()) {
            return Err(SaveError::Missing);
        }
        // An old mark or a planted link goes first, never what a link reaches; the new mark is a file of its own.
        let mark = mark_of(video.expose());
        delete(&mark)?;
        let made = fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&mark);
        made.map(drop).map_err(|e| save_fault(&e))
    }

    fn prune_marks(&self) -> Result<(), SaveError> {
        let part = unsaved_part(self).ok_or(SaveError::NoFolder)?;
        let entries = fs::read_dir(part.expose()).map_err(|e| read_fault(&e))?;
        let marks = entries.filter_map(Result::ok).filter_map(|e| video_of(&e));
        let orphans = marks.filter(|(_, video)| !plain(video));
        let gone = orphans.map(|(mark, _)| delete(&mark));
        gone.fold(Ok(()), Result::and)
    }

    fn remove(&self, file: &SnipFile) -> Result<(), SaveError> {
        let part = unsaved_part(self).ok_or(SaveError::NoFolder)?;
        if !file.lies_in(&part) {
            return Err(SaveError::BadPath);
        }
        delete(&mark_of(file.expose()))?;
        delete(file.expose())
    }

    fn free(&self) -> Result<u64, SaveError> {
        let snips = self.snips.as_ref().ok_or(SaveError::NoFolder)?;
        self.room.free(snips.expose()).map_err(|e| read_fault(&e))
    }
}

/// `folder`'s unsaved part; `None` in a hidden run.
pub(crate) fn unsaved_part(folder: &dyn SnipFolder) -> Option<FolderPath> {
    let snips = folder.snips()?;
    FolderPath::new(unsaved_dir(snips.expose())).ok()
}

/// The error a write fails with: a missing folder or drive, one that cannot be written, a full drive, or another.
pub(crate) fn save_fault(e: &io::Error) -> SaveError {
    match e.kind() {
        io::ErrorKind::NotFound => SaveError::Missing,
        io::ErrorKind::PermissionDenied | io::ErrorKind::ReadOnlyFilesystem => SaveError::ReadOnly,
        io::ErrorKind::StorageFull => SaveError::Full,
        _ => SaveError::Io,
    }
}

/// The error a read fails with: a missing folder or drive, or another.
fn read_fault(e: &io::Error) -> SaveError {
    match e.kind() {
        io::ErrorKind::NotFound => SaveError::Missing,
        _ => SaveError::Io,
    }
}

/// `entry` as an unsaved file aged at `now`, with its copy mark's age: a plain picture or video, never a link or a
/// folder.
fn aged(entry: &fs::DirEntry, now: SystemTime) -> Option<UnsavedFile> {
    entry.file_type().ok().filter(fs::FileType::is_file)?;
    let path = entry.path();
    let mut kinds = [FileKind::Png, FileKind::Mp4].into_iter();
    let file = kinds.find_map(|kind| SnipFile::new(path.clone(), kind))?;
    let age = since(entry.metadata().ok()?.modified().ok()?, now);
    let mark = fs::symlink_metadata(mark_of(&path))
        .ok()
        .filter(fs::Metadata::is_file);
    let copied = mark
        .and_then(|m| m.modified().ok())
        .map(|at| since(at, now));
    Some(UnsavedFile { file, age, copied })
}

/// How long before `now` `at` was; zero for a time from the future.
fn since(at: SystemTime, now: SystemTime) -> Duration {
    now.duration_since(at).unwrap_or_default()
}

/// The copy mark beside `video`: its name with `MARK` after it.
fn mark_of(video: &Path) -> PathBuf {
    let mut name = video.as_os_str().to_owned();
    name.push(MARK);
    PathBuf::from(name)
}

/// `entry` as a copy mark and the video it names: a plain file whose name ends in `MARK`.
fn video_of(entry: &fs::DirEntry) -> Option<(PathBuf, PathBuf)> {
    entry.file_type().ok().filter(fs::FileType::is_file)?;
    let mark = entry.path();
    let video = mark.to_str()?.strip_suffix(MARK)?.to_owned();
    Some((mark, PathBuf::from(video)))
}

/// True when `path` is a plain file, never a link or a folder.
fn plain(path: &Path) -> bool {
    fs::symlink_metadata(path).is_ok_and(|m| m.is_file())
}

/// Deletes the file at `path`; one already gone is no error.
fn delete(path: &Path) -> Result<(), SaveError> {
    match fs::remove_file(path) {
        Err(e) if e.kind() != io::ErrorKind::NotFound => Err(save_fault(&e)),
        _ => Ok(()),
    }
}

#[cfg(test)]
mod tests;
