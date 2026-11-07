//! The notebook (design "The notebook"): its entries, each with a stable random id, within `max_entries` and a saved
//! size that fits one sealed blob. Limits govern what is added or edited; a notebook saved under higher ones loads whole.

use kx_module_api::Secret;
use kx_module_api::ports::{Label, Random};

use crate::entry::{Draft, Entry, EntryError, Stamp};
use crate::kinds::tables;
use crate::store::{HEAD_LEN, entry_len};

mod many;
mod merge;

/// Draws of a new id before the source counts as stuck: eight collisions among 10,000 ids of 2⁶⁴ never happen.
const ID_TRIES: usize = 8;

/// The notebook's limits; the module fills them from `[vault]`.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Limits {
    /// The most characters one value, name or own label holds (`field_chars`).
    pub field_chars: usize,
    /// The most entries (`max_entries`).
    pub max_entries: usize,
    /// The most fields of one entry (`max_fields`).
    pub max_fields: usize,
    /// The most bytes the saved notebook takes; the module passes the sealed store's `MAX_BLOB_BYTES`.
    pub max_bytes: usize,
}

/// The entries, in the order they were saved; `order` sorts them for the list.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Notebook {
    entries: Vec<Entry>,
}

impl Notebook {
    /// The notebook of `entries`, as the store read them.
    #[must_use]
    pub fn new(entries: Vec<Entry>) -> Self {
        Self { entries }
    }

    /// Every entry.
    #[must_use]
    pub fn entries(&self) -> &[Entry] {
        &self.entries
    }

    /// Entry `id`.
    #[must_use]
    pub fn get(&self, id: u64) -> Option<&Entry> {
        self.entries.iter().find(|e| e.id() == id)
    }

    /// How many entries it holds.
    #[must_use]
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// True when it holds none, so the list invites a first one.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// The bytes the saved notebook takes.
    #[must_use]
    pub fn bytes(&self) -> usize {
        self.entries
            .iter()
            .map(entry_len)
            .fold(HEAD_LEN, usize::saturating_add)
    }

    /// Saves `draft` as a new entry changed at `now`, with an id from `random` that no entry holds; gives the id.
    ///
    /// # Errors
    /// `Full` at `max_entries` or past `max_bytes`, `NoRandom` without random bytes, or the entry's own refusal.
    pub fn add(
        &mut self,
        draft: Draft,
        now: Stamp,
        random: &dyn Random,
        limits: &Limits,
    ) -> Result<u64, EntryError> {
        if self.entries.len() >= limits.max_entries {
            return Err(EntryError::Full);
        }
        let id = self.fresh_id(random)?;
        let entry = Entry::new(id, draft, self.stamp(now), limits)?;
        self.fits(&entry, None, limits)?;
        self.entries.push(entry);
        Ok(id)
    }

    /// Puts `draft` in place of entry `id`, changed at `now`, keeping its id and last use; false when it is gone.
    ///
    /// # Errors
    /// `Full` past `max_bytes`, or the entry's own refusal; the entry stays as it was.
    pub fn edit(
        &mut self,
        id: u64,
        draft: Draft,
        now: Stamp,
        limits: &Limits,
    ) -> Result<bool, EntryError> {
        let Some(at) = self.entries.iter().position(|e| e.id() == id) else {
            return Ok(false);
        };
        let mut entry = Entry::new(id, draft, self.stamp(now), limits)?;
        if let Some(used) = self.entries[at].used() {
            entry.use_at(used);
        }
        self.fits(&entry, Some(at), limits)?;
        self.entries[at] = entry;
        Ok(true)
    }

    /// Takes entry `id` out.
    pub fn remove(&mut self, id: u64) -> Option<Entry> {
        let at = self.entries.iter().position(|e| e.id() == id)?;
        Some(self.entries.remove(at))
    }

    /// Notes that entry `id` was filled at `now`; false when it is gone.
    pub fn use_at(&mut self, id: u64, now: Stamp) -> bool {
        let now = self.stamp(now);
        let entry = self.entries.iter_mut().find(|e| e.id() == id);
        entry.map(|e| e.use_at(now)).is_some()
    }

    /// The shown values under a label that feeds the `@@` list, in the entries' order; a hidden one stays out.
    #[must_use]
    pub fn emails(&self) -> Vec<Secret> {
        let feeds = |label: &Label| match label {
            Label::Table(id) => tables().label(id).is_some_and(|l| l.emails),
            Label::Own(_) => false,
        };
        let fields = self.entries.iter().flat_map(Entry::fields);
        let emails = fields.filter(|f| !f.hidden && feeds(&f.label));
        emails.map(|f| f.value.clone()).collect()
    }

    /// `now`, or just after the newest time the notebook holds when the clock stepped back, so each change reads as
    /// newer, as Bring back's "newer copy wins" needs.
    fn stamp(&self, now: Stamp) -> Stamp {
        let times = self
            .entries
            .iter()
            .flat_map(|e| [Some(e.changed()), e.used()]);
        times
            .flatten()
            .max()
            .map_or(now, |newest| now.max(newest.after()))
    }

    /// An id no entry holds.
    fn fresh_id(&self, random: &dyn Random) -> Result<u64, EntryError> {
        draw_id(random, |id| self.get(id).is_some())
    }

    /// Refuses `entry` when the notebook with it, in place of the entry at `instead`, passes `max_bytes`.
    fn fits(
        &self,
        entry: &Entry,
        instead: Option<usize>,
        limits: &Limits,
    ) -> Result<(), EntryError> {
        let gone = instead
            .and_then(|at| self.entries.get(at))
            .map_or(0, entry_len);
        let after = self
            .bytes()
            .saturating_sub(gone)
            .saturating_add(entry_len(entry));
        if after > limits.max_bytes {
            Err(EntryError::Full)
        } else {
            Ok(())
        }
    }
}

/// An id from `random` that is not `taken`.
fn draw_id(random: &dyn Random, taken: impl Fn(u64) -> bool) -> Result<u64, EntryError> {
    for _ in 0..ID_TRIES {
        let mut bytes = [0; size_of::<u64>()];
        random.fill(&mut bytes)?;
        let id = u64::from_le_bytes(bytes);
        if !taken(id) {
            return Ok(id);
        }
    }
    Err(EntryError::NoRandom)
}

#[cfg(test)]
mod tests;
