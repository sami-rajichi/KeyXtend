//! The store (spec §7 "Store", ADR-0034 decision 4): the history sealed on the shelf, one folder per calendar day and
//! a `pinned` one, each with its list and its entries' blobs. A load drops what vanished by hand, sets a damaged
//! list's folder aside and cleans what a crash left; moves and deletes leave each entry whole or gone, never half.

mod codec;
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
    /// The places whose list was damaged, so set aside with its entries; the module tells it once.
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

    /// Removes every blob of `entry` at once, in its folder and where a move left one; its list follows at the next
    /// save.
    ///
    /// # Errors
    /// `Io` when a blob could not be removed; the others are gone.
    pub fn delete(&mut self, entry: &Entry) -> Result<(), SealedError> {
        let mut failed = false;
        let home = self.moves.remove(&entry.id);
        for place in [Some(entry.place), home].into_iter().flatten() {
            let folder = folder_of(place)?;
            for name in blob_names(entry, true)? {
                failed |= self.shelf.remove(&folder, &name).is_err();
            }
        }
        let disk = &self.shelf;
        let mine = |n: &BlobName| parts::owner(n).is_some_and(|(id, _)| id == entry.id);
        for (f, left) in &mut self.pending {
            left.retain(|n| !mine(n) || disk.remove(f, n).is_err());
            failed |= left.iter().any(mine);
        }
        if failed { Err(SealedError::Io) } else { Ok(()) }
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

    /// Drops whole each day folder `keep_days` or more days before `today`, but one a pending move's blobs wait in.
    ///
    /// # Errors
    /// The shelf's error; the folders left drop at the next call.
    pub fn drop_past(&mut self, today: Date, keep_days: u8) -> Result<(), SealedError> {
        for folder in self.shelf.folders()? {
            let Some(Place::Day(day)) = place_of(&folder) else {
                continue;
            };
            if past(day, today, keep_days) && !self.waiting(&folder) {
                self.shelf.drop_folder(&folder)?;
                self.lists.remove(&folder);
                self.pending.remove(&folder);
            }
        }
        Ok(())
    }

    /// Where `entry`'s blobs are: where a pending move found them, else its place.
    fn home(&self, entry: &Entry) -> Place {
        self.moves.get(&entry.id).copied().unwrap_or(entry.place)
    }

    /// True when a pending move's blobs wait in `folder`.
    fn waiting(&self, folder: &BlobName) -> bool {
        let here = |p: &Place| folder_of(*p).is_ok_and(|f| f == *folder);
        self.moves.values().any(here)
    }

    /// Keeps `name` in `folder` from being removed by a move that left it there earlier.
    fn cancel(&mut self, folder: &BlobName, name: &BlobName) {
        if let Some(left) = self.pending.get_mut(folder) {
            left.remove(name);
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

/// The entries of `history` by folder, each in order.
fn group(history: &History) -> Result<BTreeMap<BlobName, Vec<&Entry>>, SealedError> {
    let mut groups: BTreeMap<BlobName, Vec<&Entry>> = BTreeMap::new();
    for entry in history.entries() {
        groups
            .entry(folder_of(entry.place)?)
            .or_default()
            .push(entry);
    }
    Ok(groups)
}

#[cfg(test)]
mod props;
#[cfg(test)]
mod tests;
