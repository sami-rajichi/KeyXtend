//! Our input on its way back through the hooks, so input of the hand never overtakes ours. The host keeps the acts it
//! sent, in order, and each echo matches the next act of its kind, so it knows which batch came back; input waits until
//! the newest batch's first echo, which proves all of it passed. A wait given up keeps its acts, so their late echoes
//! still match them and never end a newer batch's wait.

use std::time::Duration;

use kx_module_api::Mono;
use kx_module_api::ports::PointerAct;

use super::held::LET_GO;

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

/// The acts sent and not back yet, and the wait for the newest batch.
#[derive(Debug)]
pub(crate) struct Echo {
    /// Every act sent and not back, oldest first, given-up ones too; only the loop side adds to it, within its room.
    flight: Vec<PointerAct>,
    /// How many acts of `flight` came back or were passed by.
    back: usize,
    /// Where the newest batch starts in `flight`.
    newest: usize,
    /// When the newest batch went out, until its first echo or `echo_ms`.
    since: Option<Mono>,
    /// Presses trimmed for room while still out, one per kind, in room reserved for a let-go; older than `flight`.
    doubt: Vec<PointerAct>,
    /// How long that may take.
    wait: Duration,
}

impl Echo {
    /// No wait, `echo_ms` for each one, and room for `room` acts out at once.
    pub(crate) fn new(echo_ms: u32, room: usize) -> Self {
        Self {
            flight: Vec::with_capacity(room),
            back: 0,
            newest: 0,
            since: None,
            doubt: Vec::with_capacity(LET_GO),
            wait: Duration::from_millis(u64::from(echo_ms)),
        }
    }

    /// Our batch `acts`, all taken by the OS, went out at `at`; later input waits for its first echo. The oldest acts
    /// out make room for it when it would not fit, their presses kept in doubt.
    pub(crate) fn sent(&mut self, acts: &[PointerAct], at: Mono) {
        if acts.is_empty() {
            return;
        }
        let back = std::mem::take(&mut self.back);
        self.flight.drain(..back.min(self.flight.len()));
        self.trim(acts.len());
        self.newest = self.flight.len();
        self.flight.extend_from_slice(acts);
        self.since = Some(at);
    }

    /// One of our events came back as `act`. A press in doubt is older than every act out, so it takes its own echo;
    /// otherwise the echo matches our next act of its kind. A move matches only the next act, as a press sent with a
    /// move may echo a move first; a press or key passes by acts whose echo never came.
    pub(crate) fn back(&mut self, act: PointerAct) {
        if is_press(act) && self.forget(act) {
            return;
        }
        let rest = self.flight.get(self.back..).unwrap_or_default();
        let step = match act {
            PointerAct::Move(_) => rest
                .first()
                .is_some_and(|next| alike(*next, act))
                .then_some(0),
            _ => rest.iter().position(|next| alike(*next, act)),
        };
        if let Some(step) = step {
            self.back += step + 1;
            if self.back > self.newest {
                self.since = None;
            }
        }
    }

    /// True when no echo is awaited.
    #[cfg(test)]
    pub(crate) fn is_clear(&self) -> bool {
        self.since.is_none()
    }

    /// True while our `press` may still be out: sent and not back, or trimmed in doubt.
    pub(crate) fn out(&self, press: PointerAct) -> bool {
        let rest = self.flight.get(self.back..).unwrap_or_default();
        rest.iter().chain(&self.doubt).any(|act| alike(*act, press))
    }

    /// An Up is owed for `press` after all we sent, so its doubt no longer matters.
    pub(crate) fn owed(&mut self, press: PointerAct) {
        self.forget(press);
    }

    /// The room reserved at the start.
    #[cfg(test)]
    pub(crate) fn room(&self) -> usize {
        self.flight.capacity()
    }

    /// Whether input waits at `now`. A wait past `echo_ms` ends as `Lost`; its acts stay out, so a late echo still
    /// matches its own act.
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

    /// Drops the oldest acts out until `more` fit in the room reserved, keeping their presses in doubt.
    fn trim(&mut self, more: usize) {
        let room = self.flight.capacity();
        let over = self.flight.len().saturating_add(more).saturating_sub(room);
        let over = over.min(self.flight.len());
        for i in 0..over {
            if let Some(&act) = self.flight.get(i) {
                self.doubt_press(act);
            }
        }
        self.flight.drain(..over);
    }

    /// Keeps `act` in doubt when it is a press, one per kind, while the room reserved for them lasts.
    fn doubt_press(&mut self, act: PointerAct) {
        let new = !self.doubt.iter().any(|d| alike(*d, act));
        if is_press(act) && new && self.doubt.len() < self.doubt.capacity() {
            self.doubt.push(act);
        }
    }

    /// Takes the press of `press`'s kind out of doubt; true when one was there.
    fn forget(&mut self, press: PointerAct) -> bool {
        let before = self.doubt.len();
        self.doubt.retain(|d| !alike(*d, press));
        self.doubt.len() < before
    }
}

/// True for a button press or a key press.
fn is_press(act: PointerAct) -> bool {
    matches!(act, PointerAct::Down(..) | PointerAct::KeyDown(_))
}

/// True when `a` and `b` are the same kind of act, whatever their points.
fn alike(a: PointerAct, b: PointerAct) -> bool {
    match (a, b) {
        (PointerAct::Move(_), PointerAct::Move(_)) => true,
        (PointerAct::Down(x, _), PointerAct::Down(y, _))
        | (PointerAct::Up(x, _), PointerAct::Up(y, _)) => x == y,
        (PointerAct::KeyDown(x), PointerAct::KeyDown(y))
        | (PointerAct::KeyUp(x), PointerAct::KeyUp(y)) => x == y,
        _ => false,
    }
}
