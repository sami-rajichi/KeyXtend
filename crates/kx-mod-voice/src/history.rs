//! History (design "History, recordings and counts"), pure: the texts dictated, pinned first then newest first, each
//! with its time, language, model and recording, and each change gives the `Edit` the store carries out. A later text
//! of this run's last take replaces it, as Try again does, and an unpin counts as new; pins sit outside
//! `history_max`; recordings share `voice_max_mb`, oldest unpinned first.

mod entry;

pub use entry::{Asked, Edit, Entry, Rules, Sound, Stamp};

use std::cmp::Ordering;
use std::collections::BTreeSet;

use kx_module_api::{Redacted, Secret};

use crate::flow::Kept;

/// Puts pinned texts first, then the newest by the counter, never the wall clock; ties by id, newer first.
fn newer(a: &Entry, b: &Entry) -> Ordering {
    let pins = b.pinned.cmp(&a.pinned);
    pins.then(b.seq.cmp(&a.seq)).then(b.id.cmp(&a.id))
}

/// The texts kept, in order, and the counter that numbers them.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct History {
    entries: Vec<Entry>,
    /// The next number the counter hands out.
    next: u64,
    /// This run's last take and its text, which a later text of that take replaces.
    last: Option<(u32, u64)>,
    /// That text had its recording, so no second one reuses its blob names.
    voiced: bool,
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
            last: None,
            voiced: false,
        }
    }

    /// The texts in order.
    #[must_use]
    pub fn entries(&self) -> &[Entry] {
        &self.entries
    }

    /// The text `id`.
    #[must_use]
    pub fn get(&self, id: u64) -> Option<&Entry> {
        self.entries.iter().find(|e| e.id == id)
    }

    /// How many texts it holds.
    #[must_use]
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// True when it holds none.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// How many texts are unpinned, which `history_max` and Delete all count.
    #[must_use]
    pub fn unpinned(&self) -> usize {
        self.entries.iter().filter(|e| !e.pinned).count()
    }

    /// Bytes all recordings take.
    #[must_use]
    pub fn voice_bytes(&self) -> u64 {
        let sizes = self.entries.iter().filter_map(|e| e.sound.map(|s| s.bytes));
        sizes.fold(0, u64::saturating_add)
    }

    /// The next number the counter hands out.
    #[must_use]
    pub fn next(&self) -> u64 {
        self.next
    }

    /// Keeps `kept`, dictated at `at`: a later text of this run's last take replaces its text in place, else it is
    /// new and the oldest unpinned past `history_max` go. Nothing while texts are not kept.
    pub fn keep(&mut self, kept: Kept, at: Stamp, rules: &Rules) -> Edit {
        let mut edit = Edit::default();
        if !rules.texts {
            return edit;
        }
        let Kept {
            take,
            text,
            lang,
            model,
        } = kept;
        let id = match self.last_of(take).and_then(|id| self.at_mut(id)) {
            Some(entry) => {
                (entry.text, entry.lang, entry.model) = (text, lang, model);
                entry.id
            }
            None => self.add(at, text, lang, model),
        };
        self.last = Some((take, id));
        edit.put.insert(id);
        self.trim_into(rules, Some(id), &mut edit);
        edit
    }

    /// The text a recording of `take` of `bytes` joins: this run's last take, which had none yet, while recordings
    /// are kept and `bytes` fits in the room pinned recordings leave.
    #[must_use]
    pub fn admit(&self, take: u32, bytes: u64, rules: &Rules) -> Option<u64> {
        let id = self.last_of(take)?;
        let pins = self.entries.iter().filter(|e| e.pinned && e.id != id);
        let pinned = pins.filter_map(|e| e.sound.map(|s| s.bytes));
        let room = rules
            .voice_bytes
            .saturating_sub(pinned.fold(0, u64::saturating_add));
        let fits = bytes > 0 && bytes <= room;
        (rules.voice && !self.voiced && fits).then_some(id)
    }

    /// Pairs `sound`, saved, with text `id`, then drops the oldest unpinned recordings past `voice_max_mb`; a text
    /// gone meanwhile has the recording's blobs removed.
    pub fn attach(&mut self, id: u64, sound: Sound, rules: &Rules) -> Edit {
        let mut edit = Edit::default();
        self.voiced |= self.last.is_some_and(|(_, last)| last == id);
        let Some(entry) = self.at_mut(id) else {
            edit.mute.insert(id, sound);
            return edit;
        };
        entry.sound = Some(sound);
        edit.put.insert(id);
        self.trim_voice(rules, Some(id), &mut edit);
        edit
    }

    /// Pins `ids`, keeping their place among the pins by age.
    pub fn pin(&mut self, ids: &[u64]) -> Edit {
        let mut edit = Edit::default();
        let unpinned = self.entries.iter_mut().filter(|e| !e.pinned);
        for entry in unpinned.filter(|e| ids.contains(&e.id)) {
            entry.pinned = true;
            edit.put.insert(entry.id);
        }
        self.entries.sort_by(newer);
        edit
    }

    /// Unpins `ids` as if dictated `now`, as the clipboard's History does: each goes to the top with that time, so the
    /// limit drops it last, and nothing is deleted now.
    pub fn unpin(&mut self, ids: &[u64], now: Stamp) -> Edit {
        let mut edit = Edit::default();
        for &id in ids {
            if !self.get(id).is_some_and(|e| e.pinned) {
                continue;
            }
            let seq = self.number();
            if let Some(entry) = self.at_mut(id) {
                (entry.pinned, entry.seq, entry.at) = (false, seq, now);
                edit.put.insert(id);
            }
        }
        self.entries.sort_by(newer);
        edit
    }

    /// Deletes the ticked `ids`, pinned or not, with their recordings.
    pub fn delete(&mut self, ids: &[u64]) -> Edit {
        self.take_where(|e| ids.contains(&e.id))
    }

    /// The Delete-all question: every unpinned text now; `None` when there is none, so nothing is asked.
    #[must_use]
    pub fn ask_all(&self) -> Option<Asked> {
        let unpinned = self.entries.iter().filter(|e| !e.pinned);
        let ids: BTreeSet<u64> = unpinned.map(|e| e.id).collect();
        (!ids.is_empty()).then_some(Asked(ids))
    }

    /// Yes to the question: deletes the texts it counted that are still unpinned, never one kept or pinned since.
    pub fn delete_all(&mut self, asked: Asked) -> Edit {
        let Asked(ids) = asked;
        self.take_where(|e| !e.pinned && ids.contains(&e.id))
    }

    /// Applies lower limits at once, as after a settings change.
    pub fn trim(&mut self, rules: &Rules) -> Edit {
        let mut edit = Edit::default();
        self.trim_into(rules, None, &mut edit);
        edit
    }

    /// The text of `take` when it is this run's last take and still kept.
    fn last_of(&self, take: u32) -> Option<u64> {
        let (last, id) = self.last?;
        (last == take && self.get(id).is_some()).then_some(id)
    }

    fn at_mut(&mut self, id: u64) -> Option<&mut Entry> {
        self.entries.iter_mut().find(|e| e.id == id)
    }

    /// Adds a new text at the top of the unpinned ones, with no recording yet; gives its id.
    fn add(&mut self, at: Stamp, text: Secret, lang: String, model: Redacted<String>) -> u64 {
        self.voiced = false;
        let id = self.number();
        let (seq, pinned, sound) = (id, false, None);
        let entry = Entry {
            id,
            seq,
            at,
            text,
            lang,
            model,
            pinned,
            sound,
        };
        self.entries.push(entry);
        self.entries.sort_by(newer);
        id
    }

    /// Takes out every text `pick` chooses; gives the edit removing them.
    fn take_where(&mut self, pick: impl Fn(&Entry) -> bool) -> Edit {
        let mut edit = Edit::default();
        let (gone, kept): (Vec<Entry>, Vec<Entry>) = std::mem::take(&mut self.entries)
            .into_iter()
            .partition(pick);
        self.entries = kept;
        for entry in &gone {
            edit.drop_entry(entry);
        }
        edit
    }

    /// Drops the oldest unpinned texts past `max`, never `keep`, then the oldest unpinned recordings past
    /// `voice_bytes`, `keep`'s last.
    fn trim_into(&mut self, rules: &Rules, keep: Option<u64>, edit: &mut Edit) {
        while self.unpinned() > rules.max {
            let oldest = self
                .entries
                .iter()
                .rposition(|e| !e.pinned && Some(e.id) != keep);
            let Some(at) = oldest else {
                break;
            };
            edit.drop_entry(&self.entries.remove(at));
        }
        self.trim_voice(rules, keep, edit);
        self.trim_voice(rules, None, edit);
    }

    /// Drops the oldest unpinned recordings, never `keep`'s, until all fit in `voice_bytes`.
    fn trim_voice(&mut self, rules: &Rules, keep: Option<u64>, edit: &mut Edit) {
        let mut total = self.voice_bytes();
        let unpinned = self
            .entries
            .iter_mut()
            .rev()
            .filter(|e| !e.pinned && Some(e.id) != keep);
        for entry in unpinned {
            if total <= rules.voice_bytes {
                break;
            }
            if let Some(sound) = entry.sound.take() {
                total = total.saturating_sub(sound.bytes);
                edit.put.insert(entry.id);
                edit.mute.insert(entry.id, sound);
            }
        }
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
