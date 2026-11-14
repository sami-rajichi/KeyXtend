//! Opening the store: every text, then the recordings' blobs, then the numbers, and a newer version's file stops the
//! open before anything changes. A blob that will not read is dropped and counted; a recording cut short by a crash
//! loses its blobs and its text is written again without it; a blob no text claims goes.

use std::collections::BTreeMap;
use std::sync::Arc;

use kx_module_api::Zeroizing;
use kx_module_api::ports::{BlobName, MAX_BLOB_BYTES, SealedError, SealedShelf};

use super::codec::Bad;
use super::names::{CHECKS, COUNTS, NUMBERS, SCORES, SOUNDS, TEXTS, name, sound_of, text_id};
use super::{Opened, Store, StoreError, numbers, text};
use crate::history::{Edit, Entry, History, Sound};

/// What the reads found, before anything changes.
#[derive(Default)]
struct Found {
    entries: Vec<Entry>,
    /// Past every id a text or a recording's blob names, so none is used again.
    next: u64,
    /// Blobs that would not read.
    damaged: usize,
    /// Blobs to remove, by folder and name.
    trash: Vec<(BlobName, BlobName)>,
    /// Texts to write again without their recording.
    edit: Edit,
}

impl Found {
    fn seen(&mut self, id: u64) {
        self.next = self.next.max(id.saturating_add(1));
    }

    /// Notes `blob` in `folder` as damaged, to remove.
    fn damage(&mut self, folder: &BlobName, blob: &BlobName) {
        self.damaged += 1;
        self.trash.push((folder.clone(), blob.clone()));
    }
}

impl Store {
    /// Opens the voice folder on `shelf`, with blobs of `blob_bytes` at most, capped by the port's `MAX_BLOB_BYTES`.
    ///
    /// # Errors
    /// `Newer` when a newer version saved a file, found before anything changed; else the shelf's error.
    pub fn open(shelf: Arc<dyn SealedShelf>, blob_bytes: usize) -> Result<Opened, StoreError> {
        let blob = blob_bytes.clamp(1, MAX_BLOB_BYTES);
        let mut store = Self {
            shelf,
            blob,
            pending: Edit::default(),
        };
        let mut found = Found::default();
        store.read_texts(&mut found)?;
        store.read_sounds(&mut found)?;
        let counts = store.number(COUNTS, numbers::decode_counts, &mut found)?;
        let checks = store.number(CHECKS, numbers::decode_checks, &mut found)?;
        let scores = store.number(SCORES, numbers::decode_scores, &mut found)?;
        let history = History::new(std::mem::take(&mut found.entries), found.next);
        let damaged = found.damaged;
        store.clean(&history, found);
        Ok(Opened {
            store,
            history,
            counts,
            checks,
            scores,
            damaged,
        })
    }

    /// Reads each text whose blob is named by its id; another name is left alone.
    fn read_texts(&self, found: &mut Found) -> Result<(), StoreError> {
        let folder = name(TEXTS)?;
        for blob in self.shelf.names(&folder)? {
            let Some(id) = text_id(&blob) else {
                continue;
            };
            found.seen(id);
            let Some(bytes) = self.load(&folder, &blob, found)? else {
                continue;
            };
            match text::decode(&bytes) {
                Ok(entry) if entry.id == id => found.entries.push(entry),
                Err(Bad::Newer) => return Err(StoreError::Newer),
                _ => found.damage(&folder, &blob),
            }
        }
        Ok(())
    }

    /// Keeps each text's recording only when all its blobs are there; the rest of the recordings' blobs go.
    fn read_sounds(&self, found: &mut Found) -> Result<(), StoreError> {
        let folder = name(SOUNDS)?;
        let mut held: BTreeMap<u64, Vec<(u32, BlobName)>> = BTreeMap::new();
        for blob in self.shelf.names(&folder)? {
            if let Some((id, n)) = sound_of(&blob) {
                found.seen(id);
                held.entry(id).or_default().push((n, blob));
            }
        }
        for entry in &mut found.entries {
            let blobs = held.remove(&entry.id).unwrap_or_default();
            if entry.sound.is_some_and(|sound| whole(sound, &blobs)) {
                continue;
            }
            if entry.sound.take().is_some() {
                found.edit.put.insert(entry.id);
            }
            found
                .trash
                .extend(blobs.into_iter().map(|(_, b)| (folder.clone(), b)));
        }
        let orphans = held.into_values().flatten();
        found
            .trash
            .extend(orphans.map(|(_, b)| (folder.clone(), b)));
        Ok(())
    }

    /// The numbers in blob `blob` read by `decode`; nothing saved, or damaged, starts empty.
    fn number<T: Default>(
        &self,
        blob: &str,
        decode: fn(&[u8]) -> Result<T, Bad>,
        found: &mut Found,
    ) -> Result<T, StoreError> {
        let (folder, blob) = (name(NUMBERS)?, name(blob)?);
        let Some(bytes) = self.load(&folder, &blob, found)? else {
            return Ok(T::default());
        };
        match decode(&bytes) {
            Ok(value) => Ok(value),
            Err(Bad::Newer) => Err(StoreError::Newer),
            Err(Bad::Damaged) => {
                found.damage(&folder, &blob);
                Ok(T::default())
            }
        }
    }

    /// The opened bytes of `blob`, in a wiped buffer; `None` when it is gone or would not open, which is noted.
    fn load(
        &self,
        folder: &BlobName,
        blob: &BlobName,
        found: &mut Found,
    ) -> Result<Option<Zeroizing<Vec<u8>>>, StoreError> {
        match self.shelf.load(folder, blob) {
            Ok(bytes) => Ok(bytes.map(Zeroizing::new)),
            Err(SealedError::Damaged) => {
                found.damage(folder, blob);
                Ok(None)
            }
            Err(error) => Err(error.into()),
        }
    }

    /// Writes the texts that lost their recording, then removes the leftovers; one that fails waits for later.
    fn clean(&mut self, history: &History, found: Found) {
        // A text not written is tried again with the next change.
        let _ = self.apply(history, found.edit);
        for (folder, blob) in &found.trash {
            // A blob not removed is found again at the next open.
            let _ = self.shelf.remove(folder, blob);
        }
    }
}

/// True when `blobs` are exactly blobs 0 to `sound.blobs` of a recording.
fn whole(sound: Sound, blobs: &[(u32, BlobName)]) -> bool {
    let mut numbers: Vec<u32> = blobs.iter().map(|(n, _)| *n).collect();
    numbers.sort_unstable();
    numbers.into_iter().eq(0..sound.blobs)
}
