//! Loading: each day and pinned folder's list, less the entries whose blobs are gone; a damaged list sets its folder
//! aside, and blobs of ours no list names are removed. An id two lists hold after a crash keeps its newer copy, and
//! `missing` finds blobs gone by hand while running.

use std::cmp::Ordering;
use std::collections::{BTreeMap, BTreeSet};

use kx_module_api::ports::{BlobName, SealedError};

use super::list::{self, List};
use super::parts::{self, thumb_name};
use super::{Loaded, Saved, Store, blob_names, complete, folder_of, place_of};
use crate::entry::{Entry, Place};
use crate::hash::fnv;
use crate::order::{History, newer};

impl Store {
    /// Loads the history from the shelf, forgetting what this store knew before.
    ///
    /// # Errors
    /// `Locked` without the OS key, `Io` when the shelf cannot be read; the module then fails to start.
    pub fn load(&mut self) -> Result<Loaded, SealedError> {
        self.lists.clear();
        self.pending.clear();
        self.moves.clear();
        let (mut all, mut aside, mut next) = (Vec::new(), Vec::new(), 0);
        for folder in self.shelf.folders()? {
            let Some(place) = place_of(&folder) else {
                continue;
            };
            let read = match self.read_list(&folder, place) {
                Err(SealedError::Damaged) => {
                    aside.push(place);
                    None
                }
                other => other?,
            };
            if let Some((list, _)) = &read {
                next = next.max(list.next);
            }
            all.extend(self.check(&folder, read)?);
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

    /// `folder`'s list and how many blobs hold it, or `None` when it has none.
    ///
    /// # Errors
    /// `Damaged` when a blob of it would not open or is missing, or the list would not read.
    fn read_list(
        &self,
        folder: &BlobName,
        place: Place,
    ) -> Result<Option<(List, usize)>, SealedError> {
        let Some(mut bytes) = self.shelf.load(folder, &list::head_name()?)? else {
            return Ok(None);
        };
        let (round, total) = list::head(&bytes).map_err(|_| SealedError::Damaged)?;
        let mut chunks = 1;
        while bytes.len() < total {
            match self.shelf.load(folder, &list::chunk_name(round, chunks)?)? {
                Some(chunk) if !chunk.is_empty() => bytes.extend(chunk),
                _ => return Err(SealedError::Damaged),
            }
            chunks += 1;
        }
        let list = list::decode(&bytes, place).map_err(|_| SealedError::Damaged)?;
        Ok(Some((list, chunks)))
    }

    /// The entries of `folder`'s list whose blobs are all there, in order; removes each blob of ours none names.
    fn check(
        &mut self,
        folder: &BlobName,
        read: Option<(List, usize)>,
    ) -> Result<Vec<Entry>, SealedError> {
        let names: BTreeSet<BlobName> = self.shelf.names(folder)?.into_iter().collect();
        let Some((list, chunks)) = read else {
            self.clean(folder, &names, &BTreeSet::new());
            return Ok(Vec::new());
        };
        let saved = list.entries.len();
        let mut entries: Vec<Entry> = list
            .entries
            .into_iter()
            .filter(|e| complete(e, &names))
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
        self.clean(folder, &names, &keep);
        let refs: Vec<&Entry> = entries.iter().collect();
        let sum = (!changed).then(|| fnv(&list::records(&refs)));
        let round = list.round;
        self.lists
            .insert(folder.clone(), Saved { round, chunks, sum });
        Ok(entries)
    }

    /// Removes from `folder` each blob of ours in `names` that `keep` leaves out; one that stays goes next load.
    fn clean(&self, folder: &BlobName, names: &BTreeSet<BlobName>, keep: &BTreeSet<BlobName>) {
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
