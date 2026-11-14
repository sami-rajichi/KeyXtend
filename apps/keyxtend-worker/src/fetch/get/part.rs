//! A download's `.part` file: opened or made at the start, and at the end checked and named, deleted, or kept for a
//! resume. A named file exists only once its size and SHA-256 passed, and it never replaces another file.

use super::Load;
use kx_ipc::{FetchFault, FetchNote};
use std::fs::{self, File, OpenOptions};
use std::io::{self, ErrorKind};
use std::path::{Component, Path, PathBuf, is_separator};

/// The faults that delete the part: its bytes are wrong, or the user dropped it.
const DELETES: [FetchFault; 3] = [
    FetchFault::BadSize,
    FetchFault::BadHash,
    FetchFault::Cancelled,
];

/// How a download ended: its last note, and the part it left for a resume.
pub struct End {
    /// `Done` or a `Fault`.
    pub note: FetchNote,
    /// The part, when a fault kept bytes in it.
    pub left: Option<PathBuf>,
}

impl Load {
    /// Ends on `fault`: a bad size or fingerprint or a cancel deletes the part, as does an empty one; else it stays.
    pub fn fail(self, fault: FetchFault) -> End {
        let id = self.row.id;
        let mut left = self.leave();
        if DELETES.contains(&fault)
            && let Some(part) = left.take()
        {
            remove(&part);
        }
        let note = FetchNote::Fault { id, fault };
        End { note, left }
    }

    /// Checks the part's size and SHA-256, flushes it, then gives it its name; a bad one is deleted, and a name taken
    /// meanwhile or a failed rename keeps it.
    pub fn name(self) -> End {
        let sized = self.file.metadata().is_ok_and(|m| m.len() == self.row.size);
        if !sized {
            return self.fail(FetchFault::BadSize);
        }
        if self.print.clone().done() != self.row.want {
            return self.fail(FetchFault::BadHash);
        }
        if let Err(e) = self.file.sync_all() {
            return self.fail(disk(&e));
        }
        let Self { row, file, .. } = self;
        drop(file);
        let named = if taken(&row.path) {
            Err(FetchFault::Disk)
        } else {
            fs::rename(&row.part, &row.path).map_err(|e| disk(&e))
        };
        let (id, left) = (row.id, named.is_err().then_some(row.part));
        let note = named.map_or_else(
            |fault| FetchNote::Fault { id, fault },
            |()| FetchNote::Done { id },
        );
        End { note, left }
    }

    /// Stops here, closing the request and the part; the part stays when it holds bytes, for a resume.
    pub fn leave(self) -> Option<PathBuf> {
        let empty = self.file.metadata().is_ok_and(|m| m.len() == 0);
        let Self { file, row, .. } = self;
        drop(file);
        if empty {
            remove(&row.part);
            return None;
        }
        Some(row.part)
    }
}

/// True for a path the job may write beside: absolute, ending in a file's name and never climbing with `..`, so its
/// part sits in the folder it names. The host checks the rest (`kx_module_api::ports::path_ok`).
pub(super) fn plain(text: &str) -> bool {
    let path = Path::new(text);
    let climbs = path.components().any(|c| c == Component::ParentDir);
    path.is_absolute() && !climbs && !text.ends_with(is_separator)
}

/// True when anything, a link included, already has the name `path`.
pub(super) fn taken(path: impl AsRef<Path>) -> bool {
    fs::symlink_metadata(path).is_ok()
}

/// Opens `part` to read and write, made when missing, and gives its length; a part longer than `size` is emptied.
///
/// # Errors
/// The file's error, or `InvalidInput` for a link or a folder at its name, which is never followed.
pub(super) fn open(part: &Path, size: u64) -> io::Result<(File, u64)> {
    if fs::symlink_metadata(part).is_ok_and(|m| !m.is_file()) {
        return Err(ErrorKind::InvalidInput.into());
    }
    let mut options = OpenOptions::new();
    let file = options
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .open(part)?;
    let len = file.metadata()?.len();
    if len <= size {
        return Ok((file, len));
    }
    file.set_len(0)?;
    Ok((file, 0))
}

/// The fault a file error is: a full drive or quota, else `Disk`.
pub(super) fn disk(e: &io::Error) -> FetchFault {
    match e.kind() {
        ErrorKind::StorageFull | ErrorKind::QuotaExceeded => FetchFault::DiskFull,
        _ => FetchFault::Disk,
    }
}

/// Deletes `part`; one already gone is no error, and the job has no other way to tell a failure.
pub fn remove(part: &Path) {
    let _ = fs::remove_file(part);
}
