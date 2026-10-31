//! Loading: a read pass over every folder first, so a newer version's list fails the load before anything changes;
//! then each list less the entries whose blobs are gone, a list that will not read kept aside, and blobs of ours no
//! list names removed, but in an aside folder. An id two lists hold after a crash keeps its better copy, and
//! `missing` finds blobs gone by hand while running.

use std::cmp::Ordering;
use std::collections::{BTreeMap, BTreeSet};

use kx_module_api::ports::{BlobName, SealedError};

use super::list::{self, List};
use super::parts::{self, thumb_name};
use super::{Loaded, Saved, Store, StoreError, blob_names, complete, folder_of, place_of};
use crate::entry::{Entry, Place};
use crate::hash::fnv;
use crate::order::{History, newer};

/// What a folder's list blobs held at the read pass.
enum Read {
    /// A list that read, and how many blobs hold it.
    Sound(List, usize),
    /// No list.
    Missing,
    /// A list that would not read at this version: its bytes, joined as far as they went.
    Bad(Vec<u8>),
    /// A head the shelf found damaged and set aside.
    Damaged,
}

/// A folder as the read pass found it, before anything changed.
struct Found {
    /// The folder.
    folder: BlobName,
    /// The place it keeps.
    place: Place,
    /// Its blob names.
    names: BTreeSet<BlobName>,
    /// Its list.
    read: Read,
}

impl Found {
    /// The counter past this folder's: its list's, and past every id its blobs are named after.
    fn next(&self) -> u64 {
        let listed = match &self.read {
            Read::Sound(list, _) => list.next,
            _ => 0,
        };
        let ids = self.names.iter().filter_map(parts::owner);
        let past = ids.map(|(id, _)| id.saturating_add(1)).max();
        listed.max(past.unwrap_or(0))
    }
}

impl Store {
    /// Loads the history from the shelf, forgetting what this store knew before.
    ///
    /// # Errors
    /// `Newer` when a list was saved by a newer version, before any change; else the shelf's, such as `Locked`
    /// without the OS key. The module then fails to start.
    pub fn load(&mut self) -> Result<Loaded, StoreError> {
        self.forget();
        let found = self.scan()?;
        let next = found.iter().map(Found::next).max().unwrap_or(0);
        let (mut all, mut aside) = (Vec::new(), Vec::new());
        for f in found {
            if f.names.iter().any(|n| list::aside_of(n).is_some()) {
                self.kept.insert(f.folder.clone());
            }
            let bytes = match f.read {
                Read::Sound(list, chunks) => {
                    all.extend(self.check(&f.folder, &f.names, list, chunks)?);
                    continue;
                }
                Read::Missing => {
                    self.clean(&f.folder, &f.names, &BTreeSet::new());
                    continue;
                }
                Read::Bad(bytes) => bytes,
                Read::Damaged => Vec::new(),
            };
            self.set_aside(&f.folder, &f.names, &bytes)?;
            aside.push(f.place);
        }
        let entries = self.merge(all)?;
        Ok(Loaded {
            history: History::new(entries, next),
            aside,
        })
    }

    /// The ids in `history` whose content is gone while running, as when their folder or a blob went by hand.
    ///
    /// # Errors
    /// The shelf's error when it cannot be listed.
    pub fn missing(&self, history: &History) -> Result<Vec<u64>, SealedError> {
        let present: BTreeSet<BlobName> = self.shelf.folders()?.into_iter().collect();
        let mut held: BTreeMap<BlobName, BTreeSet<BlobName>> = BTreeMap::new();
        let mut gone = Vec::new();
        for entry in history.entries() {
            let folder = folder_of(self.home(entry))?;
            if !held.contains_key(&folder) {
                let there = present.contains(&folder);
                let names = if there {
                    self.shelf.names(&folder)?
                } else {
                    Vec::new()
                };
                held.insert(folder.clone(), names.into_iter().collect());
            }
            if held
                .get(&folder)
                .is_none_or(|names| !complete(entry, names))
            {
                gone.push(entry.id);
            }
        }
        Ok(gone)
    }

    /// Reads every day and pinned folder's list and names, changing nothing.
    fn scan(&self) -> Result<Vec<Found>, StoreError> {
        let mut found = Vec::new();
        for folder in self.shelf.folders()? {
            let Some(place) = place_of(&folder) else {
                continue;
            };
            let read = self.read_list(&folder, place)?;
            let names = self.shelf.names(&folder)?.into_iter().collect();
            found.push(Found {
                folder,
                place,
                names,
                read,
            });
        }
        Ok(found)
    }

    /// `folder`'s list as read; a missing or damaged extra blob makes it one that would not read.
    ///
    /// # Errors
    /// `Newer` when its head is a newer version's, else the shelf's error but `Damaged`.
    fn read_list(&self, folder: &BlobName, place: Place) -> Result<Read, StoreError> {
        let mut bytes = match self.shelf.load(folder, &list::head_name()?) {
            Ok(Some(bytes)) => bytes,
            Ok(None) => return Ok(Read::Missing),
            Err(SealedError::Damaged) => return Ok(Read::Damaged),
            Err(e) => return Err(e.into()),
        };
        if list::newer(&bytes) {
            return Err(StoreError::Newer);
        }
        let Ok((round, total)) = list::head(&bytes) else {
            return Ok(Read::Bad(bytes));
        };
        let mut chunks = 1;
        while bytes.len() < total {
            match self.shelf.load(folder, &list::chunk_name(round, chunks)?) {
                Ok(Some(chunk)) if !chunk.is_empty() => bytes.extend(chunk),
                Ok(_) | Err(SealedError::Damaged) => return Ok(Read::Bad(bytes)),
                Err(e) => return Err(e.into()),
            }
            chunks += 1;
        }
        Ok(match list::decode(&bytes, place) {
            Ok(list) => Read::Sound(list, chunks),
            Err(_) => Read::Bad(bytes),
        })
    }

    /// The entries of `folder`'s `list` whose blobs are all among `names`, in order; removes each blob of ours none
    /// names.
    fn check(
        &mut self,
        folder: &BlobName,
        names: &BTreeSet<BlobName>,
        list: List,
        chunks: usize,
    ) -> Result<Vec<Entry>, SealedError> {
        let saved = list.entries.len();
        let mut entries: Vec<Entry> = list
            .entries
            .into_iter()
            .filter(|e| complete(e, names))
            .collect();
        let mut changed = entries.len() != saved;
        for e in &mut entries {
            if e.thumb && !thumb_name(e.id).is_ok_and(|t| names.contains(&t)) {
                e.thumb = false;
                changed = true;
            }
        }
        entries.sort_by(newer);
        let mut keep = BTreeSet::from([list::head_name()?]);
        for n in 1..chunks {
            keep.insert(list::chunk_name(list.round, n)?);
        }
        for e in &entries {
            keep.extend(blob_names(e, e.thumb)?);
        }
        self.clean(folder, names, &keep);
        let refs: Vec<&Entry> = entries.iter().collect();
        let sum = (!changed).then(|| fnv(&list::records(&refs)));
        let round = list.round;
        self.lists
            .insert(folder.clone(), Saved { round, chunks, sum });
        Ok(entries)
    }

    /// Removes from `folder` each blob of ours in `names` that `keep` leaves out, but in an aside folder; one that
    /// stays goes next load.
    fn clean(&self, folder: &BlobName, names: &BTreeSet<BlobName>, keep: &BTreeSet<BlobName>) {
        if self.kept.contains(folder) {
            return;
        }
        let ours = |n: &&BlobName| list::is_list(n) || parts::owner(n).is_some();
        for name in names.iter().filter(ours).filter(|n| !keep.contains(*n)) {
            let _ = self.shelf.remove(folder, name);
        }
    }

    /// Keeps one copy of each id, the better as `better` says; the other's folder saves again and loses its blobs.
    fn merge(&mut self, mut all: Vec<Entry>) -> Result<Vec<Entry>, SealedError> {
        all.sort_by(|a, b| a.id.cmp(&b.id).then_with(|| better(a, b)));
        let mut kept: Vec<Entry> = Vec::new();
        for e in all {
            if kept.last().is_none_or(|k| k.id != e.id) {
                kept.push(e);
                continue;
            }
            let folder = folder_of(e.place)?;
            if let Some(saved) = self.lists.get_mut(&folder) {
                saved.sum = None;
            }
            let names = blob_names(&e, true)?;
            self.pending.entry(folder).or_default().extend(names);
        }
        Ok(kept)
    }
}

/// Puts the copy to keep first, as a crash in a move leaves two: the newer number, then the pinned, then the later
/// day.
fn better(a: &Entry, b: &Entry) -> Ordering {
    let day = |e: &Entry| match e.place {
        Place::Day(d) => Some(d),
        Place::Pinned => None,
    };
    let seq = b.seq.cmp(&a.seq);
    seq.then(b.pinned().cmp(&a.pinned()))
        .then(day(b).cmp(&day(a)))
}
