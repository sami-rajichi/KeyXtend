//! The word table: each learned spelling with its count and last-used day under a stable id, which chains name, and
//! two sorted indexes, by spelling and by folded key.

use kx_module_api::text::{Casing, fold};

use super::clean::{grow, halved, most, oldest, peak};

/// A word's id in its learner; chains name words by it, and a removed word's id goes to the next new word.
pub(super) type Id = u16;

/// One learned word; no `Debug`, so its spelling is never printed.
#[derive(Clone, PartialEq, Eq)]
pub(super) struct Word {
    /// Its spelling.
    pub(super) text: Box<str>,
    /// Its uses.
    pub(super) count: u16,
    /// The day it was last used.
    pub(super) day: u32,
}

/// The learned words; no `Debug`, as they hold spellings.
#[derive(Clone, PartialEq, Eq)]
pub(super) struct Words {
    /// The most words kept after a clean-up.
    cap: usize,
    /// Each id's word; a removed word leaves a hole, and no hole ends the list.
    slots: Vec<Option<Word>>,
    /// The ids, sorted by spelling.
    by_text: Vec<Id>,
    /// The ids, sorted by folded key, then id.
    by_fold: Vec<Id>,
    /// The sum of every count.
    total: u64,
}

impl Words {
    /// An empty table keeping `cap` words after a clean-up.
    pub(super) fn new(cap: usize) -> Self {
        Self {
            cap,
            slots: Vec::new(),
            by_text: Vec::new(),
            by_fold: Vec::new(),
            total: 0,
        }
    }

    /// A table of `words` under their ids, indexed at once; `None` when two share an id or a spelling, or an id
    /// passes both the most a table capped at `cap` holds and the words given, so no slot is held ahead of them.
    pub(super) fn load(cap: usize, casing: &Casing, words: Vec<(Id, Word)>) -> Option<Self> {
        let (count, end) = (
            words.len(),
            words.iter().map(|w| usize::from(w.0) + 1).max(),
        );
        if end.is_some_and(|end| end > most(cap).max(count)) {
            return None;
        }
        let mut slots = Vec::with_capacity(end.unwrap_or_default());
        slots.resize(end.unwrap_or_default(), None);
        for (id, word) in words {
            if slots.get_mut(usize::from(id))?.replace(word).is_some() {
                return None;
            }
        }
        let mut table = Self::new(cap);
        table.slots = slots;
        let mut by_text = Vec::with_capacity(count);
        by_text.extend(table.iter().map(|(id, _)| id));
        by_text.sort_by(|&a, &b| table.text(a).cmp(table.text(b)));
        if by_text
            .windows(2)
            .any(|p| p.first().map(|&a| table.text(a)) == p.last().map(|&b| table.text(b)))
        {
            return None;
        }
        let mut by_fold = by_text.clone();
        by_fold.sort_by_cached_key(|&id| (fold(table.text(id), casing), id));
        (table.by_text, table.by_fold) = (by_text, by_fold);
        table.retotal();
        Some(table)
    }

    /// How many words it holds.
    pub(super) fn len(&self) -> usize {
        self.by_text.len()
    }

    /// The sum of every count.
    pub(super) fn total(&self) -> u64 {
        self.total
    }

    /// Word `id`, when it holds one.
    pub(super) fn get(&self, id: Id) -> Option<&Word> {
        self.slots.get(usize::from(id))?.as_ref()
    }

    /// Every word with its id, in id order.
    pub(super) fn iter(&self) -> impl Iterator<Item = (Id, &Word)> {
        (0..=Id::MAX)
            .zip(&self.slots)
            .filter_map(|(id, slot)| Some((id, slot.as_ref()?)))
    }

    /// The id of the word spelt exactly `text`.
    pub(super) fn find(&self, text: &str) -> Option<Id> {
        let at = self.by_text.binary_search_by(|&id| self.text(id).cmp(text));
        at.ok().and_then(|at| self.by_text.get(at).copied())
    }

    /// The id of the word that differs from `text` in capitals only.
    pub(super) fn variant(&self, casing: &Casing, text: &str) -> Option<Id> {
        let lower = casing.lower(text);
        self.group(casing, text)
            .find(|&id| casing.lower(self.text(id)) == lower)
    }

    /// The ids of the words whose folded key is `text`'s, in id order.
    pub(super) fn group<'w>(
        &'w self,
        casing: &'w Casing,
        text: &str,
    ) -> impl Iterator<Item = Id> + 'w {
        let key = fold(text, casing);
        let from = self
            .by_fold
            .partition_point(|&id| fold(self.text(id), casing) < key);
        let tail = self.by_fold.get(from..).unwrap_or_default();
        let same = move |id: &Id| fold(self.text(*id), casing) == key;
        tail.iter().copied().take_while(same)
    }

    /// Calls `each` with every word whose folded key starts with `prefix`, folded keys in order.
    pub(super) fn starting(&self, casing: &Casing, prefix: &str, each: &mut dyn FnMut(&str)) {
        for &id in self.span(casing, prefix) {
            each(self.text(id));
        }
    }

    /// The ids of the words whose folded key starts with `prefix`, folded keys in order: two binary searches.
    pub(super) fn span(&self, casing: &Casing, prefix: &str) -> &[Id] {
        let key = |id: &Id| fold(self.text(*id), casing);
        let from = (self.by_fold).partition_point(|id| key(id).as_str() < prefix);
        let tail = self.by_fold.get(from..).unwrap_or_default();
        let len = tail.partition_point(|id| key(id).starts_with(prefix));
        tail.get(..len).unwrap_or_default()
    }

    /// Adds `word` under the lowest free id; `None` when every id is taken.
    pub(super) fn insert(&mut self, casing: &Casing, word: Word) -> Option<Id> {
        let free = self.slots.iter().position(Option::is_none);
        let id = Id::try_from(free.unwrap_or(self.slots.len())).ok()?;
        self.put(casing, id, word);
        Some(id)
    }

    /// Places `word` under `id`, which must be free, and indexes it.
    pub(super) fn put(&mut self, casing: &Casing, id: Id, word: Word) {
        let (at, most) = (usize::from(id), most(self.cap));
        if self.slots.len() <= at {
            let more = at + 1 - self.slots.len();
            grow(&mut self.slots, more, most);
            self.slots.resize(at + 1, None);
        }
        let key = fold(&word.text, casing);
        let by_text = (self.by_text).partition_point(|&other| self.text(other) < &*word.text);
        let by_fold = (self.by_fold)
            .partition_point(|&other| (fold(self.text(other), casing), other) < (key.clone(), id));
        grow(&mut self.by_text, 1, most);
        grow(&mut self.by_fold, 1, most);
        self.total += u64::from(word.count);
        self.by_text.insert(by_text, id);
        self.by_fold.insert(by_fold, id);
        if let Some(slot) = self.slots.get_mut(at) {
            *slot = Some(word);
        }
    }

    /// Adds `n` uses to word `id` on `day`; the learner halves every table first when they would pass the top.
    pub(super) fn bump(&mut self, id: Id, n: u16, day: u32) {
        if let Some(word) = self.slots.get_mut(usize::from(id)).and_then(Option::as_mut) {
            let count = word.count.saturating_add(n);
            self.total += u64::from(count - word.count);
            (word.count, word.day) = (count, day);
        }
    }

    /// Halves every count, none below `floor` unless it was already.
    pub(super) fn halve(&mut self, floor: u16) {
        let counts = self.slots.iter_mut().flatten().map(|w| &mut w.count);
        counts.for_each(|count| *count = halved(*count, floor));
        self.retotal();
    }

    /// Spells word `id` as `text`, which folds to the same key, so the fold index stays as it is.
    pub(super) fn respell(&mut self, id: Id, text: &str) {
        if let Ok(at) = self.at_text(id) {
            self.by_text.remove(at);
        }
        if let Some(word) = self.slots.get_mut(usize::from(id)).and_then(Option::as_mut) {
            word.text = text.into();
        }
        let at = self
            .by_text
            .partition_point(|&other| self.text(other) < text);
        self.by_text.insert(at, id);
    }

    /// Removes word `id` and gives it.
    pub(super) fn remove(&mut self, id: Id) -> Option<Word> {
        if let Ok(at) = self.at_text(id) {
            self.by_text.remove(at);
        }
        if let Some(at) = self.by_fold.iter().position(|&other| other == id) {
            self.by_fold.remove(at);
        }
        let word = self.slots.get_mut(usize::from(id))?.take()?;
        self.total -= u64::from(word.count);
        self.trim();
        Some(word)
    }

    /// Removes every word `keep` refuses; gives a flag per id, set for each id removed.
    pub(super) fn retain(&mut self, mut keep: impl FnMut(Id, &Word) -> bool) -> Vec<bool> {
        let mut gone = vec![false; self.slots.len()];
        let slots = (0..=Id::MAX).zip(self.slots.iter_mut());
        for ((id, slot), flag) in slots.zip(&mut gone) {
            if slot.as_ref().is_some_and(|w| !keep(id, w)) {
                *slot = None;
                *flag = true;
            }
        }
        let kept = |id: &Id| !gone.get(usize::from(*id)).copied().unwrap_or_default();
        self.by_text.retain(kept);
        self.by_fold.retain(kept);
        self.trim();
        self.retotal();
        gone
    }

    /// Past its peak, drops the least recently used words down to its cap; gives the flags `retain` gives.
    pub(super) fn clean(&mut self) -> Option<Vec<bool>> {
        if self.len() <= peak(self.cap) {
            return None;
        }
        let entries = self.iter().map(|(id, w)| (w.day, w.count, usize::from(id)));
        let gone = oldest(entries.collect(), self.cap);
        Some(self.retain(|id, _| gone.binary_search(&usize::from(id)).is_err()))
    }

    /// The bytes its lists hold on the heap, allocator overhead aside.
    #[cfg(test)]
    pub(super) fn heap_bytes(&self) -> usize {
        let texts: usize = self.slots.iter().flatten().map(|w| w.text.len()).sum();
        let slots = self.slots.capacity() * size_of::<Option<Word>>();
        let ids = (self.by_text.capacity() + self.by_fold.capacity()) * size_of::<Id>();
        texts + slots + ids
    }

    /// Where `id` sits in the spelling index.
    fn at_text(&self, id: Id) -> Result<usize, usize> {
        let text = self.text(id);
        self.by_text
            .binary_search_by(|&other| self.text(other).cmp(text))
    }

    /// Word `id`'s spelling; empty for a hole.
    fn text(&self, id: Id) -> &str {
        self.get(id).map_or("", |w| &w.text)
    }

    /// Drops the holes that end the list.
    fn trim(&mut self) {
        while self.slots.last().is_some_and(Option::is_none) {
            self.slots.pop();
        }
    }

    /// Sums every count again.
    fn retotal(&mut self) {
        self.total = self
            .slots
            .iter()
            .flatten()
            .map(|w| u64::from(w.count))
            .sum();
    }
}
