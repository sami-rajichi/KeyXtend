//! Saving: first the blobs of each pending move, then each folder's list when its records changed, the folders a move
//! left last, and only after every list the blobs the moves left. A folder that fails is skipped and the rest go on;
//! a crash leaves at most a duplicate, which the next load merges.

use std::collections::{BTreeMap, BTreeSet};

use kx_module_api::ports::{BlobName, SealedError};

use super::{Saved, Store, blob_names, folder_of, list};
use crate::entry::Entry;
use crate::hash::fnv;
use crate::order::History;

/// The entries of each folder.
type Groups<'h> = BTreeMap<BlobName, Vec<&'h Entry>>;

/// A folder whose list saved, with the blob names its list uses and those its held copies use.
type Written = (BlobName, BTreeSet<BlobName>, BTreeSet<BlobName>);

impl Store {
    /// Saves the moves left pending, then the list of each folder whose entries changed in `history`, and drops a
    /// folder left with none; a move or folder that fails is skipped.
    ///
    /// # Errors
    /// The first shelf error; what is left saves at the next call.
    pub fn save(&mut self, history: &History) -> Result<(), SealedError> {
        let mut first = self.carry_all(history);
        let groups = self.groups(history)?;
        let mut written = Vec::new();
        for folder in self.order(&groups) {
            match self.write_one(history, &groups, &folder) {
                Ok(done) => written.push(done),
                Err(e) => first = first.and(Err(e)),
            }
        }
        for (folder, used, held) in written {
            first = first.and(self.flush(&folder, &used, &held));
        }
        first
    }

    /// The entries of `history` by folder, but those whose move is pending, which the folder they left holds.
    fn groups<'h>(&self, history: &'h History) -> Result<Groups<'h>, SealedError> {
        let mut groups = Groups::new();
        let settled = history.entries().iter();
        for entry in settled.filter(|e| !self.moves.contains_key(&e.id)) {
            groups
                .entry(folder_of(entry.place)?)
                .or_default()
                .push(entry);
        }
        Ok(groups)
    }

    /// Every folder to write, those no move left first, so a list a move reaches saves before the one it left.
    fn order(&self, groups: &Groups<'_>) -> Vec<BlobName> {
        let mut folders: BTreeSet<BlobName> = groups.keys().cloned().collect();
        folders.extend(self.lists.keys().cloned());
        folders.extend(self.pending.keys().cloned());
        folders.extend(self.left.values().cloned());
        let sources: BTreeSet<&BlobName> = self.left.values().collect();
        let (last, mut order): (Vec<_>, Vec<_>) =
            folders.into_iter().partition(|f| sources.contains(f));
        order.extend(last);
        order
    }

    /// Writes `folder`'s list of its group and held copies, then ends the holds it fulfils; gives what its flush needs.
    fn write_one(
        &mut self,
        history: &History,
        groups: &Groups<'_>,
        folder: &BlobName,
    ) -> Result<Written, SealedError> {
        let held = self.held(history, folder);
        let mut entries = groups.get(folder).cloned().unwrap_or_default();
        entries.extend(&held);
        self.write(folder, &entries, history.next())?;
        self.confirm(history, folder);
        if !self.pending.contains_key(folder) {
            return Ok((folder.clone(), BTreeSet::new(), BTreeSet::new()));
        }
        let used = names_of(entries.iter().copied())?;
        Ok((folder.clone(), used, names_of(&held)?))
    }

    /// Saves `folder`'s list of `entries` when they changed, or drops the folder when none is left.
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
        if saved.and_then(|s| s.sum) == Some(sum) {
            return Ok(());
        }
        let round = saved.map_or(0, |s| list::turn(s.round));
        let chunks = self.save_list(folder, round, &list::frame(round, next, &records))?;
        let sum = Some(sum);
        self.lists
            .insert(folder.clone(), Saved { round, chunks, sum });
        if let Some(old) = saved {
            self.drop_chunks(folder, old);
        }
        Ok(())
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

    /// Drops `folder`, left with no entry, when it holds a list or blobs of ours; an aside folder, or one a move still
    /// needs, loses only its list.
    fn drop_empty(&mut self, folder: &BlobName) -> Result<(), SealedError> {
        let saved = self.lists.get(folder).copied();
        if self.kept.contains(folder) || self.moving(folder) {
            if let Some(old) = saved {
                self.shelf.remove(folder, &list::head_name()?)?;
                self.lists.remove(folder);
                self.drop_chunks(folder, old);
            }
            return Ok(());
        }
        if saved.is_some() || self.pending.contains_key(folder) {
            self.shelf.drop_folder(folder)?;
        }
        self.forget_folder(folder);
        Ok(())
    }

    /// Removes the blobs a move left in `folder` that its saved list does not name: those `used` names stay, and
    /// those `held` names stay queued. One that fails is tried again.
    fn flush(
        &mut self,
        folder: &BlobName,
        used: &BTreeSet<BlobName>,
        held: &BTreeSet<BlobName>,
    ) -> Result<(), SealedError> {
        let Some(left) = self.pending.remove(folder) else {
            return Ok(());
        };
        let (mut stay, mut failed) = (BTreeSet::new(), false);
        for name in left {
            if held.contains(&name) {
                stay.insert(name);
            } else if !used.contains(&name) && self.shelf.remove(folder, &name).is_err() {
                failed = true;
                stay.insert(name);
            }
        }
        if !stay.is_empty() {
            self.pending.insert(folder.clone(), stay);
        }
        if failed { Err(SealedError::Io) } else { Ok(()) }
    }
}

/// The names of every blob of `entries`, thumbnails too.
fn names_of<'e>(
    entries: impl IntoIterator<Item = &'e Entry>,
) -> Result<BTreeSet<BlobName>, SealedError> {
    let mut names = BTreeSet::new();
    for e in entries {
        names.extend(blob_names(e, true)?);
    }
    Ok(names)
}
