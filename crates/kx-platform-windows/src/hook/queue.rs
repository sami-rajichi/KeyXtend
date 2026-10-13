//! Input behind our own (P1): what the engine lets through waits until our input comes back through the hook, so
//! nothing overtakes it. The queue's room is reserved once, so a callback never allocates.

use std::time::Duration;

use kx_module_api::Mono;
use kx_module_api::ports::PointerAct;

/// Acts the queue holds besides room for one engine call: a quarter second of moves at 1000 Hz.
pub(crate) const QUEUE_ROOM: usize = 256;

/// The acts waiting for the hook thread's next turn.
#[derive(Debug)]
pub(crate) struct Queue {
    acts: Vec<PointerAct>,
    /// The room one engine call needs: its acts and the event it lets through.
    call: usize,
    /// An overflow was counted since the queue last went out.
    over: bool,
}

impl Queue {
    /// A queue for an engine that pushes at most `max_acts` per call.
    pub(crate) fn new(max_acts: usize) -> Self {
        let call = max_acts.saturating_add(1);
        Self {
            acts: Vec::with_capacity(QUEUE_ROOM.saturating_add(call)),
            call,
            over: false,
        }
    }

    /// True when one more engine call might not fit, so events must pass without the engine.
    pub(crate) fn full(&self) -> bool {
        self.acts.capacity() - self.acts.len() < self.call
    }

    /// Notes an overflow; true only for the first since the queue last went out.
    pub(crate) fn overflow(&mut self) -> bool {
        !std::mem::replace(&mut self.over, true)
    }

    /// Queues `act`; the caller checked `full` first.
    pub(crate) fn push(&mut self, act: PointerAct) {
        self.acts.push(act);
    }

    /// Queues `acts` in order; the caller checked `full` first.
    pub(crate) fn extend(&mut self, acts: &[PointerAct]) {
        self.acts.extend_from_slice(acts);
    }

    /// The acts waiting, oldest first.
    pub(crate) fn acts(&self) -> &[PointerAct] {
        &self.acts
    }

    /// True when nothing waits.
    pub(crate) fn is_empty(&self) -> bool {
        self.acts.is_empty()
    }

    /// Empties the queue once its acts went out, keeping its room.
    pub(crate) fn clear(&mut self) {
        self.acts.clear();
        self.over = false;
    }

    /// The room reserved at the start.
    #[cfg(test)]
    pub(crate) fn room(&self) -> usize {
        self.acts.capacity()
    }
}

/// Whether input must still wait for our own.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub(crate) enum Wait {
    /// Nothing of ours is out.
    Clear,
    /// Our input has not come back yet.
    Waiting,
    /// It did not come back within `echo_ms`; the wait ended now.
    Lost,
}

/// The wait for our own mouse input to come back through the hook.
#[derive(Debug)]
pub(crate) struct Echo {
    /// When our last mouse input went out.
    since: Option<Mono>,
    /// How long it may take.
    wait: Duration,
}

impl Echo {
    /// No wait, and `echo_ms` for each one.
    pub(crate) fn new(echo_ms: u32) -> Self {
        Self {
            since: None,
            wait: Duration::from_millis(u64::from(echo_ms)),
        }
    }

    /// Our mouse input went out at `at`; later input waits until it comes back.
    pub(crate) fn sent(&mut self, at: Mono) {
        self.since = Some(at);
    }

    /// Our input came back, or never will: the wait ends.
    pub(crate) fn clear(&mut self) {
        self.since = None;
    }

    /// Whether input waits at `now`; a wait past `echo_ms` ends as `Lost`.
    pub(crate) fn check(&mut self, now: Mono) -> Wait {
        match self.since {
            None => Wait::Clear,
            Some(at) if now.saturating_since(at) > self.wait => {
                self.since = None;
                Wait::Lost
            }
            Some(_) => Wait::Waiting,
        }
    }
}
