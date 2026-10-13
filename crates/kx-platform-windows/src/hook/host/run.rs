//! The host's loop side: turns, the health check, the reinstall and the stop, and how a batch goes out.

use std::panic::{AssertUnwindSafe, catch_unwind};

use kx_module_api::Mono;
use kx_module_api::ports::PointerAct;

use super::{Host, Trust};
use crate::hook::Os;
use crate::hook::batch::has_mouse;
use crate::hook::held::Held;
use crate::hook::queue::Echo;
use crate::hook::stats::Stats;
use crate::hook::timing::timer_ms;

/// How many acts the OS took out of how many were sent.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub(super) struct Sent {
    took: usize,
    total: usize,
}

impl Sent {
    /// Both counts together.
    fn and(self, other: Self) -> Self {
        Self {
            took: self.took.saturating_add(other.took),
            total: self.total.saturating_add(other.total),
        }
    }
}

impl<O: Os> Host<O> {
    /// Sends the queue, lets go when the engine is stale, settles it and re-arms the deadline timer.
    pub(super) fn run_turn(&mut self) {
        self.woken = false;
        let now = self.os.now();
        self.flush(now);
        if matches!(self.trust, Trust::Stale | Trust::Broken) {
            self.let_go(now);
        }
        if self.trust == Trust::Whole {
            self.lent.clear();
            self.engine.settle(now, &mut self.lent);
            self.send_lent(now);
        }
        let due = match self.trust {
            Trust::Whole => self.engine.deadline(),
            _ => None,
        };
        self.os.arm(due.map(|due| timer_ms(due, now)));
    }

    /// A turn, then the health check.
    pub(super) fn run_check(&mut self) {
        self.run_turn();
        let now = self.os.now();
        if self.health.due(now, self.os.last_input()) {
            self.reinstall(now);
        }
    }

    /// The stop's input: the queue, the release, then whatever our input still holds; a cut send and dropped acts are
    /// logged, counts only.
    pub(super) fn finish(&mut self) {
        let now = self.os.now();
        let sent = self
            .flush(now)
            .and(self.let_go(now))
            .and(self.let_held_go(now));
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

    /// Puts both hooks back. The engine missed input, so it lets go first, and no hook can see that input come back.
    fn reinstall(&mut self, now: Mono) {
        self.os.unhook();
        self.let_go(now);
        self.echo.clear();
        match self.os.hook() {
            Ok(()) => self.stats.reinstalled(),
            Err(code) if !self.warned => {
                self.warned = true;
                tracing::warn!(code, "the pointer hooks could not be put back");
            }
            Err(_) => {}
        }
    }

    /// Calls the engine's `release` and sends its acts; a broken engine gets this once. An engine that ends there, or
    /// panics in it, is done, and the host lets go of what our input still holds.
    fn let_go(&mut self, now: Mono) -> Sent {
        self.trust = match self.trust {
            Trust::Done => return Sent::default(),
            Trust::Broken => Trust::Done,
            Trust::Whole | Trust::Stale => Trust::Whole,
        };
        self.lent.clear();
        let (engine, lent) = (&mut self.engine, &mut self.lent);
        if catch_unwind(AssertUnwindSafe(|| engine.release(lent))).is_err() {
            self.lent.clear();
            self.trust = Trust::Done;
        }
        let sent = self.send_lent(now);
        match self.trust {
            Trust::Done => sent.and(self.let_held_go(now)),
            _ => sent,
        }
    }

    /// Sends an Up for each button and a key-up for each key our input still holds.
    fn let_held_go(&mut self, now: Mono) -> Sent {
        let acts = self.held.release(self.at);
        out(
            &self.os,
            &mut self.echo,
            &mut self.held,
            &self.stats,
            &acts,
            now,
        )
    }

    /// Sends the queue in one batch, which ends with an Up for each queued press whose release went ahead of it.
    fn flush(&mut self, now: Mono) -> Sent {
        self.queue.seal();
        let (os, echo, held) = (&self.os, &mut self.echo, &mut self.held);
        let sent = out(os, echo, held, &self.stats, self.queue.acts(), now);
        self.queue.clear();
        sent
    }

    /// Sends what the engine pushed into the lent buffer.
    fn send_lent(&mut self, now: Mono) -> Sent {
        let (os, echo, held) = (&self.os, &mut self.echo, &mut self.held);
        let sent = out(os, echo, held, &self.stats, &self.lent, now);
        self.lent.clear();
        sent
    }
}

/// Sends `acts` in one batch and notes what the taken part holds down. Mouse input starts the wait for its echo; a
/// batch the OS cut counts, and then no echo is awaited.
fn out<O: Os>(
    os: &O,
    echo: &mut Echo,
    held: &mut Held,
    stats: &Stats,
    acts: &[PointerAct],
    now: Mono,
) -> Sent {
    let total = acts.len();
    if total == 0 {
        return Sent::default();
    }
    if has_mouse(acts) {
        echo.sent(now);
    }
    let took = os.send(acts).min(total);
    held.sent(acts.get(..took).unwrap_or_default());
    if took < total {
        stats.send_failed();
        echo.clear();
    }
    Sent { took, total }
}
