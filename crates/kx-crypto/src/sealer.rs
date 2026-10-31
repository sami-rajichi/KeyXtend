//! The sealed file steps both stores share: one data key in a key file the guard wraps, and one file step at a
//! time. A file that will not open is renamed to its damaged name, so a load reports it once.

use crate::consts::{DAMAGED_EXT, MAX_BLOB_BYTES, OVERHEAD, SEALED_EXT};
use crate::guard::KeyGuard;
use crate::seal::{open, seal};
use disk::Read;
use key::DataKey;
use kx_module_api::ports::{BlobName, SealedError};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, MutexGuard, PoisonError};

mod disk;
mod key;

/// Seals and opens files under the data key that the guard wraps in `key_file`.
pub(crate) struct Sealer {
    key_file: PathBuf,
    guard: Box<dyn KeyGuard>,
    /// The data key once unwrapped or made; the lock also keeps one file step at a time.
    key: Mutex<Option<DataKey>>,
}

/// The sealed file of blob `name` in `dir`.
pub(crate) fn blob_file(dir: &Path, name: &BlobName) -> PathBuf {
    dir.join(format!("{}.{SEALED_EXT}", name.as_str()))
}

/// The name a file that will not open is set aside under.
fn damaged_of(path: &Path) -> PathBuf {
    path.with_extension(DAMAGED_EXT)
}

/// Makes the folder that holds `path`.
fn make_parent(path: &Path) -> Result<(), SealedError> {
    path.parent()
        .map_or(Ok(()), fs::create_dir_all)
        .map_err(|_| SealedError::Io)
}

/// Renames the file at `path` to its damaged name, replacing an older one; `Damaged`, or `Io` if it stays.
fn set_aside(path: &Path) -> SealedError {
    match fs::rename(path, damaged_of(path)) {
        Ok(()) => SealedError::Damaged,
        Err(_) => SealedError::Io,
    }
}

/// Locks `m`; a panic elsewhere leaves the key valid, so a poisoned lock is still used.
fn lock<T>(m: &Mutex<T>) -> MutexGuard<'_, T> {
    m.lock().unwrap_or_else(PoisonError::into_inner)
}

impl Sealer {
    /// A sealer whose key lives in `key_file`, which the first save writes.
    pub(crate) fn new(key_file: PathBuf, guard: Box<dyn KeyGuard>) -> Self {
        Self {
            key_file,
            guard,
            key: Mutex::new(None),
        }
    }

    /// The held key, else the key file's; `None` when there is no key file.
    fn key_to_open<'a>(
        &self,
        held: &'a mut Option<DataKey>,
    ) -> Result<Option<&'a DataKey>, SealedError> {
        let key = match held.take() {
            Some(key) => Some(key),
            None => key::read(&self.key_file, &*self.guard)?,
        };
        Ok(key.map(|key| &*held.insert(key)))
    }

    /// The key a save seals with; a lost key file gets a new key, so every file matches the key file.
    fn key_to_seal<'a>(&self, held: &'a mut Option<DataKey>) -> Result<&'a DataKey, SealedError> {
        if !key::present(&self.key_file)? {
            *held = None;
        }
        let key = match held.take() {
            Some(key) => key,
            None => match key::read(&self.key_file, &*self.guard)? {
                Some(key) => key,
                None => key::make(&self.key_file, &*self.guard)?,
            },
        };
        Ok(held.insert(key))
    }

    /// The opened bytes of the sealed file at `path`, bound to `aad`, or `None` when there is no file.
    pub(crate) fn load(&self, path: &Path, aad: &[u8]) -> Result<Option<Vec<u8>>, SealedError> {
        let mut held = lock(&self.key);
        let sealed = match disk::read(path, MAX_BLOB_BYTES + OVERHEAD) {
            Ok(Read::Missing) => return Ok(None),
            Ok(Read::TooBig) => return Err(set_aside(path)),
            Ok(Read::Bytes(bytes)) => bytes,
            Err(_) => return Err(SealedError::Io),
        };
        let Some(key) = self.key_to_open(&mut held)? else {
            return Err(set_aside(path));
        };
        match open(key, aad, &sealed) {
            Ok(mut plain) => Ok(Some(std::mem::take(&mut *plain))),
            Err(_) => Err(set_aside(path)),
        }
    }

    /// Seals `bytes` bound to `aad` into the file at `path` whole, making its folder once the key is ready.
    pub(crate) fn save(&self, path: &Path, aad: &[u8], bytes: &[u8]) -> Result<(), SealedError> {
        if bytes.len() > MAX_BLOB_BYTES {
            return Err(SealedError::Io);
        }
        let mut held = lock(&self.key);
        make_parent(&self.key_file)?;
        let key = self.key_to_seal(&mut held)?;
        let sealed = seal(key, aad, bytes).map_err(|_| SealedError::Io)?;
        make_parent(path)?;
        disk::write_whole(path, &sealed).map_err(|_| SealedError::Io)
    }

    /// Deletes the sealed file at `path`, its damaged copy and a temp file left beside it; missing ones are fine.
    pub(crate) fn remove(&self, path: &Path) -> Result<(), SealedError> {
        let _held = lock(&self.key);
        for file in [path.to_owned(), damaged_of(path), disk::temp_of(path)] {
            disk::remove(&file).map_err(|_| SealedError::Io)?;
        }
        Ok(())
    }

    /// Runs `step` while no other file step runs.
    pub(crate) fn alone<T>(&self, step: impl FnOnce() -> T) -> T {
        let _held = lock(&self.key);
        step()
    }
}
