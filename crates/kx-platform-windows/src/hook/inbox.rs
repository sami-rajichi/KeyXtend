//! Echoes heard while our own send runs: Windows calls our hooks inside `SendInput`, while the host is busy, so the
//! host lends this inbox to the send and settles what it holds right after.

use crate::keys::{is_replay, replay_id};

/// The most hook events one act echoes as: a pointed act gives a move, then its button.
pub(crate) const ECHOES_PER_ACT: usize = 2;

/// The ids of the acts echoed during one send, oldest first, in room reserved once; it never grows.
#[derive(Debug)]
pub(crate) struct Inbox(Vec<u16>);

impl Inbox {
    /// An inbox for the echoes of `acts` acts, reserved now.
    pub(crate) fn new(acts: usize) -> Self {
        Self(Vec::with_capacity(acts.saturating_mul(ECHOES_PER_ACT)))
    }

    /// An inbox with no room, which keeps nothing.
    pub(crate) const fn none() -> Self {
        Self(Vec::new())
    }

    /// Notes the id of our echo tagged `extra` while there is room; false when it is not ours or there is no room, so
    /// it is dropped.
    pub(crate) fn hear(&mut self, extra: usize) -> bool {
        let kept = is_replay(extra) && self.0.len() < self.0.capacity();
        if kept {
            self.0.push(replay_id(extra));
        }
        kept
    }

    /// Takes the ids out, oldest first, and keeps the room.
    pub(crate) fn drain(&mut self) -> std::vec::Drain<'_, u16> {
        self.0.drain(..)
    }

    /// Forgets every id and keeps the room.
    pub(crate) fn clear(&mut self) {
        self.0.clear();
    }

    /// The room reserved.
    #[cfg(test)]
    pub(crate) fn room(&self) -> usize {
        self.0.capacity()
    }
}
