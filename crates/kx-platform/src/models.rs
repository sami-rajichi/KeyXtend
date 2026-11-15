//! `ModelFiles` (ADR-0043): the fixed drives and their room, the models folder made with rights its owner alone holds,
//! a folder's scan, a file's fingerprint, and one `fetch` job per download after a check of the drive's room. No path
//! or name prints.

mod fetch;
mod folder;

use crate::worker::{FetchFrames, Spawner, Split};
use fetch::WorkerFetch;
use kx_ipc::SHA256_BYTES;
use kx_module_api::Clock;
use kx_module_api::ports::{
    Drive, FetchAsk, FetchNote, FetchRun, Fingerprint, FolderScan, ModelError, ModelFiles, path_ok,
};
use std::io;
use std::path::Path;
use std::sync::Arc;

/// What the system's file system tells and does for the models host beside the standard library.
pub trait Disk: Send + Sync {
    /// The fixed drives, with their room.
    fn drives(&self) -> Vec<Drive>;

    /// Bytes free on the drive that holds `folder`.
    ///
    /// # Errors
    /// The system's error, `NotFound` for a folder that is gone.
    fn free(&self, folder: &Path) -> io::Result<u64>;

    /// The SHA-256 of `file`, read whole, never through a link, and only while its size stays.
    ///
    /// # Errors
    /// The system's error, or one for a link, a folder or a size change.
    fn sha256(&self, file: &Path) -> io::Result<[u8; SHA256_BYTES]>;

    /// True for a hidden or system entry, which a scan skips.
    fn hidden(&self, path: &Path) -> bool;

    /// Makes `folder`, whose parent exists, with rights its owner and the system alone hold, never ones its drive
    /// passes down.
    ///
    /// # Errors
    /// The system's error, `AlreadyExists` when it is there.
    fn make_guarded(&self, folder: &Path) -> io::Result<()>;
}

/// Model files on the system's `Disk`, each download in a job the worker `S` starts.
pub struct WorkerModels<S: Spawner<FetchFrames, Worker: Split<FetchFrames>>> {
    spawner: Arc<S>,
    clock: Arc<dyn Clock>,
    disk: Arc<dyn Disk>,
}

impl<S: Spawner<FetchFrames, Worker: Split<FetchFrames>>> WorkerModels<S> {
    /// Model files over `disk`, whose download jobs `spawner` starts and read the time from `clock`.
    #[must_use]
    pub fn new(spawner: S, clock: Arc<dyn Clock>, disk: Arc<dyn Disk>) -> Self {
        Self {
            spawner: Arc::new(spawner),
            clock,
            disk,
        }
    }
}

impl<S: Spawner<FetchFrames, Worker: Split<FetchFrames>> + 'static> ModelFiles for WorkerModels<S> {
    fn drives(&self) -> Vec<Drive> {
        self.disk.drives()
    }

    fn make(&self, folder: &Path) -> Result<(), ModelError> {
        plain(folder)?;
        folder::make(self.disk.as_ref(), folder)
    }

    fn scan(&self, folder: &Path, exts: &[String]) -> Result<FolderScan, ModelError> {
        plain(folder)?;
        folder::scan(self.disk.as_ref(), folder, exts)
    }

    fn fingerprint(&self, file: &Path) -> Result<Fingerprint, ModelError> {
        plain(file)?;
        let bytes = self.disk.sha256(file).map_err(|e| read_fault(&e))?;
        Ok(Fingerprint::new(bytes))
    }

    fn fetch(
        &self,
        ask: FetchAsk,
        tell: Box<dyn Fn(FetchNote) + Send>,
    ) -> Result<Box<dyn FetchRun>, ModelError> {
        ask.check()?;
        let (spawner, clock) = (Arc::clone(&self.spawner), Arc::clone(&self.clock));
        let run = WorkerFetch::start(spawner, clock, self.disk.as_ref(), &ask, tell)?;
        Ok(Box::new(run))
    }

    fn remove(&self, file: &Path) -> Result<(), ModelError> {
        plain(file)?;
        folder::remove(file)
    }
}

/// Refuses a path that is no plain drive path (`path_ok`).
fn plain(path: &Path) -> Result<(), ModelError> {
    if path_ok(path) {
        Ok(())
    } else {
        Err(ModelError::BadPath)
    }
}

/// The error a write fails with: a missing folder or drive, one that cannot be written, a full drive, or another.
fn fault(e: &io::Error) -> ModelError {
    match e.kind() {
        io::ErrorKind::NotFound => ModelError::Missing,
        io::ErrorKind::PermissionDenied | io::ErrorKind::ReadOnlyFilesystem => ModelError::ReadOnly,
        io::ErrorKind::StorageFull => ModelError::Full,
        _ => ModelError::Io,
    }
}

/// The error a read fails with: a missing file or folder, or another.
fn read_fault(e: &io::Error) -> ModelError {
    match fault(e) {
        ModelError::Missing => ModelError::Missing,
        _ => ModelError::Io,
    }
}

#[cfg(test)]
mod tests;
