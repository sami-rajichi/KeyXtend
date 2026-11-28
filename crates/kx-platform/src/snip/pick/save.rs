//! The file a Save dialog answered, the one place outside the Snips folder a snip is written (edge 6): a picture's
//! bytes, or a copy of a plain video of the unsaved part, never what a link there reaches.

use crate::snip::{save_fault, unsaved_part};
use kx_module_api::Redacted;
use kx_module_api::ports::{FileKind, SaveError, SaveTo, SnipFile, SnipFolder};
use std::fs::{self, File};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process;
use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering};

/// The extension of the part file a Save writes before it takes the chosen name.
const PART_EXT: &str = "kxpart";
/// How many part names a Save tries when others are already there, as stale ones left by a crash.
pub(super) const PART_TRIES: u32 = 8;
/// Numbers this process's part files, so each attempt has a name of its own.
static PARTS: AtomicU32 = AtomicU32::new(0);

/// The file the user chose, and the Snips folder whose unsaved part alone it copies videos from.
pub(super) struct DiskSaveTo {
    file: SnipFile,
    folder: Arc<dyn SnipFolder>,
}

impl DiskSaveTo {
    /// The chosen `file`, copying videos from `folder`'s unsaved part.
    pub(super) fn new(file: SnipFile, folder: Arc<dyn SnipFolder>) -> Self {
        Self { file, folder }
    }

    /// Refuses a write of another kind than the answer's.
    fn takes(&self, kind: FileKind) -> Result<(), SaveError> {
        (self.file.kind() == kind)
            .then_some(())
            .ok_or(SaveError::BadPath)
    }
}

impl SaveTo for DiskSaveTo {
    fn file(&self) -> &SnipFile {
        &self.file
    }

    fn png(&self, png: &Redacted<Vec<u8>>) -> Result<(), SaveError> {
        self.takes(FileKind::Png)?;
        let written = put(self.file.expose(), |to| to.write_all(png.expose()));
        written.map_err(|e| save_fault(&e))
    }

    fn copy(&self, video: &SnipFile) -> Result<(), SaveError> {
        self.takes(FileKind::Mp4)?;
        let inside = unsaved_part(&*self.folder).is_some_and(|u| video.lies_in(&u));
        // A link or a folder there is refused, never followed; a file that is gone reads as `Missing` below.
        let linked = fs::symlink_metadata(video.expose()).is_ok_and(|m| !m.is_file());
        if video.kind() != FileKind::Mp4 || !inside || linked {
            return Err(SaveError::BadPath);
        }
        let copied = File::open(video.expose())
            .and_then(|mut from| put(self.file.expose(), |to| io::copy(&mut from, to).map(drop)));
        copied.map_err(|e| save_fault(&e))
    }
}

/// Writes `to` whole with what `body` writes, through a new part file beside it renamed over it; a write that fails
/// deletes the part and leaves `to` as it was.
pub(super) fn put(to: &Path, body: impl FnOnce(&mut File) -> io::Result<()>) -> io::Result<()> {
    put_by(to, &PARTS, body)
}

/// `put`, numbering its part files from `parts`.
pub(super) fn put_by(
    to: &Path,
    parts: &AtomicU32,
    body: impl FnOnce(&mut File) -> io::Result<()>,
) -> io::Result<()> {
    let (part, mut file) = fresh(to, parts)?;
    let written = body(&mut file).and_then(|()| file.sync_all());
    drop(file);
    let placed = written.and_then(|()| fs::rename(&part, to));
    if placed.is_err() && fs::remove_file(&part).is_err() {
        tracing::warn!("a Save's part file could not be removed");
    }
    placed
}

/// A new part file beside `to`, numbered from `parts`; a name already there is skipped, `PART_TRIES` at most.
fn fresh(to: &Path, parts: &AtomicU32) -> io::Result<(PathBuf, File)> {
    let mut taken = io::Error::from(io::ErrorKind::AlreadyExists);
    for _ in 0..PART_TRIES {
        let part = part_of(to, parts.fetch_add(1, Ordering::Relaxed));
        match File::options().write(true).create_new(true).open(&part) {
            Ok(file) => return Ok((part, file)),
            Err(e) if e.kind() == io::ErrorKind::AlreadyExists => taken = e,
            Err(e) => return Err(e),
        }
    }
    Err(taken)
}

/// The part file numbered `n` that `to` is written through: `<name>.<process id>-<n>.PART_EXT`, beside it.
pub(super) fn part_of(to: &Path, n: u32) -> PathBuf {
    let mut name = to.as_os_str().to_owned();
    name.push(format!(".{}-{n}.{PART_EXT}", process::id()));
    PathBuf::from(name)
}
