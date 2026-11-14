//! A fake model files service: drives, folders and files the test sets in a small tree, downloads it moves on cue,
//! and failures once per call; every path passes the port's own rule first, and each call is counted.

use crate::lock;
use kx_module_api::ports::{
    Drive, FetchAsk, FetchRun, Fingerprint, FolderScan, ModelError, ModelFiles, path_ok,
};
use std::collections::HashMap;
use std::fmt;
use std::path::Path;
use std::sync::{Arc, Condvar, Mutex, PoisonError};
use std::time::Duration;

mod fetch;
mod tree;

pub use fetch::FetchLog;
use fetch::{FakeFetchRun, Hearer, Tell};
use tree::Tree;

/// One kind of call, to fail once or count.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum FileCall {
    /// `make`.
    Make,
    /// `scan`.
    Scan,
    /// `fingerprint`.
    Fingerprint,
    /// `fetch`.
    Fetch,
    /// `remove`.
    Remove,
}

impl FileCall {
    /// Whether the port lets this call return `error`, besides the refusals its path or ask gives.
    fn returns(self, error: ModelError) -> bool {
        use ModelError::{Full, Gone, Io, Missing, ReadOnly};
        match self {
            Self::Make => matches!(error, Missing | ReadOnly | Io),
            Self::Scan | Self::Fingerprint => matches!(error, Missing | Io),
            Self::Fetch => matches!(error, Full | Missing | ReadOnly | Gone),
            Self::Remove => matches!(error, ReadOnly | Io),
        }
    }
}

/// What the calls did, and what the test set for them.
#[derive(Default)]
struct Log {
    drives: Vec<Drive>,
    tree: Tree,
    fails: HashMap<FileCall, ModelError>,
    calls: HashMap<FileCall, usize>,
    fetches: Vec<FetchLog>,
    /// Each download's callback, by its place in `fetches`, until its run drops.
    hearers: Vec<Option<Hearer>>,
}

impl Log {
    /// Counts `call`, then refuses it for a path the port's rule refuses, else with the failure set for it.
    fn enter(&mut self, call: FileCall, path: &Path) -> Result<(), ModelError> {
        *self.calls.entry(call).or_default() += 1;
        if !path_ok(path) {
            return Err(ModelError::BadPath);
        }
        self.fails.remove(&call).map_or(Ok(()), Err)
    }
}

/// The log, and the signal each change gives, for a test waiting on another thread.
#[derive(Default)]
struct Shared {
    log: Mutex<Log>,
    changed: Condvar,
}

/// Model files shared by their clones: the tree starts empty, `set_drives` and `add_folder` give it roots, and a
/// folder or a download's file needs its drive or folder there, else `Missing`.
///
/// Notes are told on the thread that moves the download, and a `tell` must not call back into the fake.
#[derive(Clone, Default)]
pub struct FakeModelFiles {
    shared: Arc<Shared>,
}

impl FakeModelFiles {
    /// Model files with no drive, no folder and nothing scripted.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Makes `drives` answer `drives`, and adds each root to the tree.
    pub fn set_drives(&self, drives: Vec<Drive>) {
        let mut log = lock(&self.shared.log);
        for drive in &drives {
            log.tree.add_folder(drive.root.expose());
        }
        log.drives = drives;
    }

    /// Adds the folder at `folder` and every folder above it, as one already on the drive.
    pub fn add_folder(&self, folder: &Path) {
        lock(&self.shared.log).tree.add_folder(folder);
    }

    /// Adds a file of `bytes` whose fingerprint is `print` at `file`, with its folders, as one already there.
    pub fn add_file(&self, file: &Path, bytes: u64, print: Fingerprint) {
        lock(&self.shared.log).tree.add_file(file, bytes, print);
    }

    /// Removes the file or folder at `path`, with everything under it, as the user or a lost drive would.
    pub fn vanish(&self, path: &Path) {
        lock(&self.shared.log).tree.vanish(path);
    }

    /// Fails the next `call` with `error`, once; a debug build stops on an error the port never returns there.
    pub fn fail(&self, call: FileCall, error: ModelError) {
        let ported = call.returns(error);
        debug_assert!(ported, "the port never returns {error:?} from {call:?}");
        lock(&self.shared.log).fails.insert(call, error);
    }

    /// Whether `path` is a folder in the tree.
    #[must_use]
    pub fn has_folder(&self, path: &Path) -> bool {
        lock(&self.shared.log).tree.folder(path)
    }

    /// Whether `path` is a named file in the tree.
    #[must_use]
    pub fn has_file(&self, path: &Path) -> bool {
        lock(&self.shared.log).tree.print(path).is_some()
    }

    /// The bytes the part file beside `file` holds, if one is there.
    #[must_use]
    pub fn part(&self, file: &Path) -> Option<u64> {
        lock(&self.shared.log).tree.part(file)
    }

    /// How many `call`s were made, refused ones too.
    #[must_use]
    pub fn calls(&self, call: FileCall) -> usize {
        let log = lock(&self.shared.log);
        log.calls.get(&call).copied().unwrap_or_default()
    }

    /// Each download started, in order, with its ask.
    #[must_use]
    pub fn fetches(&self) -> Vec<FetchLog> {
        lock(&self.shared.log).fetches.clone()
    }

    /// Waits up to `within` until `until` holds for the downloads, so a test never sleeps; true when it does.
    #[must_use]
    pub fn wait(&self, within: Duration, until: impl Fn(&[FetchLog]) -> bool) -> bool {
        let log = lock(&self.shared.log);
        let waited = (self.shared.changed).wait_timeout_while(log, within, |l| !until(&l.fetches));
        let (log, _) = waited.unwrap_or_else(PoisonError::into_inner);
        until(&log.fetches)
    }
}

/// Shows the counts, never a path.
impl fmt::Debug for FakeModelFiles {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let log = lock(&self.shared.log);
        f.debug_struct("FakeModelFiles")
            .field("drives", &log.drives.len())
            .field("entries", &log.tree.len())
            .field("fetches", &log.fetches.len())
            .finish_non_exhaustive()
    }
}

impl ModelFiles for FakeModelFiles {
    fn drives(&self) -> Vec<Drive> {
        lock(&self.shared.log).drives.clone()
    }

    fn make(&self, folder: &Path) -> Result<(), ModelError> {
        let mut log = lock(&self.shared.log);
        log.enter(FileCall::Make, folder)?;
        if !log.tree.drive(folder) {
            return Err(ModelError::Missing);
        }
        log.tree.add_folder(folder);
        Ok(())
    }

    fn scan(&self, folder: &Path, exts: &[String]) -> Result<FolderScan, ModelError> {
        let mut log = lock(&self.shared.log);
        log.enter(FileCall::Scan, folder)?;
        log.tree.scan(folder, exts).ok_or(ModelError::Missing)
    }

    fn fingerprint(&self, file: &Path) -> Result<Fingerprint, ModelError> {
        let mut log = lock(&self.shared.log);
        log.enter(FileCall::Fingerprint, file)?;
        log.tree.print(file).ok_or(ModelError::Missing)
    }

    fn fetch(&self, ask: FetchAsk, tell: Tell) -> Result<Box<dyn FetchRun>, ModelError> {
        let mut log = lock(&self.shared.log);
        *log.calls.entry(FileCall::Fetch).or_default() += 1;
        ask.check()?;
        if let Some(error) = log.fails.remove(&FileCall::Fetch) {
            return Err(error);
        }
        if !log.tree.parent(ask.file.expose()) {
            return Err(ModelError::Missing);
        }
        let id = log.fetches.len();
        log.fetches.push(FetchLog::new(ask));
        log.hearers.push(Some(Arc::new(Mutex::new(tell))));
        drop(log);
        self.shared.changed.notify_all();
        let shared = Arc::clone(&self.shared);
        Ok(Box::new(FakeFetchRun { shared, id }))
    }

    fn remove(&self, file: &Path) -> Result<(), ModelError> {
        let mut log = lock(&self.shared.log);
        log.enter(FileCall::Remove, file)?;
        log.tree.remove(file);
        Ok(())
    }
}

#[cfg(test)]
mod tests;
