//! The texts that left the history, as content hashes alone, takes apart from deletes so a bulk delete never pushes a
//! take out: Paste at once never writes one back until the history keeps it again (owner's check, 2026-11-21). Every
//! note of a take's copy told within `take_wait_ms` keeps nothing, however many come and from any app; a rich entry's
//! hash never matches, as only the board names it, but the history cannot tell it without its blobs.

use std::collections::VecDeque;
use std::time::Duration;

use kx_module_api::Mono;
use kx_module_api::ports::{ClipKind, ClipPart};

use crate::entry::{Entry, text_of};
use crate::hash;
use crate::settings::ClipSettings;

/// The hashes of the texts takes read and of those deleted, swept or trimmed, and of the takes' copies still due, each
/// with the end of its wait; at most `due_max` are due.
pub(super) struct Lost {
    taken: Ring,
    gone: Ring,
    due: VecDeque<(u64, Mono)>,
    due_max: usize,
    wait: Duration,
}

impl Lost {
    /// None yet; at most `max_entries` texts of each kind, and `queue_copies` takes due, as no more copies wait.
    pub(super) fn new(s: &ClipSettings) -> Self {
        Self {
            taken: Ring::new(s.max_entries),
            gone: Ring::new(s.max_entries),
            due: VecDeque::new(),
            due_max: s.queue_copies,
            wait: s.take_wait(),
        }
    }

    /// `entry` left the history; a text entry's hash is kept, matched only when that text was all it held.
    pub(super) fn gone(&mut self, entry: &Entry) {
        if matches!(entry.kind, ClipKind::Text | ClipKind::Link) {
            self.gone.add(entry.hash);
        }
    }

    /// A take read `text` at `now`, so it never goes back, and its copy is due until the wait ends, one wait per text;
    /// past the most due, the one that ends first is due no more.
    pub(super) fn taken(&mut self, text: &str, now: Mono) {
        let hash = hash::of_text(text);
        self.taken.add(hash);
        self.due.retain(|(h, _)| *h != hash);
        if self.due.len() >= self.due_max {
            self.due.pop_front();
        }
        self.due.push_back((hash, now.saturating_add(self.wait)));
    }

    /// The watch told a copy of `parts` at `now`: true when one of its texts is a take's copy still due, which stays
    /// due until its wait ends. With none due, no text is hashed.
    pub(super) fn noted(&mut self, parts: &[ClipPart], now: Mono) -> bool {
        self.due.retain(|(_, end)| now < *end);
        if self.due.is_empty() {
            return false;
        }
        let mut texts = parts.iter().filter_map(|p| match p {
            ClipPart::Text(t) => Some(hash::of_text(t.expose())),
            _ => None,
        });
        texts.any(|h| self.due.iter().any(|(d, _)| *d == h))
    }

    /// The history kept `parts` again, so their text may go back.
    pub(super) fn kept(&mut self, parts: &[ClipPart]) {
        if let Some(text) = text_of(parts) {
            let hash = hash::of_text(text);
            self.taken.remove(hash);
            self.gone.remove(hash);
        }
    }

    /// True when `text` left the history and was not kept since.
    pub(super) fn has(&self, text: &str) -> bool {
        let hash = hash::of_text(text);
        self.taken.has(hash) || self.gone.has(hash)
    }
}

/// Hashes, oldest first, at most `max`.
struct Ring {
    hashes: VecDeque<u64>,
    max: usize,
}

impl Ring {
    /// None yet, holding at most `max`.
    fn new(max: usize) -> Self {
        let hashes = VecDeque::new();
        Self { hashes, max }
    }

    /// Keeps `hash` as the newest, moving it there if held; past the most, the oldest goes.
    fn add(&mut self, hash: u64) {
        self.remove(hash);
        if self.hashes.len() >= self.max {
            self.hashes.pop_front();
        }
        self.hashes.push_back(hash);
    }

    /// Lets `hash` go.
    fn remove(&mut self, hash: u64) {
        self.hashes.retain(|h| *h != hash);
    }

    /// True when it holds `hash`.
    fn has(&self, hash: u64) -> bool {
        self.hashes.contains(&hash)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fixture::{at, copied, date, text};

    /// A text entry copied now.
    fn entry(s: &str) -> Entry {
        copied(s, &at(date(2026, 11, 22), 9, 0))
    }

    /// A record holding at most `max` texts and `due` takes due, on the default wait.
    fn lost(max: usize, due: usize) -> Lost {
        let mut s = ClipSettings::from_defaults().unwrap();
        (s.max_entries, s.queue_copies) = (max, due);
        Lost::new(&s)
    }

    #[test]
    fn a_text_stays_lost_until_kept_and_past_the_most_the_oldest_of_its_kind_goes() {
        let (mut lost, now) = (lost(2, 1), Mono::default());
        lost.taken("a", now);
        lost.gone(&entry("b"));
        assert!(lost.has("a") && lost.has("b"));
        lost.taken("c", now);
        lost.taken("d", now);
        assert!(!lost.has("a"), "a went first among the takes");
        assert!(lost.has("b") && lost.has("c"));
        lost.kept(&[text("b")]);
        assert!(!lost.has("b") && lost.has("c"));
    }

    #[test]
    fn a_bulk_delete_never_pushes_out_a_take() {
        let (mut lost, now) = (lost(2, 1), Mono::default());
        lost.taken("k", now);
        for t in ["b", "c", "d"] {
            lost.gone(&entry(t));
        }
        assert!(lost.has("k") && !lost.has("b") && lost.has("d"));
    }

    #[test]
    fn a_hash_added_again_moves_to_the_back() {
        let (mut lost, now) = (lost(2, 1), Mono::default());
        for t in ["a", "b", "a", "c"] {
            lost.taken(t, now);
        }
        assert!(lost.has("a") && !lost.has("b"), "a take moved back");
        for t in ["x", "y", "x", "z"] {
            lost.gone(&entry(t));
        }
        assert!(lost.has("x") && !lost.has("y"), "a delete moved back");
    }

    #[test]
    fn a_picture_entry_is_never_held() {
        let mut lost = lost(2, 1);
        let mut picture = entry("d");
        picture.kind = ClipKind::Image;
        lost.gone(&picture);
        assert!(!lost.has("d"));
    }

    #[test]
    fn a_takes_copy_is_noted_until_its_wait_ends_and_past_the_most_due_the_oldest_goes() {
        let mut lost = lost(4, 2);
        let (now, wait) = (Mono::default(), lost.wait);
        for t in ["a", "b", "c", "c"] {
            lost.taken(t, now);
        }
        assert!(!lost.noted(&[text("a")], now), "a went first");
        assert!(
            lost.noted(&[text("b")], now),
            "a take again holds one place"
        );
        assert!(lost.noted(&[text("b")], now), "noted until its wait ends");
        assert!(
            !lost.noted(&[text("c")], now.saturating_add(wait)),
            "its wait ended"
        );
        assert!(
            lost.has("a") && lost.has("b") && lost.has("c"),
            "still lost"
        );
    }
}
