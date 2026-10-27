//! A fake sealed store: blobs kept in memory by name up to the port's `MAX_BLOB_BYTES`, one failure on cue each way,
//! and counted calls.

use crate::lock;
use kx_module_api::ports::{BlobName, MAX_BLOB_BYTES, SealedError, SealedStore};
use std::collections::BTreeMap;
use std::fmt;
use std::sync::{Arc, Mutex};

/// What the calls did, and what the test set for them.
#[derive(Default)]
struct Log {
    blobs: BTreeMap<BlobName, Vec<u8>>,
    load_fail: Option<SealedError>,
    save_fail: Option<SealedError>,
    loads: usize,
    saves: usize,
}

/// A sealed store shared by its clones, which never seals: it keeps each blob's bytes as given.
///
/// A scripted `Damaged` load sets the blob aside, as the real adapter does, so the next load finds nothing.
#[derive(Clone, Default)]
pub struct FakeSealedStore {
    log: Arc<Mutex<Log>>,
}

impl FakeSealedStore {
    /// A store with no blob.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Refuses the next load with `error`; a debug build stops on one `SealedStore::load` never returns.
    pub fn fail_load(&self, error: SealedError) {
        debug_assert!(
            error != SealedError::BadName,
            "the port never returns {error:?} from a load"
        );
        lock(&self.log).load_fail = Some(error);
    }

    /// Refuses the next save with `error`, keeping the old blob; a debug build stops on one `SealedStore::save` never
    /// returns.
    pub fn fail_save(&self, error: SealedError) {
        let ported = matches!(error, SealedError::Locked | SealedError::Io);
        debug_assert!(ported, "the port never returns {error:?} from a save");
        lock(&self.log).save_fail = Some(error);
    }

    /// The bytes kept under `name`, read without a load.
    #[must_use]
    pub fn blob(&self, name: &BlobName) -> Option<Vec<u8>> {
        lock(&self.log).blobs.get(name).cloned()
    }

    /// Keeps `bytes` under `name` without a save, as an earlier run would have left them.
    pub fn plant(&self, name: &BlobName, bytes: Vec<u8>) {
        lock(&self.log).blobs.insert(name.clone(), bytes);
    }

    /// How many loads were asked, refused ones too.
    #[must_use]
    pub fn loads(&self) -> usize {
        lock(&self.log).loads
    }

    /// How many saves were asked, refused ones too.
    #[must_use]
    pub fn saves(&self) -> usize {
        lock(&self.log).saves
    }
}

impl fmt::Debug for FakeSealedStore {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let log = lock(&self.log);
        let names: Vec<_> = log.blobs.keys().map(BlobName::as_str).collect();
        f.debug_struct("FakeSealedStore")
            .field("blobs", &names)
            .field("loads", &log.loads)
            .field("saves", &log.saves)
            .finish_non_exhaustive()
    }
}

impl SealedStore for FakeSealedStore {
    fn load(&self, name: &BlobName) -> Result<Option<Vec<u8>>, SealedError> {
        let mut log = lock(&self.log);
        log.loads += 1;
        if let Some(error) = log.load_fail.take() {
            if error == SealedError::Damaged {
                log.blobs.remove(name);
            }
            return Err(error);
        }
        Ok(log.blobs.get(name).cloned())
    }

    fn save(&self, name: &BlobName, bytes: &[u8]) -> Result<(), SealedError> {
        let mut log = lock(&self.log);
        log.saves += 1;
        if let Some(error) = log.save_fail.take() {
            return Err(error);
        }
        if bytes.len() > MAX_BLOB_BYTES {
            return Err(SealedError::Io);
        }
        log.blobs.insert(name.clone(), bytes.to_vec());
        Ok(())
    }
}

#[cfg(test)]
mod tests;
