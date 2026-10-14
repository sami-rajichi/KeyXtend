//! Our input on its way back through the hooks, so input of the hand never overtakes ours. Each act we send carries its
//! own id, so its echo settles it and every older act still out, and only an echo of the newest batch ends the wait.

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

/// The acts sent and not settled yet, and the wait for the newest batch.
#[derive(Debug)]
pub(crate) struct Echo {
    /// Every act sent and not settled, oldest first, given-up ones too; only the loop side adds to it, within its room.
    flight: Vec<PointerAct>,
    /// The id the next act sent gets; `flight` holds the ids just below it.
    next: u32,
    /// How many acts at the start of `flight` are settled: back, or passed by a newer act's echo.
    back: usize,
    /// Where the newest batch starts in `flight`.
    newest: usize,
    /// When the newest batch went out, until an echo of it or `echo_ms`.
    since: Option<Mono>,
    /// Presses whose echo never came, one per kind with the newest one's id, in room reserved for a let-go.
    doubt: Vec<(u32, PointerAct)>,
    /// How long that may take.
    wait: Duration,
}

impl Echo {
    /// No wait, `echo_ms` for each one, and room for `room` acts out at once.
    pub(crate) fn new(echo_ms: u32, room: usize) -> Self {
        Self {
            flight: Vec::with_capacity(room),
            next: 0,
            back: 0,
            newest: 0,
            since: None,
            doubt: Vec::with_capacity(LET_GO),
            wait: Duration::from_millis(u64::from(echo_ms)),
        }
    }

    /// The id the next batch's first act gets.
    pub(crate) fn next(&self) -> u32 {
        self.next
    }

    /// Our batch `acts`, all taken by the OS, went out at `at` with ids from `next`; later input waits for its echo. The
    /// oldest acts out make room for it when it would not fit, their presses kept in doubt.
    pub(crate) fn sent(&mut self, acts: &[PointerAct], at: Mono) {
        if acts.is_empty() {
            return;
        }
        let back = std::mem::take(&mut self.back);
        self.flight.drain(..back.min(self.flight.len()));
        self.trim(acts.len());
        self.newest = self.flight.len();
        self.flight.extend_from_slice(acts);
        self.next = self.next.wrapping_add(count(acts.len()));
        self.since = Some(at);
    }

    /// Our act `id` came back: it and every older act still out are settled, and those older presses stay in doubt, as
    /// their echo never came. An echo of the newest batch ends the wait, and one that names no act out changes nothing.
    pub(crate) fn back(&mut self, id: u32) {
        self.doubt.retain(|(d, _)| *d != id);
        let Some(at) = self.index(id).filter(|at| *at >= self.back) else {
            return;
        };
        for k in self.back..at {
            self.doubt_press(k);
        }
        self.back = at + 1;
        if at >= self.newest {
            self.since = None;
        }
    }

    /// True when no echo is awaited.
    #[cfg(test)]
    pub(crate) fn is_clear(&self) -> bool {
        self.since.is_none()
    }

    /// True while our `press` may still be out: sent and not settled, or in doubt.
    pub(crate) fn out(&self, press: PointerAct) -> bool {
        let rest = self.flight.get(self.back..).unwrap_or_default();
        let doubt = self.doubt.iter().map(|(_, act)| act);
        rest.iter().chain(doubt).any(|act| alike(*act, press))
    }

    /// An Up is owed for `press` after all we sent, so its doubt no longer matters.
    pub(crate) fn owed(&mut self, press: PointerAct) {
        self.doubt.retain(|(_, act)| !alike(*act, press));
    }

    /// The room reserved at the start.
    #[cfg(test)]
    pub(crate) fn room(&self) -> usize {
        self.flight.capacity()
    }

    /// The next act sent gets `id`, as after many sends.
    #[cfg(test)]
    pub(crate) fn skip_to(&mut self, id: u32) {
        self.next = id;
    }

    /// Whether input waits at `now`. A wait past `echo_ms` ends as `Lost`; its acts stay out, so a late echo still
    /// settles them.
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

    /// Where act `id` sits in `flight`, if it is there.
    fn index(&self, id: u32) -> Option<usize> {
        let len = self.flight.len();
        let age = usize::try_from(self.next.wrapping_sub(id)).ok()?;
        (1..=len).contains(&age).then(|| len - age)
    }

    /// Drops the oldest acts out until `more` fit in the room reserved, keeping their presses in doubt.
    fn trim(&mut self, more: usize) {
        let room = self.flight.capacity();
        let over = self.flight.len().saturating_add(more).saturating_sub(room);
        let over = over.min(self.flight.len());
        for k in 0..over {
            self.doubt_press(k);
        }
        self.flight.drain(..over);
    }

    /// Keeps act `k` of `flight` in doubt when it is a press: one per kind with the newest one's id, while the room
    /// reserved for them lasts.
    fn doubt_press(&mut self, k: usize) {
        let Some(&act) = self.flight.get(k).filter(|act| is_press(**act)) else {
            return;
        };
        let id = self.next.wrapping_sub(count(self.flight.len() - k));
        if let Some(slot) = self.doubt.iter_mut().find(|(_, d)| alike(*d, act)) {
            *slot = (id, act);
        } else if self.doubt.len() < self.doubt.capacity() {
            self.doubt.push((id, act));
        }
    }
}

/// `n` acts as a count of ids; a batch is far smaller than `u32::MAX`.
fn count(n: usize) -> u32 {
    u32::try_from(n).unwrap_or(u32::MAX)
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
