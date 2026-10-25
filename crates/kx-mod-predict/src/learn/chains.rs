//! One chain length's table: keys of word ids, the previous words then the word, sorted, each with its count and
//! last-used day. The keys sit in one flat list, so a full table holds no per-entry overhead.

use std::cmp::Ordering;

use super::clean::{grow, halved, most, oldest, peak};
use super::table::Id;

/// One chain taken out of a table, to put back on an undo.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct Link {
    /// Its word ids, the previous words then the word.
    pub(super) key: Vec<Id>,
    /// Its uses.
    pub(super) count: u16,
    /// The day it was last used.
    pub(super) day: u32,
}

/// The chains of one length.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct Chains {
    /// The ids per key: the chain's length.
    width: usize,
    /// The most chains kept after a clean-up.
    cap: usize,
    /// Every key, `width` ids each, in order.
    keys: Vec<Id>,
    /// Each chain's uses.
    counts: Vec<u16>,
    /// The day each chain was last used.
    days: Vec<u32>,
}

impl Chains {
    /// An empty table of chains `width` words long, keeping `cap` after a clean-up.
    pub(super) fn new(width: usize, cap: usize) -> Self {
        Self {
            width,
            cap,
            keys: Vec::new(),
            counts: Vec::new(),
            days: Vec::new(),
        }
    }

    /// How many chains it holds.
    pub(super) fn len(&self) -> usize {
        self.counts.len()
    }

    /// The uses of the chain `key`; 0 when it holds none.
    pub(super) fn count(&self, key: &[Id]) -> u16 {
        self.find(key)
            .ok()
            .and_then(|at| self.counts.get(at).copied())
            .unwrap_or_default()
    }

    /// Adds one use of the chain `key` on `day`; the learner halves every table first when it would pass the top.
    pub(super) fn bump(&mut self, key: &[Id], day: u32) {
        match self.find(key) {
            Ok(at) => self.add(at, 1, day),
            Err(at) => self.place(at, key, 1, day),
        }
    }

    /// Halves every count, none below `floor` unless it was already.
    pub(super) fn halve(&mut self, floor: u16) {
        self.counts.iter_mut().for_each(|c| *c = halved(*c, floor));
    }

    /// Names word `into` wherever a key names word `from`; keys that then meet add their counts and keep the later
    /// day.
    pub(super) fn rekey(&mut self, from: Id, into: Id) {
        for mut link in self.retain(|_, key, _| !key.contains(&from)) {
            link.key
                .iter_mut()
                .filter(|id| **id == from)
                .for_each(|id| *id = into);
            match self.find(&link.key) {
                Ok(at) => self.add(at, link.count, link.day.max(self.entry(at).2)),
                Err(at) => self.place(at, &link.key, link.count, link.day),
            }
        }
    }

    /// Appends the chain `key` with `count` and `day` when it is `width` long and sorts after every chain held.
    pub(super) fn push(&mut self, key: &[Id], count: u16, day: u32) -> bool {
        let last = self.len().checked_sub(1).map(|at| self.key(at));
        let after = last.is_none_or(|last| last < key);
        if key.len() != self.width || !after {
            return false;
        }
        self.place(self.len(), key, count, day);
        true
    }

    /// Puts `link` back in its place; a chain it holds already is left as it is.
    pub(super) fn insert(&mut self, link: &Link) {
        if let Err(at) = self.find(&link.key) {
            self.place(at, &link.key, link.count, link.day);
        }
    }

    /// The last ids of the chains that start with `context`, in key order.
    pub(super) fn followers<'s>(&'s self, context: &'s [Id]) -> impl Iterator<Item = Id> + 's {
        let n = context.len();
        let from = self.lower(|key| head(key, n).cmp(context));
        (from..self.len())
            .map(|at| self.key(at))
            .take_while(move |key| head(key, n) == context)
            .filter_map(|key| key.last().copied())
    }

    /// Entry `at`: its key, count and last-used day.
    pub(super) fn entry(&self, at: usize) -> (&[Id], u16, u32) {
        let count = self.counts.get(at).copied().unwrap_or_default();
        (
            self.key(at),
            count,
            self.days.get(at).copied().unwrap_or_default(),
        )
    }

    /// Removes every chain `keep` refuses, given its place, key and day; gives the chains removed.
    pub(super) fn retain(&mut self, mut keep: impl FnMut(usize, &[Id], u32) -> bool) -> Vec<Link> {
        let old = std::mem::replace(self, Self::new(self.width, self.cap));
        let mut gone = Vec::new();
        for at in 0..old.len() {
            let (key, count, day) = old.entry(at);
            if keep(at, key, day) {
                self.push(key, count, day);
            } else {
                let key = key.to_vec();
                gone.push(Link { key, count, day });
            }
        }
        gone
    }

    /// Past its peak, drops the least recently used chains down to its cap.
    pub(super) fn clean(&mut self) {
        if self.len() <= peak(self.cap) {
            return;
        }
        let entries = (0..self.len()).map(|at| {
            let (_, count, day) = self.entry(at);
            (day, count, at)
        });
        let gone = oldest(entries.collect(), self.cap);
        self.retain(|at, _, _| gone.binary_search(&at).is_err());
    }

    /// The bytes its lists hold on the heap, allocator overhead aside.
    #[cfg(test)]
    pub(super) fn heap_bytes(&self) -> usize {
        let keys = self.keys.capacity() * size_of::<Id>();
        keys + self.counts.capacity() * size_of::<u16>() + self.days.capacity() * size_of::<u32>()
    }

    /// Entry `at`'s key; empty past the end.
    fn key(&self, at: usize) -> &[Id] {
        let start = at.saturating_mul(self.width);
        self.keys
            .get(start..start.saturating_add(self.width))
            .unwrap_or_default()
    }

    /// Where `key` is, or where it would go.
    fn find(&self, key: &[Id]) -> Result<usize, usize> {
        let at = self.lower(|other| other.cmp(key));
        if at < self.len() && self.key(at) == key {
            Ok(at)
        } else {
            Err(at)
        }
    }

    /// The first entry whose key `order` does not put below the target.
    fn lower(&self, order: impl Fn(&[Id]) -> Ordering) -> usize {
        let (mut lo, mut hi) = (0, self.len());
        while lo < hi {
            let mid = lo + (hi - lo) / 2;
            if order(self.key(mid)) == Ordering::Less {
                lo = mid + 1;
            } else {
                hi = mid;
            }
        }
        lo
    }

    /// Adds `n` uses to entry `at`, last used on `day`.
    fn add(&mut self, at: usize, n: u16, day: u32) {
        if let (Some(count), Some(last)) = (self.counts.get_mut(at), self.days.get_mut(at)) {
            (*count, *last) = (count.saturating_add(n), day);
        }
    }

    /// Inserts the chain `key` at entry `at` with `count` and `day`, growing no further than its peak needs.
    fn place(&mut self, at: usize, key: &[Id], count: u16, day: u32) {
        let most = most(self.cap);
        grow(&mut self.keys, self.width, most.saturating_mul(self.width));
        grow(&mut self.counts, 1, most);
        grow(&mut self.days, 1, most);
        let start = at.saturating_mul(self.width).min(self.keys.len());
        self.keys.splice(start..start, key.iter().copied());
        self.counts.insert(at, count);
        self.days.insert(at, day);
    }
}

/// The first `n` ids of `key`; empty when it is shorter.
fn head(key: &[Id], n: usize) -> &[Id] {
    key.get(..n).unwrap_or_default()
}
