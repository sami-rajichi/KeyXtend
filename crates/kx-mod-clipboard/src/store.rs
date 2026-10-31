//! The store (spec §7 "Store", ADR-0034 decision 4): the history sealed on the shelf, one folder per calendar day and
//! a `pinned` one, each with its list and its entries' blobs. A load refuses a newer version's list, drops what
//! vanished by hand, keeps a list that will not read aside with its folder's content, and cleans what a crash left;
//! moves and deletes leave each entry whole or gone, never half.

mod aside;
mod carry;
mod codec;
mod error;
mod list;
mod load;
mod parts;
mod save;

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::sync::Arc;

use kx_module_api::ports::{
    BlobName, ClipPart, Date, MAX_BLOB_BYTES, Pixels, SealedError, SealedShelf,
};

use crate::entry::{Entry, Place};
use crate::keep::past;
use crate::order::History;
pub use error::StoreError;
use parts::{part_name, thumb_name};

/// The pinned folder's name; day folders are named `yyyy-mm-dd`.
const PINNED: &str = "pinned";

/// What the store needs from the settings; the module fills it.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct StoreConfig {
    /// The longest side a thumbnail keeps, in pixels (`thumb_px`); a larger saved one reads as damaged.
    pub thumb_px: u32,
    /// The most bytes one blob holds; the module passes the port's `MAX_BLOB_BYTES`, which caps it.
    pub blob_bytes: usize,
}

/// What a load found.
#[derive(Debug)]
pub struct Loaded {
    /// The entries and the counter.
    pub history: History,
    /// The places whose list would not read, so it was kept aside in its folder; the module tells it once.
    pub aside: Vec<Place>,
}

/// What the store knows of a folder's saved list.
#[derive(Copy, Clone, Debug)]
struct Saved {
    /// The round of its extra blobs.
    round: u8,
    /// How many blobs hold it, the head first.
    chunks: usize,
    /// The hash of its records, or `None` when it must save again.
    sum: Option<u64>,
}

/// The history's blobs on the sealed shelf; the actor drives it from one thread.
pub struct Store {
    /// Where the blobs live.
    shelf: Arc<dyn SealedShelf>,
    /// The longest thumbnail side.
    thumb_px: u32,
    /// The most bytes one blob holds.
    blob: usize,
    /// Each folder's saved list.
    lists: BTreeMap<BlobName, Saved>,
    /// Blobs to remove from a folder once its saved list no longer names them, as a move leaves.
    pending: BTreeMap<BlobName, BTreeSet<BlobName>>,
    /// Entries whose blobs still sit where a move found them, until a save copies them.
    moves: BTreeMap<u64, Place>,
    /// Moved entries the folder they left still names, until the list they moved to saves.
    left: BTreeMap<u64, BlobName>,
    /// Folders holding a list kept aside, whose orphans no load cleans.
    kept: BTreeSet<BlobName>,
    /// Moves a save found gone, until the module takes them.
    gone: Vec<u64>,
}

impl Store {
    /// A store over `shelf`; `load` reads what it holds.
    #[must_use]
    pub fn new(shelf: Arc<dyn SealedShelf>, cfg: StoreConfig) -> Self {
        Self {
            shelf,
            thumb_px: cfg.thumb_px,
            blob: cfg.blob_bytes.clamp(list::HEAD_LEN, MAX_BLOB_BYTES),
            lists: BTreeMap::new(),
            pending: BTreeMap::new(),
            moves: BTreeMap::new(),
            left: BTreeMap::new(),
            kept: BTreeSet::new(),
            gone: Vec::new(),
        }
    }

    /// Saves `parts`, the content of `entry`, new in the history, in its folder; gives how many blobs hold it.
    ///
    /// # Errors
    /// The shelf's error; the blobs saved before it are removed.
    pub fn put(&mut self, entry: &Entry, parts: &[ClipPart]) -> Result<u16, SealedError> {
        let folder = folder_of(entry.place)?;
        let bytes = parts::encode(parts);
        let chunks: Vec<&[u8]> = bytes.chunks(self.blob).collect();
        let count = u16::try_from(chunks.len()).map_err(|_| SealedError::Io)?;
        for (n, chunk) in chunks.iter().enumerate() {
            let name = part_name(entry.id, n)?;
            if let Err(e) = self.shelf.save(&folder, &name, chunk) {
                self.unput(&folder, entry.id, n);
                return Err(e);
            }
            self.cancel(&folder, &name);
        }
        Ok(count)
    }

    /// The content of `entry`; `None` when a blob of it is gone, as when its folder was deleted by hand.
    ///
    /// # Errors
    /// `Damaged` when a blob would not open or the content would not read, else the shelf's error.
    pub fn parts(&self, entry: &Entry) -> Result<Option<Vec<ClipPart>>, SealedError> {
        if entry.parts == 0 {
            return Ok(None);
        }
        let folder = folder_of(self.home(entry))?;
        let mut bytes = Vec::new();
        for n in 0..usize::from(entry.parts) {
            match self.shelf.load(&folder, &part_name(entry.id, n)?)? {
                Some(chunk) => bytes.extend(chunk),
                None => return Ok(None),
            }
        }
        parts::decode(&bytes)
            .map(Some)
            .map_err(|_| SealedError::Damaged)
    }

    /// Saves `thumb` as `entry`'s thumbnail.
    ///
    /// # Errors
    /// The shelf's error.
    pub fn put_thumb(&mut self, entry: &Entry, thumb: &Pixels) -> Result<(), SealedError> {
        let (folder, name) = (folder_of(self.home(entry))?, thumb_name(entry.id)?);
        self.shelf
            .save(&folder, &name, &parts::encode_thumb(thumb))?;
        self.cancel(&folder, &name);
        Ok(())
    }

    /// `entry`'s thumbnail, or `None` when none is saved.
    ///
    /// # Errors
    /// `Damaged` when it would not open or read, or a side is over `thumb_px`, else the shelf's error.
    pub fn thumb(&self, entry: &Entry) -> Result<Option<Pixels>, SealedError> {
        let bytes = self
            .shelf
            .load(&folder_of(self.home(entry))?, &thumb_name(entry.id)?)?;
        let read = |b: Vec<u8>| parts::decode_thumb(&b, self.thumb_px);
        bytes
            .map(read)
            .transpose()
            .map_err(|_| SealedError::Damaged)
    }

    /// Drops whole each day folder `keep_days` or more days before `today`, an aside one too, but one a move still
    /// needs; a folder that fails is skipped.
    ///
    /// # Errors
    /// The first shelf error; the folders left drop at the next call.
    pub fn drop_past(&mut self, today: Date, keep_days: u8) -> Result<(), SealedError> {
        let mut first = Ok(());
        for folder in self.shelf.folders()? {
            let Some(Place::Day(day)) = place_of(&folder) else {
                continue;
            };
            if !past(day, today, keep_days) || self.busy(&folder) {
                continue;
            }
            match self.shelf.drop_folder(&folder) {
                Ok(()) => self.forget_folder(&folder),
                Err(e) => first = first.and(Err(e)),
            }
        }
        first
    }

    /// Where `entry`'s blobs are: where a pending move found them, else its place.
    fn home(&self, entry: &Entry) -> Place {
        self.moves.get(&entry.id).copied().unwrap_or(entry.place)
    }

    /// True when a move still needs `folder`: blobs in it wait for a removal, or `moving` says so.
    fn busy(&self, folder: &BlobName) -> bool {
        self.pending.contains_key(folder) || self.moving(folder)
    }

    /// True when a pending move's blobs wait in `folder` for a copy, or its list holds a moved entry.
    fn moving(&self, folder: &BlobName) -> bool {
        let here = |p: &Place| folder_of(*p).is_ok_and(|f| f == *folder);
        self.moves.values().any(here) || self.left.values().any(|f| f == folder)
    }

    /// Forgets what this store knew of every folder, before a load.
    fn forget(&mut self) {
        self.lists.clear();
        self.pending.clear();
        self.moves.clear();
        self.left.clear();
        self.kept.clear();
        self.gone.clear();
    }

    /// Forgets what this store knew of `folder`, once it is dropped.
    fn forget_folder(&mut self, folder: &BlobName) {
        self.lists.remove(folder);
        self.pending.remove(folder);
        self.kept.remove(folder);
    }

    /// Keeps `name` in `folder` from being removed by a move that left it there earlier.
    fn cancel(&mut self, folder: &BlobName, name: &BlobName) {
        if let Some(left) = self.pending.get_mut(folder) {
            left.remove(name);
            if left.is_empty() {
                self.pending.remove(folder);
            }
        }
    }

    /// Removes content blobs 0 to `n` of entry `id` after a failed save; one that stays goes at the next load.
    fn unput(&self, folder: &BlobName, id: u64, n: usize) {
        for name in (0..n).filter_map(|i| part_name(id, i).ok()) {
            let _ = self.shelf.remove(folder, &name);
        }
    }
}

/// Counts the folders, never what they hold.
impl fmt::Debug for Store {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Store")
            .field("lists", &self.lists.len())
            .field("pending", &self.pending.len())
            .field("moves", &self.moves.len())
            .field("left", &self.left.len())
            .field("kept", &self.kept.len())
            .field("gone", &self.gone.len())
            .finish_non_exhaustive()
    }
}

/// The folder of `place`: its day as `yyyy-mm-dd`, or `pinned`.
fn folder_of(place: Place) -> Result<BlobName, SealedError> {
    match place {
        Place::Day(day) => BlobName::new(&day.to_string()),
        Place::Pinned => BlobName::new(PINNED),
    }
}

/// The place `folder` keeps; `None` for a folder of another name.
fn place_of(folder: &BlobName) -> Option<Place> {
    if folder.as_str() == PINNED {
        Some(Place::Pinned)
    } else {
        Date::parse(folder.as_str()).map(Place::Day)
    }
}

/// The names of `entry`'s content blobs, then its thumbnail's when `thumb`.
fn blob_names(entry: &Entry, thumb: bool) -> Result<Vec<BlobName>, SealedError> {
    let parts = (0..usize::from(entry.parts)).map(|n| part_name(entry.id, n));
    let mut names = parts.collect::<Result<Vec<_>, _>>()?;
    if thumb {
        names.push(thumb_name(entry.id)?);
    }
    Ok(names)
}

/// True when `entry` has content and every blob of it is among `names`.
fn complete(entry: &Entry, names: &BTreeSet<BlobName>) -> bool {
    let there = |n| part_name(entry.id, n).is_ok_and(|b| names.contains(&b));
    entry.parts > 0 && (0..usize::from(entry.parts)).all(there)
}

#[cfg(test)]
mod props;
#[cfg(test)]
mod tests;
