//! `SealedFiles`: the sealed store over one folder, one `<name>.sealed` file per blob, under one data key.
//! A blob that will not open is renamed to `<name>.damaged`, so `load` reports it once.

use crate::consts::{DAMAGED_EXT, KEY_FILE, MAX_BLOB_BYTES, OVERHEAD, RESERVED_NAMES, SEALED_EXT};
use crate::guard::KeyGuard;
use crate::seal::{open, seal};
use disk::Read;
use key::DataKey;
use kx_module_api::ports::{BlobName, SealedError, SealedStore};
use std::fmt;
use std::fs;
use std::path::PathBuf;
use std::sync::{Mutex, MutexGuard, PoisonError};

mod disk;
mod key;

/// The sealed store over the folder `dir`; the guard wraps its data key.
pub struct SealedFiles {
    dir: PathBuf,
    guard: Box<dyn KeyGuard>,
    /// The data key once unwrapped or made; the lock also keeps one file step at a time.
    key: Mutex<Option<DataKey>>,
}

impl SealedFiles {
    /// A store over `dir`, which the first save creates.
    #[must_use]
    pub fn new(dir: PathBuf, guard: Box<dyn KeyGuard>) -> Self {
        Self {
            dir,
            guard,
            key: Mutex::new(None),
        }
    }

    /// The file of blob `name` with extension `ext`.
    fn file(&self, name: &str, ext: &str) -> PathBuf {
        self.dir.join(format!("{name}.{ext}"))
    }

    /// The held key, else the key file's; `None` when there is no key file.
    fn key_to_open<'a>(
        &self,
        held: &'a mut Option<DataKey>,
    ) -> Result<Option<&'a DataKey>, SealedError> {
        let key = match held.take() {
            Some(key) => Some(key),
            None => key::read(&self.dir.join(KEY_FILE), &*self.guard)?,
        };
        Ok(key.map(|key| &*held.insert(key)))
    }

    /// The key a save seals with; a lost key file gets a new key, so every blob matches the file.
    fn key_to_seal<'a>(&self, held: &'a mut Option<DataKey>) -> Result<&'a DataKey, SealedError> {
        let path = self.dir.join(KEY_FILE);
        if !key::present(&path)? {
            *held = None;
        }
        let key = match held.take() {
            Some(key) => key,
            None => match key::read(&path, &*self.guard)? {
                Some(key) => key,
                None => key::make(&path, &*self.guard)?,
            },
        };
        Ok(held.insert(key))
    }

    /// Renames blob `name`'s file to its damaged file, replacing an older one.
    fn set_aside(&self, name: &str) -> Result<Option<Vec<u8>>, SealedError> {
        let from = self.file(name, SEALED_EXT);
        fs::rename(from, self.file(name, DAMAGED_EXT)).map_err(|_| SealedError::Io)?;
        Err(SealedError::Damaged)
    }
}

/// `name` as text, unless Windows keeps it for a device.
fn checked(name: &BlobName) -> Result<&str, SealedError> {
    let text = name.as_str();
    if RESERVED_NAMES.contains(&text) {
        return Err(SealedError::BadName);
    }
    Ok(text)
}

/// Locks `m`; a panic elsewhere leaves the key valid, so a poisoned lock is still used.
fn lock<T>(m: &Mutex<T>) -> MutexGuard<'_, T> {
    m.lock().unwrap_or_else(PoisonError::into_inner)
}

impl SealedStore for SealedFiles {
    fn load(&self, name: &BlobName) -> Result<Option<Vec<u8>>, SealedError> {
        let name = checked(name)?;
        let mut held = lock(&self.key);
        let path = self.file(name, SEALED_EXT);
        let sealed = match disk::read(&path, MAX_BLOB_BYTES + OVERHEAD) {
            Ok(Read::Missing) => return Ok(None),
            Ok(Read::TooBig) => return self.set_aside(name),
            Ok(Read::Bytes(bytes)) => bytes,
            Err(_) => return Err(SealedError::Io),
        };
        let Some(key) = self.key_to_open(&mut held)? else {
            return self.set_aside(name);
        };
        match open(key, name.as_bytes(), &sealed) {
            Ok(mut plain) => Ok(Some(std::mem::take(&mut *plain))),
            Err(_) => self.set_aside(name),
        }
    }

    fn save(&self, name: &BlobName, bytes: &[u8]) -> Result<(), SealedError> {
        let name = checked(name)?;
        if bytes.len() > MAX_BLOB_BYTES {
            return Err(SealedError::Io);
        }
        let mut held = lock(&self.key);
        fs::create_dir_all(&self.dir).map_err(|_| SealedError::Io)?;
        let key = self.key_to_seal(&mut held)?;
        let sealed = seal(key, name.as_bytes(), bytes).map_err(|_| SealedError::Io)?;
        disk::write_whole(&self.file(name, SEALED_EXT), &sealed).map_err(|_| SealedError::Io)
    }
}

impl fmt::Debug for SealedFiles {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // The folder's path holds the account name, and the key is secret, so neither is shown.
        f.debug_struct("SealedFiles").finish_non_exhaustive()
    }
}

#[cfg(test)]
mod tests;
