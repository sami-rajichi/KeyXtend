//! Input behind our own (P1): what the engine lets through waits until our input comes back through the hook, so
//! nothing overtakes it. Moves coalesce, buttons and keys keep a headroom, and the room is reserved once, so a callback
//! never allocates.

use std::time::Duration;

use kx_module_api::Mono;
use kx_module_api::ports::{Button, Point, PointerAct};

use super::accept::Kind;
use super::held::{SLOTS, slot};

/// Acts of any kind the queue holds: a quarter second of input at 1000 Hz.
pub(crate) const QUEUE_ROOM: usize = 256;
/// Room past `QUEUE_ROOM` that only buttons and keys use, so they stay in order once moves filled the rest.
pub(crate) const HEADROOM: usize = 16;

/// The acts waiting for the hook thread's next turn.
#[derive(Debug)]
pub(crate) struct Queue {
    acts: Vec<PointerAct>,
    /// The most acts it takes: `QUEUE_ROOM`, the headroom and one engine call. `seal` may add one Up per button.
    limit: usize,
    /// The room one engine call needs: its acts and the event it lets through.
    call: usize,
    /// An overflow was counted since the queue last went out.
    over: bool,
    /// Per button slot, the button and where the hand let it go while its press waited here; a later press clears it.
    passed: [Option<(Button, Point)>; SLOTS],
}

impl Queue {
    /// A queue for an engine that pushes at most `max_acts` per call.
    pub(crate) fn new(max_acts: usize) -> Self {
        let call = max_acts.saturating_add(1);
        let limit = QUEUE_ROOM.saturating_add(HEADROOM).saturating_add(call);
        Self {
            acts: Vec::with_capacity(limit.saturating_add(SLOTS)),
            limit,
            call,
            over: false,
            passed: [None; SLOTS],
        }
    }

    /// True when one more engine call might not fit, so events must pass without the engine.
    pub(crate) fn full(&self) -> bool {
        self.limit.saturating_sub(self.acts.len()) < self.call
    }

    /// Notes an overflow; true only for the first since the queue last went out.
    pub(crate) fn overflow(&mut self) -> bool {
        !std::mem::replace(&mut self.over, true)
    }

    /// Queues `act`; false when it found no room. A move replaces a move just before it, and takes no headroom.
    pub(crate) fn put(&mut self, act: PointerAct) -> bool {
        let len = self.acts.len();
        match (act, self.acts.last_mut()) {
            (PointerAct::Move(_), Some(last @ PointerAct::Move(_))) => *last = act,
            (PointerAct::Move(_), _) if len >= QUEUE_ROOM => return false,
            _ if len >= self.limit => return false,
            _ => self.acts.push(act),
        }
        true
    }

    /// Queues `acts` in order; how many buttons and keys found no room. A move with no room is left to the next.
    pub(crate) fn extend(&mut self, acts: &[PointerAct]) -> usize {
        let mut cut = 0;
        for &act in acts {
            if !self.put(act) && !matches!(act, PointerAct::Move(_)) {
                cut += 1;
            }
        }
        cut
    }

    /// The hand's event `kind` at `at` passed ahead of the queue: an Up must follow a press still waiting here.
    pub(crate) fn passed(&mut self, kind: Kind, at: Point) {
        if self.acts.is_empty() {
            return;
        }
        let (button, up) = match kind {
            Kind::Down(button) => (button, None),
            Kind::Up(button) => (button, Some((button, at))),
            Kind::Move => return,
        };
        if let Some(cell) = self.passed.get_mut(slot(button)) {
            *cell = up;
        }
    }

    /// Ends the batch with an Up for each button whose press is its last act here and whose release already passed.
    /// The room for these was reserved, so this never allocates.
    pub(crate) fn seal(&mut self) {
        for (button, at) in self.passed.into_iter().flatten() {
            if self.pressed(button) {
                self.acts.push(PointerAct::Up(button, at));
            }
        }
    }

    /// True when the last act here for `button` is its press.
    fn pressed(&self, button: Button) -> bool {
        self.acts.iter().rev().find_map(|act| match *act {
            PointerAct::Down(b, _) if b == button => Some(true),
            PointerAct::Up(b, _) if b == button => Some(false),
            _ => None,
        }) == Some(true)
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
        self.passed = [None; SLOTS];
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
