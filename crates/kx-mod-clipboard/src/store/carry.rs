//! Moves and deletes: an entry's blobs are copied to its new folder, and the folder it left keeps naming it in its
//! list until the list it moved to saves. A copy that fails stays pending; content found gone ends the move and
//! queues every blob of it for removal; a delete removes them wherever a move left them.

use std::collections::BTreeSet;

use kx_module_api::ports::{BlobName, SealedError};

use super::parts::{self, Blob};
use super::{Store, blob_names, folder_of, place_of};
use crate::entry::{Entry, Place};
use crate::order::History;

impl Store {
    /// Follows the move of `id` in `history` from `from`, then saves: its blobs are copied, the new folder's list
    /// saved before the old one's, then the old blobs removed. False when its content is gone, as by hand.
    ///
    /// # Errors
    /// The shelf's error; the move stays pending, and reads find the blobs where they are.
    pub fn moved(&mut self, history: &History, id: u64, from: Place) -> Result<bool, SealedError> {
        let kept = match history.get(id) {
            Some(entry) => {
                if from != entry.place {
                    self.moves.entry(id).or_insert(from);
                    self.left.entry(id).or_insert(folder_of(from)?);
                }
                self.carry(entry)?
            }
            None => false,
        };
        let saved = self.save(history);
        if kept {
            saved.map(|()| true)
        } else {
            Ok(false)
        }
    }

    /// Removes every blob of `entry` at once, in its folder and wherever a move left one; its list follows at the
    /// next save.
    ///
    /// # Errors
    /// `Io` when a blob could not be removed; the others are gone.
    pub fn delete(&mut self, entry: &Entry) -> Result<(), SealedError> {
        let home = self.moves.remove(&entry.id).map(folder_of).transpose()?;
        let held = self.left.remove(&entry.id);
        let folders: BTreeSet<BlobName> = [Some(folder_of(entry.place)?), home, held]
            .into_iter()
            .flatten()
            .collect();
        let mut failed = false;
        for folder in &folders {
            for name in blob_names(entry, true)? {
                failed |= self.shelf.remove(folder, &name).is_err();
            }
        }
        let disk = &self.shelf;
        let mine = |n: &BlobName| parts::owner(n).is_some_and(|(id, _)| id == entry.id);
        for (f, left) in &mut self.pending {
            left.retain(|n| !mine(n) || disk.remove(f, n).is_err());
            failed |= left.iter().any(mine);
        }
        self.pending.retain(|_, left| !left.is_empty());
        if failed { Err(SealedError::Io) } else { Ok(()) }
    }

    /// Carries each pending move whose entry is still in `history`, and forgets the others; a failed one is skipped.
    pub(super) fn carry_all(&mut self, history: &History) -> Result<(), SealedError> {
        self.moves.retain(|id, _| history.get(*id).is_some());
        self.left.retain(|id, _| history.get(*id).is_some());
        let mut first = Ok(());
        let moving: Vec<u64> = self.moves.keys().copied().collect();
        for entry in moving.iter().filter_map(|id| history.get(*id)) {
            first = first.and(self.carry(entry).map(|_| ()));
        }
        first
    }

    /// Copies `entry`'s blobs from where a pending move left them into its folder, and queues the old ones for
    /// removal; false when they are gone, which ends the move.
    fn carry(&mut self, entry: &Entry) -> Result<bool, SealedError> {
        let from = self.home(entry);
        if from == entry.place {
            self.moves.remove(&entry.id);
            return Ok(true);
        }
        let (old, new) = (folder_of(from)?, folder_of(entry.place)?);
        let names = blob_names(entry, entry.thumb)?;
        for (n, name) in names.iter().enumerate() {
            let thumb = parts::owner(name).is_some_and(|(_, b)| b == Blob::Thumb);
            // A copy left by a failed try holds the same bytes, so the next try writes over it.
            let copied = match self.shelf.load(&old, name) {
                Ok(Some(bytes)) => self.shelf.save(&new, name, &bytes),
                Ok(None) | Err(SealedError::Damaged) if thumb => continue,
                Ok(None) | Err(SealedError::Damaged) => {
                    self.lost(entry.id, (&old, &names), (&new, &names[..n]));
                    return Ok(false);
                }
                Err(e) => Err(e),
            };
            if let Err(e) = copied {
                self.queue(&new, &names[..=n]);
                return Err(e);
            }
        }
        for name in &names {
            self.cancel(&new, name);
        }
        self.moves.remove(&entry.id);
        self.queue(&old, &names);
        Ok(true)
    }

    /// Ends the move of `id`, whose content is gone; the blobs left in the old folder and those copied into the new
    /// one are queued, so a delete or the next save removes them.
    fn lost(&mut self, id: u64, old: (&BlobName, &[BlobName]), new: (&BlobName, &[BlobName])) {
        self.moves.remove(&id);
        self.left.remove(&id);
        self.queue(old.0, old.1);
        self.queue(new.0, new.1);
    }

    /// Queues `names` in `folder` for removal once its saved list no longer names them.
    fn queue(&mut self, folder: &BlobName, names: &[BlobName]) {
        if !names.is_empty() {
            let left = self.pending.entry(folder.clone()).or_default();
            left.extend(names.iter().cloned());
        }
    }

    /// Copies, at `folder`'s place, of the entries a move took from it: its list keeps them until the list they
    /// moved to saves, and while their move is pending.
    pub(super) fn held(&self, history: &History, folder: &BlobName) -> Vec<Entry> {
        let Some(place) = place_of(folder) else {
            return Vec::new();
        };
        let ids = self.left.iter().filter(|(_, f)| *f == folder);
        ids.filter_map(|(id, _)| history.get(*id))
            .filter(|e| e.place != place || self.moves.contains_key(&e.id))
            .map(|e| Entry { place, ..e.clone() })
            .collect()
    }

    /// Ends the hold on each moved entry that `folder`'s list, just saved, now names.
    pub(super) fn confirm(&mut self, history: &History, folder: &BlobName) {
        let moves = &self.moves;
        let here = |id: &u64| {
            let e = history.get(*id);
            e.is_some_and(|e| folder_of(e.place).is_ok_and(|f| f == *folder))
        };
        self.left
            .retain(|id, _| moves.contains_key(id) || !here(id));
    }
}
