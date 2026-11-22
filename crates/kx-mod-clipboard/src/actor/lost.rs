//! The texts that left the history, by a take, a delete or a sweep, as content hashes alone: Paste at once never writes
//! one back until the history keeps it again (owner's check, 2026-11-21). A rich entry's text is not matched, as only
//! the board can name it; past `max_entries` the oldest goes.

use std::collections::VecDeque;

use kx_module_api::ports::{ClipKind, ClipPart};

use crate::entry::{Entry, text_of};
use crate::hash;

/// The hashes of the texts that left the history, oldest first, and how many it holds at most.
pub(super) struct Lost {
    hashes: VecDeque<u64>,
    max: usize,
}

impl Lost {
    /// None yet, holding at most `max`.
    pub(super) fn new(max: usize) -> Self {
        let hashes = VecDeque::new();
        Self { hashes, max }
    }

    /// `entry` left the history; a text entry's hash is kept, matched only when that text was all it held.
    pub(super) fn gone(&mut self, entry: &Entry) {
        if matches!(entry.kind, ClipKind::Text | ClipKind::Link) {
            self.add(entry.hash);
        }
    }

    /// A take read `text`, so it never goes back either.
    pub(super) fn taken(&mut self, text: &str) {
        self.add(hash::of_text(text));
    }

    /// The history kept `parts` again, so their text may go back.
    pub(super) fn kept(&mut self, parts: &[ClipPart]) {
        if let Some(text) = text_of(parts) {
            let hash = hash::of_text(text);
            self.hashes.retain(|h| *h != hash);
        }
    }

    /// True when `text` left the history and was not kept since.
    pub(super) fn has(&self, text: &str) -> bool {
        self.hashes.contains(&hash::of_text(text))
    }

    /// Keeps `hash`, unless it is kept already; past the most it holds, the oldest goes.
    fn add(&mut self, hash: u64) {
        if self.hashes.contains(&hash) {
            return;
        }
        if self.hashes.len() >= self.max {
            self.hashes.pop_front();
        }
        self.hashes.push_back(hash);
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

    #[test]
    fn a_text_stays_lost_until_kept_and_past_the_most_the_oldest_goes() {
        let mut lost = Lost::new(2);
        lost.taken("a");
        lost.gone(&entry("b"));
        assert!(lost.has("a") && lost.has("b"));
        lost.taken("a");
        lost.taken("c");
        assert!(!lost.has("a"), "a went first, held once");
        assert!(lost.has("b") && lost.has("c"));
        lost.kept(&[text("b")]);
        assert!(!lost.has("b") && lost.has("c"));
    }

    #[test]
    fn a_picture_entry_is_never_held() {
        let mut lost = Lost::new(2);
        let mut picture = entry("d");
        picture.kind = ClipKind::Image;
        lost.gone(&picture);
        assert!(!lost.has("d"));
    }
}
