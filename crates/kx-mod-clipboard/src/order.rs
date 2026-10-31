//! The history's order (spec §7 "Order"): pinned entries first, then the rest, each newest first. A copy of the same
//! content, an insert or an unpin moves an entry to the top with the new time; the order follows a counter, never the
//! wall clock, so a clock set back keeps it.

use std::cmp::Ordering;

use kx_module_api::ports::LocalTime;

use crate::entry::{Entry, Place, Stamp};

/// Puts `a` before `b` when it is pinned and `b` is not, else when it is newer; ties by id, newer first.
#[must_use]
pub fn newer(a: &Entry, b: &Entry) -> Ordering {
    let pins = b.pinned().cmp(&a.pinned());
    pins.then(b.seq.cmp(&a.seq)).then(b.id.cmp(&a.id))
}

/// The kept entries in order, and the counter that numbers them and their moves.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct History {
    /// The entries, in order.
    entries: Vec<Entry>,
    /// The next number the counter hands out.
    next: u64,
}

impl History {
    /// The history of `entries` as the store loaded them; the counter starts at `next`, or past every number in use.
    #[must_use]
    pub fn new(mut entries: Vec<Entry>, next: u64) -> Self {
        let past = |e: &Entry| e.id.max(e.seq).saturating_add(1);
        let used = entries.iter().map(past).max().unwrap_or_default();
        entries.sort_by(newer);
        Self {
            entries,
            next: next.max(used),
        }
    }

    /// The entries in order.
    #[must_use]
    pub fn entries(&self) -> &[Entry] {
        &self.entries
    }

    /// The entry `id`.
    #[must_use]
    pub fn get(&self, id: u64) -> Option<&Entry> {
        self.entries.iter().find(|e| e.id == id)
    }

    /// How many entries it holds.
    #[must_use]
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// True when it holds none.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// The next number the counter hands out, which the store saves.
    #[must_use]
    pub fn next(&self) -> u64 {
        self.next
    }

    /// Bytes the entries keep.
    #[must_use]
    pub fn bytes(&self) -> u64 {
        self.entries
            .iter()
            .map(|e| e.stored)
            .fold(0, u64::saturating_add)
    }

    /// The entry holding the same content as a copy hashed `hash`: one with that hash for which `same`, comparing the
    /// content, says yes.
    pub fn twin(&self, hash: u64, mut same: impl FnMut(&Entry) -> bool) -> Option<u64> {
        let mut hashed = self.entries.iter().filter(|e| e.hash == hash);
        hashed.find(|e| same(e)).map(|e| e.id)
    }

    /// Numbers `entry` and puts it at the top of the unpinned entries; gives its id.
    pub fn add(&mut self, mut entry: Entry) -> u64 {
        let id = self.number();
        (entry.id, entry.seq) = (id, id);
        self.entries.push(entry);
        self.entries.sort_by(newer);
        id
    }

    /// Moves `id` to the top with the time `now`, as a copy of the same content or an insert does; a pinned entry
    /// stays pinned. Gives where it was.
    pub fn raise(&mut self, id: u64, now: &LocalTime) -> Option<Place> {
        self.renew(id, now, true)
    }

    /// Pins `id`, keeping its time; gives where it was.
    pub fn pin(&mut self, id: u64) -> Option<Place> {
        let place = self.get(id)?.place;
        self.update(id, |e| e.place = Place::Pinned);
        Some(place)
    }

    /// Unpins `id` as if copied at `now`, so the next sweep keeps it; gives where it was.
    pub fn unpin(&mut self, id: u64, now: &LocalTime) -> Option<Place> {
        let place = self.get(id)?.place;
        if place == Place::Pinned {
            self.renew(id, now, false)
        } else {
            Some(place)
        }
    }

    /// Lets `edit` change `id`, such as its blob count or thumbnail, then puts it back in order; false when absent.
    pub fn update(&mut self, id: u64, edit: impl FnOnce(&mut Entry)) -> bool {
        let Some(entry) = self.entries.iter_mut().find(|e| e.id == id) else {
            return false;
        };
        edit(entry);
        self.entries.sort_by(newer);
        true
    }

    /// Takes `id` out.
    pub fn remove(&mut self, id: u64) -> Option<Entry> {
        let at = self.entries.iter().position(|e| e.id == id)?;
        Some(self.entries.remove(at))
    }

    /// Takes out the oldest unpinned entry, which is the last one unless every entry is pinned.
    pub fn pop_oldest(&mut self) -> Option<Entry> {
        let unpinned = self.entries.last().is_some_and(|e| !e.pinned());
        if unpinned { self.entries.pop() } else { None }
    }

    /// Takes out every entry `pick` chooses, in order.
    pub fn take_where(&mut self, pick: impl FnMut(&Entry) -> bool) -> Vec<Entry> {
        let (gone, kept) = std::mem::take(&mut self.entries)
            .into_iter()
            .partition(pick);
        self.entries = kept;
        gone
    }

    /// Gives `id` the time `now` and the newest number, in the folder of `now`'s day unless `keep_pin` and pinned.
    fn renew(&mut self, id: u64, now: &LocalTime, keep_pin: bool) -> Option<Place> {
        let place = self.get(id)?.place;
        let seq = self.number();
        self.update(id, |e| {
            (e.at, e.seq) = (Stamp::of(now), seq);
            if !(keep_pin && e.pinned()) {
                e.place = Place::Day(now.date);
            }
        });
        Some(place)
    }

    /// The counter's next number.
    fn number(&mut self) -> u64 {
        let n = self.next;
        self.next = n.saturating_add(1);
        n
    }
}

#[cfg(test)]
mod tests;
