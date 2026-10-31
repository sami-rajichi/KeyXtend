//! Saving: first the blobs of each move, then each folder's list when its records changed, the folders a move left
//! last, then the blobs the move left. A crash leaves at most a duplicate, which the next load merges; a move that
//! fails stays pending, and each save tries it again before any list.

use std::collections::BTreeSet;

use kx_module_api::ports::{BlobName, SealedError};

use super::parts::{self, Blob};
use super::{Saved, Store, blob_names, folder_of, group, list};
use crate::entry::{Entry, Place};
use crate::hash::fnv;
use crate::order::History;

impl Store {
    /// Saves the moves left pending, then the list of each folder whose entries changed in `history`, and drops a
    /// folder left with none.
    ///
    /// # Errors
    /// The shelf's error; what is left saves at the next call.
    pub fn save(&mut self, history: &History) -> Result<(), SealedError> {
        let moving: Vec<u64> = self.moves.keys().copied().collect();
        for id in moving {
            match history.get(id) {
                Some(entry) => {
                    self.carry(entry)?;
                }
                None => {
                    self.moves.remove(&id);
                }
            }
        }
        let groups = group(history)?;
        let mut folders: BTreeSet<BlobName> = groups.keys().cloned().collect();
        folders.extend(self.lists.keys().cloned());
        folders.extend(self.pending.keys().cloned());
        let (left, rest): (Vec<_>, Vec<_>) = folders
            .into_iter()
            .partition(|f| self.pending.contains_key(f));
        for folder in rest.into_iter().chain(left) {
            let entries = groups.get(&folder).map_or(&[][..], Vec::as_slice);
            self.write(&folder, entries, history.next())?;
        }
        Ok(())
    }

    /// Follows the move of `id` in `history` from `from`, then saves: its blobs are copied, the new folder's list
    /// saved before the old one's, then the old blobs removed. False when its content is gone, as by hand.
    ///
    /// # Errors
    /// The shelf's error; the move stays pending, and reads find the blobs where they are.
    pub fn moved(&mut self, history: &History, id: u64, from: Place) -> Result<bool, SealedError> {
        self.moves.entry(id).or_insert(from);
        let kept = if let Some(entry) = history.get(id) {
            self.carry(entry)?
        } else {
            self.moves.remove(&id);
            false
        };
        self.save(history)?;
        Ok(kept)
    }

    /// Copies `entry`'s blobs from where a pending move left them into its folder, and marks the old ones to remove;
    /// false when they are gone, which ends the move.
    fn carry(&mut self, entry: &Entry) -> Result<bool, SealedError> {
        let from = self.home(entry);
        if from == entry.place {
            self.moves.remove(&entry.id);
            return Ok(true);
        }
        let (old, new) = (folder_of(from)?, folder_of(entry.place)?);
        let names = blob_names(entry, entry.thumb)?;
        for name in &names {
            let thumb = parts::owner(name).is_some_and(|(_, b)| b == Blob::Thumb);
            // A copy left by a failed try holds the same bytes, so the next try writes over it.
            match self.shelf.load(&old, name) {
                Ok(Some(bytes)) => self.shelf.save(&new, name, &bytes)?,
                Ok(None) | Err(SealedError::Damaged) if thumb => continue,
                Ok(None) | Err(SealedError::Damaged) => {
                    self.moves.remove(&entry.id);
                    return Ok(false);
                }
                Err(e) => return Err(e),
            }
            self.cancel(&new, name);
        }
        self.moves.remove(&entry.id);
        self.pending.entry(old).or_default().extend(names);
        Ok(true)
    }

    /// Saves `folder`'s list of `entries` when they changed, or drops the folder when none is left; then removes
    /// the blobs a move left there.
    fn write(
        &mut self,
        folder: &BlobName,
        entries: &[&Entry],
        next: u64,
    ) -> Result<(), SealedError> {
        if entries.is_empty() {
            return self.drop_empty(folder);
        }
        let records = list::records(entries);
        let sum = fnv(&records);
        let saved = self.lists.get(folder).copied();
        if saved.and_then(|s| s.sum) != Some(sum) {
            let round = saved.map_or(0, |s| list::turn(s.round));
            let chunks = self.save_list(folder, round, &list::frame(round, next, &records))?;
            let sum = Some(sum);
            self.lists
                .insert(folder.clone(), Saved { round, chunks, sum });
            if let Some(old) = saved {
                self.drop_chunks(folder, old);
            }
        }
        self.flush(folder, entries)
    }

    /// Saves `bytes` as round `round`'s extra blobs, then the head that names them; gives how many blobs hold them.
    fn save_list(&self, folder: &BlobName, round: u8, bytes: &[u8]) -> Result<usize, SealedError> {
        let chunks: Vec<&[u8]> = bytes.chunks(self.blob).collect();
        let Some((head, rest)) = chunks.split_first() else {
            return Err(SealedError::Io);
        };
        for (n, chunk) in (1..).zip(rest) {
            self.shelf
                .save(folder, &list::chunk_name(round, n)?, chunk)?;
        }
        self.shelf.save(folder, &list::head_name()?, head)?;
        Ok(chunks.len())
    }

    /// Removes the extra blobs of an older list; one that stays goes at the next load.
    fn drop_chunks(&self, folder: &BlobName, old: Saved) {
        for name in (1..old.chunks).filter_map(|n| list::chunk_name(old.round, n).ok()) {
            let _ = self.shelf.remove(folder, &name);
        }
    }

    /// Drops `folder`, left with no entry, when it holds a list or blobs of ours.
    fn drop_empty(&mut self, folder: &BlobName) -> Result<(), SealedError> {
        if self.lists.contains_key(folder) || self.pending.contains_key(folder) {
            self.shelf.drop_folder(folder)?;
        }
        self.lists.remove(folder);
        self.pending.remove(folder);
        Ok(())
    }

    /// Removes the blobs a move left in `folder`, except those `entries` name; one that fails is tried again.
    fn flush(&mut self, folder: &BlobName, entries: &[&Entry]) -> Result<(), SealedError> {
        if !self.pending.contains_key(folder) {
            return Ok(());
        }
        let mut used = BTreeSet::new();
        for e in entries {
            used.extend(blob_names(e, true)?);
        }
        let left = self.pending.remove(folder).unwrap_or_default();
        let stuck: BTreeSet<BlobName> = left
            .into_iter()
            .filter(|n| !used.contains(n) && self.shelf.remove(folder, n).is_err())
            .collect();
        if stuck.is_empty() {
            return Ok(());
        }
        self.pending.insert(folder.clone(), stuck);
        Err(SealedError::Io)
    }
}
