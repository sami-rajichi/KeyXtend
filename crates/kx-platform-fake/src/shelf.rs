//! A fake sealed shelf: folders of blobs kept in memory up to the port's `MAX_BLOB_BYTES`, one failure on cue per
//! call, a room and free bytes the test sets, and folders a test deletes by hand.

use crate::lock;
use kx_module_api::ports::{BlobName, MAX_BLOB_BYTES, Room, SealedError, SealedShelf};
use std::collections::{BTreeMap, HashMap};
use std::fmt;
use std::sync::{Arc, Mutex};

/// Free bytes on the fake drive until the test sets them: 1 TiB, more than any test fills.
const ROOMY: u64 = 1 << 40;
/// The fake drive's name until the test sets one.
const DRIVE: &str = "D:";

/// One kind of shelf call, to fail or count.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum ShelfCall {
    /// `load`.
    Load,
    /// `save`.
    Save,
    /// `remove`.
    Remove,
    /// `drop_folder`.
    DropFolder,
    /// `folders`.
    Folders,
    /// `names`.
    Names,
    /// `room`.
    Room,
    /// `free`.
    Free,
}

impl ShelfCall {
    /// Whether the port lets this call return `error`.
    fn returns(self, error: SealedError) -> bool {
        match self {
            Self::Load => error != SealedError::BadName,
            Self::Save => matches!(error, SealedError::Locked | SealedError::Io),
            _ => error == SealedError::Io,
        }
    }
}

/// A folder's blobs by name.
type Folder = BTreeMap<BlobName, Vec<u8>>;

/// What the calls did, and what the test set for them.
struct Log {
    folders: BTreeMap<BlobName, Folder>,
    fails: HashMap<ShelfCall, SealedError>,
    calls: HashMap<ShelfCall, usize>,
    free: u64,
    drive: String,
}

impl Default for Log {
    fn default() -> Self {
        Self {
            folders: BTreeMap::new(),
            fails: HashMap::new(),
            calls: HashMap::new(),
            free: ROOMY,
            drive: DRIVE.to_owned(),
        }
    }
}

impl Log {
    /// Counts a `call`, then takes the failure scripted for it.
    fn enter(&mut self, call: ShelfCall) -> Result<(), SealedError> {
        *self.calls.entry(call).or_default() += 1;
        self.fails.remove(&call).map_or(Ok(()), Err)
    }

    /// The bytes every blob holds together.
    fn used(&self) -> u64 {
        let blobs = self.folders.values().flat_map(BTreeMap::values);
        blobs
            .map(|b| u64::try_from(b.len()).unwrap_or(u64::MAX))
            .sum()
    }
}

/// A sealed shelf shared by its clones, which never seals: it keeps each blob's bytes as given, in sorted folders.
///
/// A scripted `Damaged` load sets the blob aside, as the real shelf does; the room's `used` is the bytes held, with no
/// seal's overhead, and its `free` stays as set.
#[derive(Clone, Default)]
pub struct FakeShelf {
    log: Arc<Mutex<Log>>,
}

impl FakeShelf {
    /// An empty shelf on a roomy drive.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Refuses the next `call` with `error`, once, replacing one set for that call and not yet used; a debug build
    /// stops on an error the port never returns there.
    pub fn fail(&self, call: ShelfCall, error: SealedError) {
        debug_assert!(
            call.returns(error),
            "the port never returns {error:?} from {call:?}"
        );
        lock(&self.log).fails.insert(call, error);
    }

    /// How many calls of kind `call` were asked, refused ones too.
    #[must_use]
    pub fn calls(&self, call: ShelfCall) -> usize {
        lock(&self.log).calls.get(&call).copied().unwrap_or(0)
    }

    /// The bytes kept as `name` in `folder`, read without a load.
    #[must_use]
    pub fn blob(&self, folder: &BlobName, name: &BlobName) -> Option<Vec<u8>> {
        let log = lock(&self.log);
        log.folders.get(folder)?.get(name).cloned()
    }

    /// Keeps `bytes` as `name` in `folder` without a save, as an earlier run would have left them.
    pub fn plant(&self, folder: &BlobName, name: &BlobName, bytes: Vec<u8>) {
        let mut log = lock(&self.log);
        log.folders
            .entry(folder.clone())
            .or_default()
            .insert(name.clone(), bytes);
    }

    /// Deletes `folder` and its blobs behind the module's back, as a user does by hand; no call is counted.
    pub fn delete_by_hand(&self, folder: &BlobName) {
        lock(&self.log).folders.remove(folder);
    }

    /// Makes the room tell `bytes` free on the drive.
    pub fn set_free(&self, bytes: u64) {
        lock(&self.log).free = bytes;
    }

    /// Makes the room name the drive `name`, such as `E:`.
    pub fn set_drive(&self, name: &str) {
        name.clone_into(&mut lock(&self.log).drive);
    }
}

/// Names the folders and counts their blobs, never their bytes.
impl fmt::Debug for FakeShelf {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let log = lock(&self.log);
        let folders = log.folders.iter().map(|(n, b)| (n.as_str(), b.len()));
        f.debug_struct("FakeShelf")
            .field("folders", &folders.collect::<Vec<_>>())
            .field("calls", &log.calls)
            .finish_non_exhaustive()
    }
}

impl SealedShelf for FakeShelf {
    fn load(&self, folder: &BlobName, name: &BlobName) -> Result<Option<Vec<u8>>, SealedError> {
        let mut log = lock(&self.log);
        let entered = log.enter(ShelfCall::Load);
        let blobs = log.folders.get_mut(folder);
        if let Err(error) = entered {
            if let (SealedError::Damaged, Some(b)) = (error, blobs) {
                b.remove(name);
            }
            return Err(error);
        }
        Ok(blobs.and_then(|b| b.get(name).cloned()))
    }

    fn save(&self, folder: &BlobName, name: &BlobName, bytes: &[u8]) -> Result<(), SealedError> {
        let mut log = lock(&self.log);
        log.enter(ShelfCall::Save)?;
        if bytes.len() > MAX_BLOB_BYTES {
            return Err(SealedError::Io);
        }
        let blobs = log.folders.entry(folder.clone()).or_default();
        blobs.insert(name.clone(), bytes.to_vec());
        Ok(())
    }

    fn remove(&self, folder: &BlobName, name: &BlobName) -> Result<(), SealedError> {
        let mut log = lock(&self.log);
        log.enter(ShelfCall::Remove)?;
        if let Some(b) = log.folders.get_mut(folder) {
            b.remove(name);
        }
        Ok(())
    }

    fn drop_folder(&self, folder: &BlobName) -> Result<(), SealedError> {
        let mut log = lock(&self.log);
        log.enter(ShelfCall::DropFolder)?;
        log.folders.remove(folder);
        Ok(())
    }

    fn folders(&self) -> Result<Vec<BlobName>, SealedError> {
        let mut log = lock(&self.log);
        log.enter(ShelfCall::Folders)?;
        Ok(log.folders.keys().cloned().collect())
    }

    fn names(&self, folder: &BlobName) -> Result<Vec<BlobName>, SealedError> {
        let mut log = lock(&self.log);
        log.enter(ShelfCall::Names)?;
        let blobs = log.folders.get(folder);
        Ok(blobs
            .map(|b| b.keys().cloned().collect())
            .unwrap_or_default())
    }

    fn room(&self) -> Result<Room, SealedError> {
        let mut log = lock(&self.log);
        log.enter(ShelfCall::Room)?;
        let (free, drive) = (log.free, log.drive.clone());
        Ok(Room {
            used: log.used(),
            free,
            drive,
        })
    }

    fn free(&self) -> Result<u64, SealedError> {
        let mut log = lock(&self.log);
        log.enter(ShelfCall::Free)?;
        Ok(log.free)
    }
}

#[cfg(test)]
mod tests;
