//! The host's loop side: turns, the health check, the reinstall, the panic hook's let-go and the stop. Each puts its
//! input in one batch, so the first echo proves all of it passed and nothing of the hand's can come between.

use std::panic::{AssertUnwindSafe, catch_unwind};

use kx_module_api::Mono;
use kx_module_api::ports::{PointerAct, PointerEngine, PointerEvent};

use super::{Host, Trust};
use crate::hook::Os;
use crate::hook::timing::timer_ms;

/// How many acts the OS took out of how many were sent.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub(super) struct Sent {
    took: usize,
    total: usize,
}

impl<O: Os> Host<O> {
    /// Sends owed Ups, the queue, the stale engine's let-go and its settle as one batch, then sets the deadline timer
    /// when the deadline changed.
    pub(super) fn run_turn(&mut self) {
        self.woken = false;
        let now = self.os.now();
        self.take_owed();
        self.take_queue();
        if matches!(self.trust, Trust::Stale | Trust::Broken) {
            self.take_let_go();
        }
        if self.trust == Trust::Whole {
            self.lent.clear();
            self.engine.settle(now, &mut self.lent);
            self.out.extend_from_slice(&self.lent);
            self.lent.clear();
        }
        let _ = self.ship(now);
        let due = match self.trust {
            Trust::Whole => self.engine.deadline(),
            _ => None,
        };
        self.arm(due, now);
    }

    /// Sets the deadline timer for `due`, or stops it, only when that changed; a refused timer is asked for again at
    /// the next turn.
    fn arm(&mut self, due: Option<Mono>, now: Mono) {
        if due != self.armed && self.os.arm(due.map(|due| timer_ms(due, now))) {
            self.armed = due;
        }
    }

    /// After `on`: asks for a turn when input waits, the deadline moved from `before` or the engine holds notes. The
    /// wake stays one until its turn runs, so a fast mouse posts no flood.
    pub(super) fn ask_turn(&mut self, before: Option<Mono>) {
        let moved = self.engine.deadline() != before;
        if !self.queue.is_empty() || moved || self.engine.has_notes() {
            self.wake();
        }
    }

    /// A turn, then the health check.
    pub(super) fn run_check(&mut self) {
        self.run_turn();
        let now = self.os.now();
        let away = self.os.away();
        if self.health.due(now, self.os.last_input(), away) {
            self.reinstall(now);
        }
    }

    /// Owed Ups, the queue, the release, then whatever our input still holds, in that order and in one batch, so no
    /// queued press follows its Up.
    pub(super) fn run_release(&mut self) -> Sent {
        let now = self.os.now();
        self.take_owed();
        self.take_queue();
        self.take_release();
        self.take_held();
        self.ship(now)
    }

    /// The stop's input, as `run_release` sends it; a cut send and dropped acts are logged, counts only.
    pub(super) fn finish(&mut self) {
        let sent = self.run_release();
        if sent.took < sent.total {
            tracing::warn!(
                sent = sent.took,
                total = sent.total,
                "the pointer hook could not send all its input as it stopped"
            );
        }
        let dropped = self.stats.dropped();
        if dropped > 0 {
            tracing::warn!(dropped, "the pointer hook dropped input it had no room for");
        }
    }

    /// Puts both hooks back. The engine missed input, so the queue and its let-go go out first; a turn then takes the
    /// notes of that let-go.
    fn reinstall(&mut self, now: Mono) {
        self.os.unhook();
        self.take_owed();
        self.take_queue();
        self.take_let_go();
        let _ = self.ship(now);
        match self.os.hook() {
            Ok(()) => self.stats.reinstalled(),
            Err(code) if !self.warned => {
                self.warned = true;
                tracing::warn!(code, "the pointer hooks could not be put back");
            }
            Err(_) => {}
        }
        self.wake();
    }

    /// Adds an Up for each press of ours the hand let go before it was proven ahead; first, so nothing queued after
    /// that release is let go. Each stays owed until the OS takes an Up for it.
    fn take_owed(&mut self) {
        self.out.extend(self.owed.release(self.at));
    }

    /// Adds the queue, which ends with an Up for each queued press whose release went ahead of it.
    fn take_queue(&mut self) {
        self.queue.seal();
        self.out.extend_from_slice(self.queue.acts());
        self.queue.clear();
    }

    /// Adds the engine's release; once that leaves it done, adds a let-go of what our input then still holds.
    fn take_let_go(&mut self) {
        self.take_release();
        if self.trust == Trust::Done {
            self.take_held();
        }
    }

    /// Adds the engine's `release`. A broken engine gets this once, and one done then, or by a panic in it, is dropped
    /// here on the loop side, so its module learns the run lost it.
    fn take_release(&mut self) {
        self.trust = match self.trust {
            Trust::Done => return,
            Trust::Broken => Trust::Done,
            Trust::Whole | Trust::Stale => Trust::Whole,
        };
        self.lent.clear();
        let (engine, lent) = (&mut self.engine, &mut self.lent);
        if catch_unwind(AssertUnwindSafe(|| engine.release(lent))).is_err() {
            self.lent.clear();
            self.trust = Trust::Done;
        }
        self.out.extend_from_slice(&self.lent);
        self.lent.clear();
        if self.trust == Trust::Done {
            // A zero-sized engine, so this allocates nothing.
            drop(std::mem::replace(&mut self.engine, Box::new(Spent)));
        }
    }

    /// Adds an Up for each button and a key-up for each key our input holds once the batch so far went out.
    fn take_held(&mut self) {
        let mut after = self.held;
        after.sent(&self.out);
        self.out.extend(after.release(self.at));
    }

    /// Sends the batch, then notes what its taken part holds down and awaits that part's echo; a cut batch counts. Input
    /// the OS took moves Windows' last input time before the hooks see it, so it counts as seen.
    fn ship(&mut self, now: Mono) -> Sent {
        let total = self.out.len();
        if total == 0 {
            return Sent::default();
        }
        let took = self.os.send(&self.out).min(total);
        let taken = self.out.get(..took).unwrap_or_default();
        self.held.sent(taken);
        for &act in taken {
            self.owed.forget(act);
        }
        self.echo.sent(taken, now);
        if took > 0 {
            self.health.saw(self.os.tick());
        }
        if took < total {
            self.stats.send_failed();
        }
        self.out.clear();
        Sent { took, total }
    }
}

/// What takes a given-up engine's place; the host never calls it.
struct Spent;

impl PointerEngine for Spent {
    fn on(&mut self, _: PointerEvent, _: Mono, _: &mut Vec<PointerAct>) -> bool {
        false
    }

    fn settle(&mut self, _: Mono, _: &mut Vec<PointerAct>) {}

    fn deadline(&self) -> Option<Mono> {
        None
    }

    fn has_notes(&self) -> bool {
        false
    }

    fn release(&mut self, _: &mut Vec<PointerAct>) {}

    fn max_acts(&self) -> usize {
        0
    }
}
