//! Our input on its way back through the hooks (ledger ruling 36). The host keeps the acts it sent, in order, and each
//! echo matches the next act of its kind, so it knows which batch came back; input waits until the newest batch's first
//! echo, which proves all of it passed.

use std::time::Duration;

use kx_module_api::Mono;
use kx_module_api::ports::PointerAct;

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
    /// Every act sent since the hooks last caught up, oldest first; only the loop side adds to it.
    flight: Vec<PointerAct>,
    /// How many acts of `flight` came back or were passed by.
    back: usize,
    /// Where the newest batch starts in `flight`.
    newest: usize,
    /// When the newest batch went out, until its first echo.
    since: Option<Mono>,
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
            wait: Duration::from_millis(u64::from(echo_ms)),
        }
    }

    /// Our batch `acts`, all taken by the OS, went out at `at`; later input waits for its first echo.
    pub(crate) fn sent(&mut self, acts: &[PointerAct], at: Mono) {
        if acts.is_empty() {
            return;
        }
        let back = std::mem::take(&mut self.back);
        self.flight.drain(..back.min(self.flight.len()));
        self.newest = self.flight.len();
        self.flight.extend_from_slice(acts);
        self.since = Some(at);
    }

    /// One of our events came back as `act`: it matches our next act of its kind. A move matches only the next act, as
    /// a press sent with a move may echo a move first; a press or key passes by acts whose echo never came.
    pub(crate) fn back(&mut self, act: PointerAct) {
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
    pub(crate) fn is_clear(&self) -> bool {
        self.since.is_none()
    }

    /// The room reserved at the start.
    #[cfg(test)]
    pub(crate) fn room(&self) -> usize {
        self.flight.capacity()
    }

    /// Whether input waits at `now`. A wait past `echo_ms` ends as `Lost`, and what is out is then given up.
    pub(crate) fn check(&mut self, now: Mono) -> Wait {
        match self.since {
            None => Wait::Clear,
            Some(at) if now.saturating_since(at) > self.wait => {
                self.since = None;
                self.flight.clear();
                self.back = 0;
                Wait::Lost
            }
            Some(_) => Wait::Waiting,
        }
    }
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
