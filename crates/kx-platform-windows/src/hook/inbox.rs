//! What our hooks hear while our own send runs: Windows calls them inside `SendInput`, while the host is busy, so the
//! host lends this inbox to the send and settles our echoes right after. The user's events pass then, only counted.

use super::Source;
use super::accept::heard;
use crate::keys::{is_replay, replay_id};

/// The most hook events one act echoes as: a pointed act gives a move, then its button.
pub(crate) const ECHOES_PER_ACT: usize = 2;

/// The ids of the acts echoed during one send, oldest first, in room reserved once; it never grows.
#[derive(Debug)]
pub(crate) struct Inbox {
    ids: Vec<u16>,
    /// Whose input is the user's; none for an inbox that keeps nothing.
    source: Option<Source>,
    /// The user's events heard during sends since the last take; they passed without the host.
    passed: u32,
}

impl Inbox {
    /// An inbox for the echoes of `acts` acts, reserved now, that counts `source`'s events as the user's.
    pub(crate) fn new(acts: usize, source: Source) -> Self {
        Self {
            ids: Vec::with_capacity(acts.saturating_mul(ECHOES_PER_ACT)),
            source: Some(source),
            passed: 0,
        }
    }

    /// An inbox with no room, which keeps and counts nothing.
    pub(crate) const fn none() -> Self {
        Self {
            ids: Vec::new(),
            source: None,
            passed: 0,
        }
    }

    /// Notes the id of our echo tagged `extra` while there is room; false when it is not ours or there is no room, so
    /// it is dropped.
    pub(crate) fn hear(&mut self, extra: usize) -> bool {
        let kept = is_replay(extra) && self.ids.len() < self.ids.capacity();
        if kept {
            self.ids.push(replay_id(extra));
        }
        kept
    }

    /// Counts an event with these marks when it is the user's, as it passed without the host; others count nothing.
    pub(crate) fn user(&mut self, injected: bool, extra: usize) {
        if self.source.is_some_and(|s| heard(s, injected, extra)) {
            self.passed = self.passed.saturating_add(1);
        }
    }

    /// The user's events counted since the last take.
    pub(crate) fn take_passed(&mut self) -> u32 {
        std::mem::take(&mut self.passed)
    }

    /// Takes the ids out, oldest first, and keeps the room.
    pub(crate) fn drain(&mut self) -> std::vec::Drain<'_, u16> {
        self.ids.drain(..)
    }

    /// Forgets every id and keeps the room; the count stays, as those events did pass.
    pub(crate) fn clear(&mut self) {
        self.ids.clear();
    }

    /// The room reserved.
    #[cfg(test)]
    pub(crate) fn room(&self) -> usize {
        self.ids.capacity()
    }
}
