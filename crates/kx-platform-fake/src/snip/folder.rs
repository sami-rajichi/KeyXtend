//! A fake Snips folder: its files and copy marks live in a table in memory, so nothing touches the disk, each with when
//! it was written by an injected clock, for the ages the start's sweep reads; room and failures come as the test sets.

use crate::{lock, wait_for};
use kx_module_api::ports::{FileKind, FolderPath, SaveError, SnipFile, SnipFolder, UnsavedFile};
use kx_module_api::{Clock, Mono, Redacted};
use std::collections::BTreeMap;
use std::fmt;
use std::path::PathBuf;
use std::sync::{Arc, Condvar, Mutex};
use std::time::Duration;

/// What the fake holds in a file.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Stored {
    /// A picture's PNG bytes.
    Png(Redacted<Vec<u8>>),
    /// A video.
    Video {
        /// Its length, in ms.
        length_ms: u32,
    },
}

/// One file, what it holds and when it was last written.
struct Entry {
    file: SnipFile,
    stored: Stored,
    at: Mono,
}

/// The Snips folder and its unsaved part.
struct Dirs {
    snips: FolderPath,
    unsaved: FolderPath,
}

/// The files by path, and what the test set.
struct Table {
    /// `None` in a hidden run.
    dirs: Option<Dirs>,
    files: BTreeMap<PathBuf, Entry>,
    free: Result<u64, SaveError>,
    list_error: Option<SaveError>,
    remove_error: Option<SaveError>,
    /// The next removes refused, as a file another program still holds, and with what.
    held: Option<(SaveError, usize)>,
    /// Every file `remove` was asked, oldest first.
    removes: Vec<SnipFile>,
    /// The copy marks by their video's path, each with its video and when it was made.
    marks: BTreeMap<PathBuf, (SnipFile, Mono)>,
    /// The error later marks give.
    mark_error: Option<SaveError>,
}

/// A Snips folder shared by its clones, and by the fakes that write into it: the recorder, the dialogs' answers and
/// the player. Its room has no end until set.
#[derive(Clone)]
pub struct FakeSnipFolder {
    table: Arc<Mutex<Table>>,
    /// Signals each `remove`, for a test waiting on another thread.
    changed: Arc<Condvar>,
    clock: Arc<dyn Clock>,
}

impl FakeSnipFolder {
    /// The folder `snips` with its unsaved part `unsaved`, empty, aging its files by `clock`. Write each test path for
    /// Windows and wrap it in `kx_test_support::paths::native_path`, so it matches on every system (ADR-0062).
    #[must_use]
    pub fn new(clock: Arc<dyn Clock>, snips: FolderPath, unsaved: FolderPath) -> Self {
        Self::with(clock, Some(Dirs { snips, unsaved }))
    }

    /// No folder, as a hidden run makes none (ADR-0037).
    #[must_use]
    pub fn hidden(clock: Arc<dyn Clock>) -> Self {
        Self::with(clock, None)
    }

    /// A folder of `dirs`, empty.
    fn with(clock: Arc<dyn Clock>, dirs: Option<Dirs>) -> Self {
        let table = Table {
            dirs,
            files: BTreeMap::new(),
            free: Ok(u64::MAX),
            list_error: None,
            remove_error: None,
            held: None,
            removes: Vec::new(),
            marks: BTreeMap::new(),
            mark_error: None,
        };
        let table = Arc::new(Mutex::new(table));
        let changed = Arc::default();
        Self {
            table,
            changed,
            clock,
        }
    }

    /// Sets what `free` answers: bytes, or an error.
    pub fn set_free(&self, free: Result<u64, SaveError>) {
        lock(&self.table).free = free;
    }

    /// Refuses later `unsaved` lists with `error`, or takes them again with `None`.
    pub fn set_list_error(&self, error: Option<SaveError>) {
        lock(&self.table).list_error = error;
    }

    /// Refuses later `remove` calls with `error`, keeping the file, or takes them again with `None`.
    pub fn set_remove_error(&self, error: Option<SaveError>) {
        lock(&self.table).remove_error = error;
    }

    /// Refuses the next `times` removes with `error`, keeping the file, as one another program still holds; then takes
    /// them again.
    pub fn refuse_removes(&self, error: SaveError, times: usize) {
        lock(&self.table).held = Some((error, times));
    }

    /// Writes `stored` into `file` now, replacing what it held.
    pub fn put(&self, file: SnipFile, stored: Stored) {
        self.put_at(file, stored, self.clock.now());
    }

    /// Writes `stored` into `file` as last written at `at`, which may be in the future.
    pub fn put_at(&self, file: SnipFile, stored: Stored, at: Mono) {
        let path = file.expose().to_path_buf();
        let entry = Entry { file, stored, at };
        lock(&self.table).files.insert(path, entry);
    }

    /// What `file` holds, or `None` when there is no such file.
    #[must_use]
    pub fn stored(&self, file: &SnipFile) -> Option<Stored> {
        let table = lock(&self.table);
        table.files.get(file.expose()).map(|e| e.stored.clone())
    }

    /// Every file held, wherever it lies, by path.
    #[must_use]
    pub fn files(&self) -> Vec<SnipFile> {
        lock(&self.table)
            .files
            .values()
            .map(|e| e.file.clone())
            .collect()
    }

    /// Every file `remove` was asked, refused ones too, oldest first.
    #[must_use]
    pub fn removes(&self) -> Vec<SnipFile> {
        lock(&self.table).removes.clone()
    }

    /// Waits up to `within` until `count` removes were asked, so a test never sleeps; true when they were.
    #[must_use]
    pub fn wait_removes(&self, count: usize, within: Duration) -> bool {
        wait_for(&self.table, &self.changed, within, |t| {
            t.removes.len() >= count
        })
    }

    /// The length of `video`, a video held directly in the unsaved part.
    ///
    /// # Errors
    /// `NoFolder` in a hidden run, `BadPath` for a picture or a file elsewhere, `Missing` when it is gone.
    pub(super) fn unsaved_video(&self, video: &SnipFile) -> Result<u32, SaveError> {
        let table = lock(&self.table);
        let dirs = table.dirs.as_ref().ok_or(SaveError::NoFolder)?;
        if video.kind() != FileKind::Mp4 || !video.lies_in(&dirs.unsaved) {
            return Err(SaveError::BadPath);
        }
        match table.files.get(video.expose()).map(|e| &e.stored) {
            Some(Stored::Video { length_ms }) => Ok(*length_ms),
            Some(Stored::Png(_)) => Err(SaveError::BadPath),
            None => Err(SaveError::Missing),
        }
    }

    /// The time by the folder's clock.
    pub(super) fn now(&self) -> Mono {
        self.clock.now()
    }

    /// The unsaved part; `None` in a hidden run.
    pub(super) fn unsaved_dir(&self) -> Option<FolderPath> {
        lock(&self.table).dirs.as_ref().map(|d| d.unsaved.clone())
    }

    /// Deletes `file` as a fake's own work, which `removes` does not list; one already gone is no matter.
    pub(super) fn forget(&self, file: &SnipFile) {
        lock(&self.table).files.remove(file.expose());
    }

    /// Notes the ask to remove `file`, then deletes it as `SnipFolder::remove` says.
    fn delete(&self, file: &SnipFile) -> Result<(), SaveError> {
        let mut table = lock(&self.table);
        table.removes.push(file.clone());
        let dirs = table.dirs.as_ref().ok_or(SaveError::NoFolder)?;
        if !file.lies_in(&dirs.unsaved) {
            return Err(SaveError::BadPath);
        }
        table.remove_error.map_or(Ok(()), Err)?;
        table.marks.remove(file.expose());
        if let Some((error, left)) = table.held.as_mut().filter(|(_, left)| *left > 0) {
            *left -= 1;
            return Err(*error);
        }
        table.files.remove(file.expose());
        Ok(())
    }

    /// True when `file` lies directly in the Snips folder or its unsaved part, by the check that reads alike
    /// everywhere.
    pub(super) fn in_snips(&self, file: &SnipFile) -> bool {
        let table = lock(&self.table);
        let dirs = table.dirs.as_ref();
        dirs.is_some_and(|d| file.lies_in(&d.snips) || file.lies_in(&d.unsaved))
    }
}

/// Shows the counts, never a path or a picture.
impl fmt::Debug for FakeSnipFolder {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let table = lock(&self.table);
        f.debug_struct("FakeSnipFolder")
            .field("hidden", &table.dirs.is_none())
            .field("files", &table.files.len())
            .field("removes", &table.removes.len())
            .field("marks", &table.marks.len())
            .finish_non_exhaustive()
    }
}

impl SnipFolder for FakeSnipFolder {
    fn snips(&self) -> Option<FolderPath> {
        lock(&self.table).dirs.as_ref().map(|d| d.snips.clone())
    }

    fn unsaved(&self) -> Result<Vec<UnsavedFile>, SaveError> {
        let now = self.clock.now();
        let table = lock(&self.table);
        let dirs = table.dirs.as_ref().ok_or(SaveError::NoFolder)?;
        table.list_error.map_or(Ok(()), Err)?;
        let there = table
            .files
            .values()
            .filter(|e| e.file.lies_in(&dirs.unsaved));
        let aged = there.map(|e| UnsavedFile {
            file: e.file.clone(),
            age: now.saturating_since(e.at),
            copied: table
                .marks
                .get(e.file.expose())
                .map(|(_, at)| now.saturating_since(*at)),
        });
        Ok(aged.collect())
    }

    fn mark_copied(&self, video: &SnipFile) -> Result<(), SaveError> {
        self.mark(video)
    }

    fn prune_marks(&self) -> Result<(), SaveError> {
        self.prune()
    }

    fn remove(&self, file: &SnipFile) -> Result<(), SaveError> {
        let removed = self.delete(file);
        self.changed.notify_all();
        removed
    }

    fn free(&self) -> Result<u64, SaveError> {
        let table = lock(&self.table);
        table.dirs.as_ref().ok_or(SaveError::NoFolder)?;
        table.free
    }
}

mod marks;
#[cfg(test)]
mod tests;
