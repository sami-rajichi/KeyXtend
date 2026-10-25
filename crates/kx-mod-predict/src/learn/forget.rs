//! Forgetting (spec §6 "Controls: Forget a word"): a word goes with every chain holding it, a seed word stays hidden
//! on the blocklist until typed `min_uses` times, and the last forget can be undone once.

use std::collections::BTreeMap;

use super::chains::Link;
use super::table::{Id, Word};
use super::{Learner, seed_spelling};
use crate::pack::Pack;

/// A forgotten seed word's uses since it was forgotten, and the day of the last.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct Mark {
    /// Its uses since.
    pub(super) uses: u16,
    /// The day it was forgotten or last used.
    pub(super) day: u32,
}

/// What one use of a word does to the blocklist.
pub(super) enum Use {
    /// The word is not on it.
    Free,
    /// The use was counted and the word stays hidden.
    Held,
    /// The word reached `min_uses` and came back with these uses.
    Back(u16),
}

/// The seed words the user forgot, by spelling; no `Debug`, so none is printed.
#[derive(Clone, Default, PartialEq, Eq)]
pub(super) struct Blocked {
    /// Each forgotten spelling's mark.
    marks: BTreeMap<Box<str>, Mark>,
}

impl Blocked {
    /// How many words it holds.
    pub(super) fn len(&self) -> usize {
        self.marks.len()
    }

    /// True when `text` is on it.
    pub(super) fn contains(&self, text: &str) -> bool {
        self.marks.contains_key(text)
    }

    /// Every spelling with its mark, in spelling order.
    pub(super) fn iter(&self) -> impl Iterator<Item = (&str, Mark)> {
        self.marks.iter().map(|(text, mark)| (&**text, *mark))
    }

    /// Counts one use of `text` on `day`; at `min_uses` it leaves the list with its uses.
    pub(super) fn used(&mut self, text: &str, day: u32, min_uses: u32) -> Use {
        let Some(mark) = self.marks.get_mut(text) else {
            return Use::Free;
        };
        mark.uses = mark.uses.saturating_add(1);
        mark.day = day;
        let uses = mark.uses;
        if u32::from(uses) < min_uses {
            return Use::Held;
        }
        self.marks.remove(text);
        Use::Back(uses)
    }

    /// Adds `text` on `day` with no uses; at `cap` words it first drops the least recently used, which it gives.
    fn block(&mut self, text: &str, day: u32, cap: usize) -> Option<(Box<str>, Mark)> {
        let full = self.marks.len() >= cap;
        let oldest = (self.marks.iter().filter(|_| full))
            .min_by_key(|(text, mark)| (mark.day, mark.uses, *text))
            .map(|(text, _)| text.clone());
        let dropped = oldest.and_then(|t| self.marks.remove_entry(&t));
        self.marks.insert(text.into(), Mark { uses: 0, day });
        dropped
    }

    /// Puts `text` back with `mark`.
    pub(super) fn put(&mut self, text: Box<str>, mark: Mark) {
        self.marks.insert(text, mark);
    }
}

/// What the last forget took, to put back; no `Debug`, as it holds a spelling.
#[derive(Clone, Default, PartialEq, Eq)]
pub(super) struct Undo {
    /// The word removed, under its id.
    word: Option<(Id, Word)>,
    /// The chains removed, each with its table's place.
    links: Vec<(usize, Link)>,
    /// The spelling the forget blocked.
    blocked: Option<Box<str>>,
    /// The blocklist entry the forget pushed out.
    dropped: Option<(Box<str>, Mark)>,
}

impl Learner {
    /// Forgets the word a chip shows as `spelling` on `day`, with every chain holding it; a seed word stays hidden
    /// until typed `min_uses` times. False when there was nothing to forget.
    pub fn forget(&mut self, seed: Option<&Pack<'_>>, spelling: &str, day: u32) -> bool {
        let seed = self.own(seed);
        let id = self.lookup(seed, spelling);
        let text: Option<Box<str>> = match id {
            Some(id) => self.words.get(id).map(|w| w.text.clone()),
            None => seed_spelling(seed, spelling).map(Box::from),
        };
        let mut undo = Undo::default();
        if let Some(id) = id {
            undo.word = self.words.remove(id).map(|word| (id, word));
            undo.links = self.unlink(id);
        }
        let seeded = text.filter(|t| seed_spelling(seed, t) == Some(&**t));
        if let Some(text) = seeded.filter(|t| !self.blocked.contains(t)) {
            undo.dropped = self.blocked.block(&text, day, self.cfg.max_words);
            undo.blocked = Some(text);
        }
        if undo == Undo::default() {
            return false;
        }
        self.undo = Some(undo);
        true
    }

    /// Puts back the word the last forget took, exactly as it was, once and only before anything else changes.
    pub fn undo_forget(&mut self) -> bool {
        let Some(undo) = self.undo.take() else {
            return false;
        };
        if let Some(text) = &undo.blocked {
            self.blocked.marks.remove(text);
        }
        if let Some((text, mark)) = undo.dropped {
            self.blocked.put(text, mark);
        }
        if let Some((id, word)) = undo.word {
            self.words.put(self.casing(), id, word);
        }
        for (at, link) in &undo.links {
            if let Some(table) = self.chains.get_mut(*at) {
                table.insert(link);
            }
        }
        true
    }

    /// Removes every chain holding word `id`; gives each with its table's place.
    fn unlink(&mut self, id: Id) -> Vec<(usize, Link)> {
        let mut gone = Vec::new();
        for (at, table) in self.chains.iter_mut().enumerate() {
            let links = table.retain(|_, key, _| !key.contains(&id));
            gone.extend(links.into_iter().map(|link| (at, link)));
        }
        gone
    }
}
