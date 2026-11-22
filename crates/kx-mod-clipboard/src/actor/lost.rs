//! The texts that left the history, by a take, a delete or a sweep, as content hashes alone: Paste at once never writes
//! one back until the history keeps it again (owner's check, 2026-11-21). The copy a take read also keeps nothing when
//! the watch tells it within `take_wait_ms`, as that note can come after the take, or twice. A rich entry's text is not
//! matched, as only the board can name it; past `max_entries` the oldest goes.

use std::collections::VecDeque;
use std::time::Duration;

use kx_module_api::Mono;
use kx_module_api::ports::{ClipKind, ClipPart};

use crate::entry::{Entry, text_of};
use crate::hash;
use crate::settings::ClipSettings;

/// The hashes of the texts that left the history, oldest first, and of the takes' copies whose note is due, each with
/// the end of its wait; it holds at most `max` and `due_max`.
pub(super) struct Lost {
    hashes: VecDeque<u64>,
    max: usize,
    due: VecDeque<(u64, Mono)>,
    due_max: usize,
    wait: Duration,
}

impl Lost {
    /// None yet; at most `max_entries` texts, and `queue_copies` takes due, as no more copies wait for the history.
    pub(super) fn new(s: &ClipSettings) -> Self {
        Self {
            hashes: VecDeque::new(),
            max: s.max_entries,
            due: VecDeque::new(),
            due_max: s.queue_copies,
            wait: s.take_wait(),
        }
    }

    /// `entry` left the history; a text entry's hash is kept, matched only when that text was all it held.
    pub(super) fn gone(&mut self, entry: &Entry) {
        if matches!(entry.kind, ClipKind::Text | ClipKind::Link) {
            self.add(entry.hash);
        }
    }

    /// A take read `text` at `now`, so it never goes back, and its copy's note is due until the wait ends; past the
    /// most due, the oldest is due no more.
    pub(super) fn taken(&mut self, text: &str, now: Mono) {
        let hash = hash::of_text(text);
        self.add(hash);
        self.expire(now);
        if self.due.len() >= self.due_max {
            self.due.pop_front();
        }
        self.due.push_back((hash, now.saturating_add(self.wait)));
    }

    /// The watch told a copy of `parts` at `now`: true when one of its texts is a take's copy still due, which is then
    /// due no more.
    pub(super) fn noted(&mut self, parts: &[ClipPart], now: Mono) -> bool {
        self.expire(now);
        let texts: Vec<u64> = (parts.iter())
            .filter_map(|p| match p {
                ClipPart::Text(t) => Some(hash::of_text(t.expose())),
                _ => None,
            })
            .collect();
        let at = self.due.iter().position(|(h, _)| texts.contains(h));
        at.and_then(|i| self.due.remove(i)).is_some()
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

    /// Drops the takes whose wait ended by `now`.
    fn expire(&mut self, now: Mono) {
        self.due.retain(|(_, end)| now < *end);
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
    fn a_text_stays_lost_until_kept_and_past_the_most_the_oldest_goes() {
        let (mut lost, now) = (lost(2, 1), Mono::default());
        lost.taken("a", now);
        lost.gone(&entry("b"));
        assert!(lost.has("a") && lost.has("b"));
        lost.taken("a", now);
        lost.taken("c", now);
        assert!(!lost.has("a"), "a went first, held once");
        assert!(lost.has("b") && lost.has("c"));
        lost.kept(&[text("b")]);
        assert!(!lost.has("b") && lost.has("c"));
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
    fn a_takes_copy_is_noted_once_within_its_wait_and_past_the_most_due_the_oldest_goes() {
        let mut lost = lost(4, 2);
        let (now, wait) = (Mono::default(), lost.wait);
        for t in ["a", "b", "c"] {
            lost.taken(t, now);
        }
        assert!(!lost.noted(&[text("a")], now), "a went first");
        assert!(lost.noted(&[text("b")], now));
        assert!(!lost.noted(&[text("b")], now), "noted once");
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
