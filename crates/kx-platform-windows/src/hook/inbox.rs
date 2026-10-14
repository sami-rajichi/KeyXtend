//! Echoes heard while our own send runs: Windows calls our hooks inside `SendInput`, while the host is busy, so the
//! host lends this inbox to the send and settles what it holds right after.

use std::fmt;

use kx_module_api::Redacted;
use kx_module_api::ports::Point;

use crate::keys::{is_replay, replay_id};

/// The most hook events one act echoes as: a pointed act gives a move, then its button.
pub(crate) const ECHOES_PER_ACT: usize = 2;

/// One echo heard inside our send: the act's id, and a mouse echo's point.
#[derive(Copy, Clone, PartialEq, Eq)]
pub(crate) struct Echoed {
    /// The id of the act it echoes.
    pub id: u16,
    /// Where a mouse echo landed; `None` for a key.
    pub at: Option<Point>,
}

/// Shows the id only: a point can show what was clicked.
impl fmt::Debug for Echoed {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Echoed")
            .field("id", &self.id)
            .field("at", &self.at.map(Redacted::new))
            .finish()
    }
}

/// The echoes heard during one send, oldest first, in room reserved once; it never grows.
#[derive(Debug)]
pub(crate) struct Inbox(Vec<Echoed>);

impl Inbox {
    /// An inbox for the echoes of `acts` acts, reserved now.
    pub(crate) fn new(acts: usize) -> Self {
        Self(Vec::with_capacity(acts.saturating_mul(ECHOES_PER_ACT)))
    }

    /// An inbox with no room, which keeps nothing.
    pub(crate) const fn none() -> Self {
        Self(Vec::new())
    }

    /// Notes our echo tagged `extra`, with a mouse echo's point `at`, while there is room; false when it is not ours
    /// or there is no room, so it is dropped.
    pub(crate) fn hear(&mut self, extra: usize, at: Option<Point>) -> bool {
        let kept = is_replay(extra) && self.0.len() < self.0.capacity();
        if kept {
            self.0.push(Echoed {
                id: replay_id(extra),
                at,
            });
        }
        kept
    }

    /// Takes the echoes out, oldest first, and keeps the room.
    pub(crate) fn drain(&mut self) -> std::vec::Drain<'_, Echoed> {
        self.0.drain(..)
    }

    /// The room reserved.
    #[cfg(test)]
    pub(crate) fn room(&self) -> usize {
        self.0.capacity()
    }
}
