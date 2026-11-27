//! Fake Save and folder dialogs: each answers the next pick the test queued, Cancel once none is left, and a chosen
//! file takes writes into the Snips folder's table in memory, or refuses them as a full or read-only place would.

use super::folder::{FakeSnipFolder, Stored};
use crate::{lock, wait_for};
use kx_module_api::Redacted;
use kx_module_api::ports::{
    AskError, FileAsk, FileKind, FolderPath, PickLimits, SaveError, SaveTo, SnipFile,
};
use std::collections::VecDeque;
use std::fmt;
use std::sync::{Arc, Condvar, Mutex};
use std::time::Duration;

/// One answer of the fake Save dialog.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Pick {
    /// The user chose this file, whose place takes writes.
    File(SnipFile),
    /// The user chose this file, but its place refuses every write with this error, as a full or read-only drive
    /// (edge 6).
    Stuck(SnipFile, SaveError),
    /// The user pressed Cancel.
    Cancel,
    /// No dialog showed.
    Fail(AskError),
}

/// The answers queued, where each dialog opened, and the writes tried.
#[derive(Default)]
struct Asks {
    picks: VecDeque<Pick>,
    folders: VecDeque<Result<Option<FolderPath>, AskError>>,
    /// The file each Save dialog proposed, oldest first.
    proposed: Vec<SnipFile>,
    /// The folder each folder dialog opened at, oldest first.
    opened: Vec<FolderPath>,
    /// The limits each dialog of either kind was held to, oldest first.
    limits: Vec<PickLimits>,
    /// Writes through an answer, refused ones too.
    writes: usize,
}

/// The asks, and the signal each call gives once done, for a test waiting on another thread.
#[derive(Default)]
struct Shared {
    asks: Mutex<Asks>,
    changed: Condvar,
}

impl Shared {
    /// Does `work` on the asks, then wakes a waiting test.
    fn note<T>(&self, work: impl FnOnce(&mut Asks) -> T) -> T {
        let done = work(&mut lock(&self.asks));
        self.changed.notify_all();
        done
    }

    /// Waits up to `within` until `count` of what `of` counts were done; true when they were.
    fn wait(&self, count: usize, within: Duration, of: impl Fn(&Asks) -> usize) -> bool {
        wait_for(&self.asks, &self.changed, within, |a| of(a) >= count)
    }
}

/// Dialogs shared by their clones, whose chosen files write into `folder`'s table.
#[derive(Clone)]
pub struct FakeFileAsk {
    shared: Arc<Shared>,
    folder: FakeSnipFolder,
}

impl FakeFileAsk {
    /// Dialogs with nothing queued, writing into `folder`'s table.
    #[must_use]
    pub fn new(folder: &FakeSnipFolder) -> Self {
        let shared = Arc::default();
        let folder = folder.clone();
        Self { shared, folder }
    }

    /// Queues `pick` for a later Save dialog, after those already queued.
    pub fn queue_save(&self, pick: Pick) {
        lock(&self.shared.asks).picks.push_back(pick);
    }

    /// Queues `answer` for a later folder dialog: a folder, `None` for Cancel, or an error.
    pub fn queue_folder(&self, answer: Result<Option<FolderPath>, AskError>) {
        lock(&self.shared.asks).folders.push_back(answer);
    }

    /// The file each Save dialog proposed, refused ones too, oldest first.
    #[must_use]
    pub fn saves(&self) -> Vec<SnipFile> {
        lock(&self.shared.asks).proposed.clone()
    }

    /// The folder each folder dialog opened at, oldest first.
    #[must_use]
    pub fn folders(&self) -> Vec<FolderPath> {
        lock(&self.shared.asks).opened.clone()
    }

    /// The limits each dialog of either kind was held to, oldest first; limits `check` refuses show none.
    #[must_use]
    pub fn limits(&self) -> Vec<PickLimits> {
        lock(&self.shared.asks).limits.clone()
    }

    /// Waits up to `within` until `count` Save dialogs were answered, so a test never sleeps; true when they were.
    #[must_use]
    pub fn wait_saves(&self, count: usize, within: Duration) -> bool {
        self.shared.wait(count, within, |a| a.proposed.len())
    }

    /// Waits up to `within` until `count` folder dialogs were answered; true when they were.
    #[must_use]
    pub fn wait_folders(&self, count: usize, within: Duration) -> bool {
        self.shared.wait(count, within, |a| a.opened.len())
    }

    /// Waits up to `within` until `count` writes through an answer were done, refused ones too; true when they were.
    #[must_use]
    pub fn wait_writes(&self, count: usize, within: Duration) -> bool {
        self.shared.wait(count, within, |a| a.writes)
    }
}

/// Shows the counts, never a path.
impl fmt::Debug for FakeFileAsk {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let asks = lock(&self.shared.asks);
        f.debug_struct("FakeFileAsk")
            .field("saves", &asks.proposed.len())
            .field("folders", &asks.opened.len())
            .field("writes", &asks.writes)
            .field("queued", &asks.picks.len())
            .finish_non_exhaustive()
    }
}

impl FileAsk for FakeFileAsk {
    fn save(
        &self,
        proposed: &SnipFile,
        limits: &PickLimits,
    ) -> Result<Option<Box<dyn SaveTo>>, AskError> {
        limits.check()?;
        let pick = self.shared.note(|asks| {
            asks.proposed.push(proposed.clone());
            asks.limits.push(*limits);
            asks.picks.pop_front().unwrap_or(Pick::Cancel)
        });
        let (file, stuck) = match pick {
            Pick::Cancel => return Ok(None),
            Pick::Fail(error) => return Err(error),
            Pick::File(file) => (file, None),
            Pick::Stuck(file, error) => (file, Some(error)),
        };
        if file.kind() != proposed.kind() {
            return Err(AskError::BadPath);
        }
        let (shared, folder) = (Arc::clone(&self.shared), self.folder.clone());
        let answer = FakeSaveTo {
            file,
            stuck,
            folder,
            shared,
        };
        Ok(Some(Box::new(answer)))
    }

    fn folder(&self, at: &FolderPath, limits: &PickLimits) -> Result<Option<FolderPath>, AskError> {
        limits.check()?;
        self.shared.note(|asks| {
            asks.opened.push(at.clone());
            asks.limits.push(*limits);
            asks.folders.pop_front().unwrap_or(Ok(None))
        })
    }
}

/// A chosen file; it writes into the folder's table unless its place is stuck.
struct FakeSaveTo {
    file: SnipFile,
    /// The error every write gives, as a full or read-only place.
    stuck: Option<SaveError>,
    folder: FakeSnipFolder,
    shared: Arc<Shared>,
}

impl FakeSaveTo {
    /// Writes `stored` there unless the answer is of another kind than `kind` or its place is stuck, then counts it.
    fn write(
        &self,
        kind: FileKind,
        stored: impl FnOnce() -> Result<Stored, SaveError>,
    ) -> Result<(), SaveError> {
        let wrote = self.put(kind, stored);
        self.shared.note(|asks| asks.writes += 1);
        wrote
    }

    /// Writes what `stored` gives there, as `write` says.
    fn put(
        &self,
        kind: FileKind,
        stored: impl FnOnce() -> Result<Stored, SaveError>,
    ) -> Result<(), SaveError> {
        let same = self.file.kind() == kind;
        same.then_some(()).ok_or(SaveError::BadPath)?;
        let stored = stored()?;
        self.stuck.map_or(Ok(()), Err)?;
        self.folder.put(self.file.clone(), stored);
        Ok(())
    }
}

impl SaveTo for FakeSaveTo {
    fn file(&self) -> &SnipFile {
        &self.file
    }

    fn png(&self, png: &Redacted<Vec<u8>>) -> Result<(), SaveError> {
        self.write(FileKind::Png, || Ok(Stored::Png(png.clone())))
    }

    fn copy(&self, video: &SnipFile) -> Result<(), SaveError> {
        let held = || self.folder.unsaved_video(video);
        self.write(FileKind::Mp4, || {
            held().map(|length_ms| Stored::Video { length_ms })
        })
    }
}

#[cfg(test)]
mod tests;
