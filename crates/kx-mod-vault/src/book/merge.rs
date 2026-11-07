//! Bring back into a notebook that has entries (design "Bring back into a notebook that has entries"): entries match by
//! id and the newer change wins whole, keeping the later last use; an entry on one side only is kept. A merge past
//! `max_entries` or the saved size brings nothing in, so no entry is lost from the box it came from.

use std::collections::HashMap;

use super::{Limits, Notebook};
use crate::entry::{Entry, Stamp};
use crate::store::entry_len;

impl Notebook {
    /// Brings `theirs` in whole under `limits`, each by its id, and gives how many came in, new or newer; `None` leaves
    /// the notebook as it was, as they would make it pass a limit it was within.
    #[must_use]
    pub fn merge(&mut self, theirs: Vec<Entry>, limits: &Limits) -> Option<u32> {
        let mut next = Next::of(self);
        for entry in theirs {
            next.take(entry);
        }
        let room = next.entries.len() <= limits.max_entries.max(self.entries.len());
        if !room || next.bytes > limits.max_bytes.max(self.bytes()) {
            return None;
        }
        self.entries = next.entries;
        Some(next.added)
    }
}

/// The notebook a merge is making, its byte total kept as it goes and each entry's place known by its id.
struct Next {
    entries: Vec<Entry>,
    at: HashMap<u64, usize>,
    bytes: usize,
    added: u32,
}

impl Next {
    /// The merge's start: a copy of `book`.
    fn of(book: &Notebook) -> Self {
        let places = book.entries.iter().enumerate();
        Self {
            entries: book.entries.clone(),
            at: places.map(|(i, e)| (e.id(), i)).collect(),
            bytes: book.bytes(),
            added: 0,
        }
    }

    /// Takes one entry: in place of an older copy, or as a new one; a copy no newer than ours is dropped.
    fn take(&mut self, mut entry: Entry) {
        let Some(&i) = self.at.get(&entry.id()) else {
            self.at.insert(entry.id(), self.entries.len());
            self.bytes = self.bytes.saturating_add(entry_len(&entry));
            self.entries.push(entry);
            self.added = self.added.saturating_add(1);
            return;
        };
        let ours = &self.entries[i];
        if entry.changed() <= ours.changed() {
            return;
        }
        if let Some(used) = later(ours.used(), entry.used()) {
            entry.use_at(used);
        }
        let kept = self.bytes.saturating_sub(entry_len(ours));
        self.bytes = kept.saturating_add(entry_len(&entry));
        self.entries[i] = entry;
        self.added = self.added.saturating_add(1);
    }
}

/// The later of two last uses.
fn later(a: Option<Stamp>, b: Option<Stamp>) -> Option<Stamp> {
    a.max(b)
}

#[cfg(test)]
mod tests;
