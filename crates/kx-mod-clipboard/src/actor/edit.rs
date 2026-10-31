//! Changes to kept entries: every removal goes through the store's delete, moves follow the history's pins, unpins
//! and raises, and a save on the debounce then deletes what it found gone and what vanished by hand.

use kx_module_api::keys;
use kx_module_api::ports::SealedError;

use super::Actor;
use crate::entry::{Entry, Place};

impl Actor {
    /// Deletes entry `id` for good: its row, then its blobs through the store.
    pub(super) fn remove(&mut self, id: u64) {
        if let Some(entry) = self.history.remove(id) {
            self.forget(&entry);
        }
    }

    /// Removes the blobs of `entry`, already out of the history; a blob left behind goes at the next load.
    pub(super) fn forget(&mut self, entry: &Entry) {
        if let Err(error) = self.store.delete(entry) {
            tracing::warn!(%error, "a clipboard entry's blobs were not all removed");
        }
        self.asked.remove(&entry.id);
        self.changed();
    }

    /// Follows the move of `id` from `from` on the shelf; content found gone deletes the entry, a failure waits.
    pub(super) fn moved(&mut self, id: u64, from: Place) {
        match self.store.moved(&self.history, id, from) {
            Ok(true) => {}
            Ok(false) => self.remove(id),
            Err(error) => tracing::warn!(%error, "a clipboard move waits for the next save"),
        }
        self.take_gone();
        self.changed();
    }

    /// Pins `ids`, moving their blobs to the pinned folder.
    pub(super) fn pin(&mut self, ids: &[u64]) {
        for &id in ids {
            if let Some(from) = self.history.pin(id) {
                self.moved(id, from);
            }
        }
    }

    /// Unpins `ids` as if copied now, moving their blobs to today's folder.
    pub(super) fn unpin(&mut self, ids: &[u64]) {
        let now = self.ports.calendar.now();
        for &id in ids {
            if let Some(from) = self.history.unpin(id, &now) {
                self.moved(id, from);
            }
        }
    }

    /// Saves now, then deletes what the save found gone and what vanished by hand; a failed save tries again after
    /// `save_ms`, logged once until one lands.
    pub(super) fn save(&mut self) {
        self.dirty = None;
        match self.store.save(&self.history) {
            Ok(()) => self.failing = false,
            Err(error) => {
                if !std::mem::replace(&mut self.failing, true) {
                    tracing::warn!(%error, "the clipboard history was not saved");
                }
                if error == SealedError::Locked {
                    self.say_once(keys::CLIP_LOCKED);
                }
                self.dirty = Some(self.ports.clock.now());
            }
        }
        self.take_gone();
        self.drop_missing();
    }

    /// Deletes the entries whose content went by hand while running, as a day folder deleted in Explorer.
    pub(super) fn drop_missing(&mut self) {
        match self.store.missing(&self.history) {
            Ok(ids) => ids.into_iter().for_each(|id| self.remove(id)),
            Err(error) => tracing::warn!(%error, "the clipboard folders could not be listed"),
        }
    }

    /// Notes a change: it saves after `save_ms`, the room is read again, and the open panel draws again.
    pub(super) fn changed(&mut self) {
        self.dirty.get_or_insert(self.ports.clock.now());
        self.room_stale = true;
        self.redraw = true;
    }

    /// Deletes the entries the store found gone; one deleted meanwhile is skipped.
    fn take_gone(&mut self) {
        for id in self.store.gone() {
            self.remove(id);
        }
    }
}
