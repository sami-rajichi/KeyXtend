//! The store (design "History, recordings and counts"): `<data>/voice/` on a sealed shelf with its own key
//! (ADR-0010), with one blob per text in `texts`, each kept recording in blobs `<id>-<n>` in `sounds`, and the counts,
//! card checks and Compare's results in `numbers`. A change writes first, then removes, and what fails is tried again
//! with the next change; nothing is logged.

mod codec;
mod load;
mod names;
mod numbers;
mod text;

#[cfg(test)]
pub(crate) mod mem;

use std::fmt;
use std::sync::Arc;

use kx_module_api::ports::{BlobName, SealedError, SealedShelf};
use kx_module_api::{Redacted, SecretBytes, Zeroizing};
use thiserror::Error;

use crate::counts::{Checks, Counts, Scores};
use crate::history::{Edit, Entry, History, Rules, Sound};
use names::{CHECKS, COUNTS, LANGS, NUMBERS, SCORES, SOUNDS, TEXTS, name, sound_name, text_name};

/// Why the store did not open.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Error)]
pub enum StoreError {
    /// A newer version of the app saved a file; nothing was changed, and no store exists to write over it.
    #[error("a voice file was saved by a newer version")]
    Newer,
    /// The shelf failed, such as `Locked` without the OS key.
    #[error(transparent)]
    Shelf(#[from] SealedError),
}

/// What opening the store read.
#[derive(Debug)]
pub struct Opened {
    /// The store, to write each change.
    pub store: Store,
    /// The texts and their recordings.
    pub history: History,
    /// The counts.
    pub counts: Counts,
    /// The card checks.
    pub checks: Checks,
    /// Compare's results.
    pub scores: Scores,
    /// The blobs that would not read and were dropped, by kind, for one note each.
    pub damaged: Damage,
}

/// Blobs that would not read at the open, by kind; a recording's blobs are only listed then, never read.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub struct Damage {
    /// Texts dropped, each with its recording.
    pub texts: usize,
    /// Numbers blobs dropped, each starting empty: counts, card checks or Compare's results.
    pub numbers: usize,
}

/// The voice folder on the sealed shelf; only `open` makes one, so nothing is written before the shelf was read.
pub struct Store {
    shelf: Arc<dyn SealedShelf>,
    /// The most bytes one blob holds.
    blob: usize,
    /// What failed, tried again with the next change.
    pending: Edit,
}

impl Store {
    /// Carries out `edit` over `history`, with what failed before: writes the texts, then removes the blobs gone.
    ///
    /// # Errors
    /// The first shelf error; what failed is tried again with the next change, but a text too large for one blob,
    /// which is never written and stays in memory for this run.
    pub fn apply(&mut self, history: &History, edit: Edit) -> Result<(), SealedError> {
        let mut todo = std::mem::take(&mut self.pending);
        todo.merge(edit);
        let written = self.put_texts(history, &todo);
        written.and(self.remove_gone(&todo))
    }

    /// Keeps the recording handed over for `take`, in blobs, with its text, when the rules and `voice_max_mb` let it;
    /// false when nothing was kept. The oldest unpinned recordings past the limit go.
    ///
    /// # Errors
    /// The shelf's error. A failed blob removes those saved before it, and the text keeps no recording; a failed
    /// text write keeps the recording, and the text waits for the next change.
    pub fn keep_sound(
        &mut self,
        history: &mut History,
        take: u32,
        pieces: &[SecretBytes],
        rules: &Rules,
    ) -> Result<bool, SealedError> {
        let total = pieces
            .iter()
            .map(|p| p.expose().len())
            .fold(0, usize::saturating_add);
        let bytes = u64::try_from(total).unwrap_or(u64::MAX);
        let Some(id) = history.admit(take, bytes, rules) else {
            return Ok(false);
        };
        let mut saved = 0;
        if let Err(error) = self.put_sound(id, pieces, total, &mut saved) {
            // A blob left is removed at the next open, as no text claims it.
            let _ = self.remove_sound(id, saved);
            return Err(error);
        }
        let edit = history.attach(
            id,
            Sound {
                bytes,
                blobs: saved,
            },
            rules,
        );
        self.apply(history, edit)?;
        Ok(true)
    }

    /// The recording of `entry` in its blobs, or `None` when it has none or a blob is gone.
    ///
    /// # Errors
    /// `Damaged` when the blobs do not add up to its size, else the shelf's error.
    pub fn sound(&self, entry: &Entry) -> Result<Option<Vec<SecretBytes>>, SealedError> {
        let Some(sound) = entry.sound else {
            return Ok(None);
        };
        let folder = name(SOUNDS)?;
        let (mut blobs, mut total) = (Vec::new(), 0_u64);
        for n in 0..sound.blobs {
            let Some(bytes) = self.shelf.load(&folder, &sound_name(entry.id, n)?)? else {
                return Ok(None);
            };
            total = total.saturating_add(u64::try_from(bytes.len()).unwrap_or(u64::MAX));
            blobs.push(Redacted::new(Zeroizing::new(bytes)));
        }
        if total == sound.bytes {
            Ok(Some(blobs))
        } else {
            Err(SealedError::Damaged)
        }
    }

    /// Saves `counts`, first dropping their oldest days until they fit in one blob.
    ///
    /// # Errors
    /// The shelf's error.
    pub fn save_counts(&self, counts: &mut Counts) -> Result<(), SealedError> {
        let mut bytes = numbers::encode_counts(counts);
        while bytes.len() > self.blob && counts.drop_oldest() {
            bytes = numbers::encode_counts(counts);
        }
        self.save_number(COUNTS, &bytes)
    }

    /// Saves the card checks in the form older builds read, then their languages beside them (ADR-0052).
    ///
    /// # Errors
    /// `Io` past one blob, else the shelf's error.
    pub fn save_checks(&self, checks: &Checks) -> Result<(), SealedError> {
        self.save_number(CHECKS, &numbers::encode_checks(checks))?;
        self.save_number(LANGS, &numbers::encode_langs(checks))
    }

    /// Saves Compare's results.
    ///
    /// # Errors
    /// `Io` past one blob, else the shelf's error.
    pub fn save_scores(&self, scores: &Scores) -> Result<(), SealedError> {
        self.save_number(SCORES, &numbers::encode_scores(scores))
    }

    fn save_number(&self, blob: &str, bytes: &[u8]) -> Result<(), SealedError> {
        if bytes.len() > self.blob {
            return Err(SealedError::Io);
        }
        self.shelf.save(&name(NUMBERS)?, &name(blob)?, bytes)
    }

    /// Writes each text `todo` puts that `history` still holds; one that fails waits in `pending`.
    fn put_texts(&mut self, history: &History, todo: &Edit) -> Result<(), SealedError> {
        let folder = name(TEXTS)?;
        let mut first = Ok(());
        for entry in todo.put.iter().filter_map(|id| history.get(*id)) {
            let bytes = text::encode(entry);
            if bytes.len() > self.blob {
                first = first.and(Err(SealedError::Io));
                continue;
            }
            let saved = text_name(entry.id).and_then(|n| self.shelf.save(&folder, &n, &bytes));
            if saved.is_err() {
                self.pending.put.insert(entry.id);
            }
            first = first.and(saved);
        }
        first
    }

    /// Removes the texts and recordings `todo` drops; one that fails waits in `pending`.
    fn remove_gone(&mut self, todo: &Edit) -> Result<(), SealedError> {
        let folder = name(TEXTS)?;
        let mut first = Ok(());
        for (&id, &sound) in &todo.gone {
            let text = text_name(id).and_then(|n| self.shelf.remove(&folder, &n));
            let gone = text.and_then(|()| self.remove_sound(id, sound.map_or(0, |s| s.blobs)));
            if gone.is_err() {
                self.pending.gone.insert(id, sound);
            }
            first = first.and(gone);
        }
        for (&id, &sound) in &todo.mute {
            let gone = self.remove_sound(id, sound.blobs);
            if gone.is_err() {
                self.pending.mute.insert(id, sound);
            }
            first = first.and(gone);
        }
        first
    }

    /// Removes blobs 0 to `blobs` of text `id`'s recording.
    fn remove_sound(&self, id: u64, blobs: u32) -> Result<(), SealedError> {
        let folder = name(SOUNDS)?;
        let mut first = Ok(());
        for n in 0..blobs {
            first = first.and(sound_name(id, n).and_then(|b| self.shelf.remove(&folder, &b)));
        }
        first
    }

    /// Regroups `pieces`, `total` bytes, into blobs of `blob` bytes saved in order, counting each in `saved`.
    fn put_sound(
        &self,
        id: u64,
        pieces: &[SecretBytes],
        total: usize,
        saved: &mut u32,
    ) -> Result<(), SealedError> {
        let folder = name(SOUNDS)?;
        let mut left = total;
        let mut buf = self.buffer(left);
        for piece in pieces {
            let mut rest = piece.expose().as_slice();
            while !rest.is_empty() {
                let room = self.blob.saturating_sub(buf.len());
                let (head, tail) = rest.split_at(rest.len().min(room));
                buf.extend_from_slice(head);
                rest = tail;
                if buf.len() == self.blob {
                    left = left.saturating_sub(buf.len());
                    self.save_blob(&folder, id, &buf, saved)?;
                    buf = self.buffer(left);
                }
            }
        }
        if buf.is_empty() {
            return Ok(());
        }
        self.save_blob(&folder, id, &buf, saved)
    }

    /// A wiped buffer for the next blob of `left` bytes still to save, sized so it never grows.
    fn buffer(&self, left: usize) -> Zeroizing<Vec<u8>> {
        Zeroizing::new(Vec::with_capacity(left.min(self.blob)))
    }

    /// Saves `bytes` as blob `saved` of text `id`'s recording, then counts it.
    fn save_blob(
        &self,
        folder: &BlobName,
        id: u64,
        bytes: &[u8],
        saved: &mut u32,
    ) -> Result<(), SealedError> {
        self.shelf.save(folder, &sound_name(id, *saved)?, bytes)?;
        *saved = saved.checked_add(1).ok_or(SealedError::Io)?;
        Ok(())
    }
}

/// Counts what waits, never content.
impl fmt::Debug for Store {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let pending = &self.pending;
        f.debug_struct("Store")
            .field("blob", &self.blob)
            .field("put", &pending.put.len())
            .field("gone", &pending.gone.len())
            .field("mute", &pending.mute.len())
            .finish_non_exhaustive()
    }
}

#[cfg(test)]
mod tests;
