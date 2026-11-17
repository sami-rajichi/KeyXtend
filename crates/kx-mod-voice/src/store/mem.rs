//! A sealed shelf in memory for the tests: blobs by folder and name, saves that fail on cue, and loads told damaged.

use std::collections::{BTreeMap, BTreeSet};
use std::sync::{Mutex, MutexGuard, PoisonError};

use kx_module_api::ports::{BlobName, MAX_BLOB_BYTES, Room, SealedError, SealedShelf};

/// A folder and a blob name, as text.
type Key = (String, String);

/// The shelf's state.
#[derive(Default)]
struct Inner {
    blobs: BTreeMap<Key, Vec<u8>>,
    /// Saves left before each fails; `None` never fails.
    saves: Option<usize>,
    /// Every remove fails, as a crash after the writes would leave the shelf.
    stuck: bool,
    /// Blobs whose load is told damaged.
    damaged: BTreeSet<Key>,
}

/// The shelf.
#[derive(Default)]
pub(crate) struct Mem(Mutex<Inner>);

impl Mem {
    fn inner(&self) -> MutexGuard<'_, Inner> {
        self.0.lock().unwrap_or_else(PoisonError::into_inner)
    }

    /// True when it holds no blob.
    pub(crate) fn is_empty(&self) -> bool {
        self.inner().blobs.is_empty()
    }

    /// The names in `folder`, sorted.
    pub(crate) fn names_in(&self, folder: &str) -> Vec<String> {
        let inner = self.inner();
        let here = inner.blobs.keys().filter(|(f, _)| f == folder);
        here.map(|(_, n)| n.clone()).collect()
    }

    /// The size of each blob in `folder`, by name.
    pub(crate) fn sizes(&self, folder: &str) -> Vec<usize> {
        let inner = self.inner();
        let here = inner.blobs.iter().filter(|((f, _), _)| f == folder);
        here.map(|(_, b)| b.len()).collect()
    }

    /// Every blob, to compare before and after.
    pub(crate) fn all(&self) -> BTreeMap<Key, Vec<u8>> {
        self.inner().blobs.clone()
    }

    /// Puts `bytes` as `name` in `folder` without a check.
    pub(crate) fn plant(&self, folder: &str, name: &str, bytes: &[u8]) {
        let key = (folder.to_owned(), name.to_owned());
        self.inner().blobs.insert(key, bytes.to_vec());
    }

    /// Removes `name` from `folder`.
    pub(crate) fn lose(&self, folder: &str, name: &str) {
        self.inner()
            .blobs
            .remove(&(folder.to_owned(), name.to_owned()));
    }

    /// Lets `n` more saves through, then fails each; `None` never fails.
    pub(crate) fn saves(&self, n: Option<usize>) {
        self.inner().saves = n;
    }

    /// Makes every remove fail, or work again.
    pub(crate) fn stuck(&self, on: bool) {
        self.inner().stuck = on;
    }

    /// Tells the load of `name` in `folder` damaged, as an adapter does when a blob will not open.
    pub(crate) fn damage(&self, folder: &str, name: &str) {
        let key = (folder.to_owned(), name.to_owned());
        self.inner().damaged.insert(key);
    }
}

/// The key of `name` in `folder`.
fn key(folder: &BlobName, name: &BlobName) -> Key {
    (folder.as_str().to_owned(), name.as_str().to_owned())
}

impl SealedShelf for Mem {
    fn load(&self, folder: &BlobName, name: &BlobName) -> Result<Option<Vec<u8>>, SealedError> {
        let inner = self.inner();
        let key = key(folder, name);
        if inner.damaged.contains(&key) {
            return Err(SealedError::Damaged);
        }
        Ok(inner.blobs.get(&key).cloned())
    }

    fn save(&self, folder: &BlobName, name: &BlobName, bytes: &[u8]) -> Result<(), SealedError> {
        let mut inner = self.inner();
        match inner.saves {
            Some(0) => return Err(SealedError::Io),
            Some(n) => inner.saves = Some(n - 1),
            None => {}
        }
        if bytes.len() > MAX_BLOB_BYTES {
            return Err(SealedError::Io);
        }
        inner.blobs.insert(key(folder, name), bytes.to_vec());
        Ok(())
    }

    fn remove(&self, folder: &BlobName, name: &BlobName) -> Result<(), SealedError> {
        let mut inner = self.inner();
        if inner.stuck {
            return Err(SealedError::Io);
        }
        inner.blobs.remove(&key(folder, name));
        Ok(())
    }

    fn drop_folder(&self, folder: &BlobName) -> Result<(), SealedError> {
        self.inner().blobs.retain(|(f, _), _| f != folder.as_str());
        Ok(())
    }

    fn folders(&self) -> Result<Vec<BlobName>, SealedError> {
        let inner = self.inner();
        let names: BTreeSet<&String> = inner.blobs.keys().map(|(f, _)| f).collect();
        Ok(names
            .into_iter()
            .filter_map(|f| BlobName::new(f).ok())
            .collect())
    }

    fn names(&self, folder: &BlobName) -> Result<Vec<BlobName>, SealedError> {
        let names = self.names_in(folder.as_str());
        Ok(names.iter().filter_map(|n| BlobName::new(n).ok()).collect())
    }

    fn room(&self) -> Result<Room, SealedError> {
        let used = self.inner().blobs.values().map(Vec::len).sum::<usize>();
        let used = u64::try_from(used).unwrap_or(u64::MAX);
        Ok(Room {
            used,
            free: u64::MAX,
            drive: "D:".into(),
        })
    }
}
