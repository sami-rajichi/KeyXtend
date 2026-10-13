//! The host on the hook thread (ADR-0027 decisions 2–8): the engine decides each event inside the callback, passed
//! input waits behind our own, the loop sends the engine's input, lagging hooks go back and the stop lets go. It is
//! generic over `Os`, so a fake drives it in tests.

mod run;

use std::panic::{AssertUnwindSafe, catch_unwind};
use std::sync::Arc;

use kx_module_api::Mono;
use kx_module_api::ports::{HookConfig, Point, PointerAct, PointerEngine, PointerEvent};

use super::accept::{Kind, Take, esc, kind, take};
use super::health::Health;
use super::queue::{Echo, Queue, Wait};
use super::stats::Stats;
use super::timing::event_at;
use super::{Mark, Os, Source};

/// The hook thread's state: the engine and its lent buffer, the input behind our own, the health check and the numbers.
pub(crate) struct Host<O: Os> {
    os: O,
    engine: Box<dyn PointerEngine>,
    source: Source,
    /// Lent to every engine call; reserved once with its `max_acts`, so pushes never allocate.
    lent: Vec<PointerAct>,
    /// The engine's `max_acts`.
    max: usize,
    queue: Queue,
    echo: Echo,
    health: Health,
    stats: Arc<Stats>,
    trust: Trust,
    /// A wake is posted and its turn has not run yet.
    woken: bool,
    /// A refused reinstall was logged.
    warned: bool,
}

/// When a callback ran, and the event's own stamp.
#[derive(Copy, Clone, Debug)]
struct Stamp {
    /// The platform clock as the callback began.
    t0: Mono,
    /// Windows' tick count then.
    tick: u32,
    /// The event's own tick stamp.
    time: u32,
}

/// How far the engine's view of the buttons can be trusted.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub(super) enum Trust {
    /// It saw every event it should.
    Whole,
    /// It missed events, so it lets go at the next turn.
    Stale,
    /// It panicked: events pass untouched, and it lets go once at the next turn.
    Broken,
    /// It panicked and had its one release; it is never called again.
    Done,
}

impl<O: Os> Host<O> {
    /// A host for `engine` on `source`'s input, as `cfg` says; `begin` installs the hooks.
    pub(crate) fn new(
        os: O,
        engine: Box<dyn PointerEngine>,
        source: Source,
        cfg: &HookConfig,
        stats: Arc<Stats>,
    ) -> Self {
        let max = engine.max_acts();
        let seen = os.last_input().unwrap_or_else(|| os.tick());
        Self {
            lent: Vec::with_capacity(max),
            max,
            queue: Queue::new(max),
            echo: Echo::new(cfg.echo_ms),
            health: Health::new(cfg, seen),
            os,
            engine,
            source,
            stats,
            trust: Trust::Whole,
            woken: false,
            warned: false,
        }
    }

    /// Installs both hooks.
    ///
    /// # Errors
    /// The OS error code when Windows refuses them.
    pub(crate) fn begin(&mut self) -> Result<(), i32> {
        self.os.hook()
    }

    /// Decides mouse message `msg` at `at`; true to drop it. A panic passes the event and marks the engine broken, and
    /// a hand's event that passes ahead of acts already queued is noted, so its Up never leaves a queued press held.
    pub(crate) fn mouse(&mut self, msg: u32, at: Point, mark: Mark) -> bool {
        let queued = !self.queue.is_empty();
        let dropped = self.guard(|h| h.on_mouse(msg, at, mark)).unwrap_or(false);
        let heard = take(self.source, mark.injected, mark.extra) == Take::Engine;
        if let (true, false, true, Some(kind)) = (queued, dropped, heard, kind(msg)) {
            self.queue.passed(kind, at);
        }
        dropped
    }

    /// Hears key message `msg` for `vk`: an Esc press reaches the engine, and every key passes on.
    pub(crate) fn key(&mut self, msg: u32, vk: u32, mark: Mark) {
        let _ = self.guard(|h| h.on_key(msg, vk, mark));
    }

    /// A turn, on a wake or the deadline timer.
    pub(crate) fn turn(&mut self) {
        let _ = self.guard(Self::run_turn);
    }

    /// The health timer: a turn, then the health check.
    pub(crate) fn check(&mut self) {
        let _ = self.guard(Self::run_check);
    }

    /// Sends the queue, then the engine's release, and only then stops the timer and removes the hooks, whatever failed.
    pub(crate) fn stop(&mut self) {
        let _ = self.guard(Self::finish);
        self.os.arm(None);
        self.os.unhook();
    }

    /// The lent buffer's and the queue's room.
    #[cfg(test)]
    pub(crate) fn room(&self) -> (usize, usize) {
        (self.lent.capacity(), self.queue.room())
    }

    /// Runs `call`; a panic marks the engine broken instead of reaching Windows, and asks for the turn that lets go.
    fn guard<T>(&mut self, call: impl FnOnce(&mut Self) -> T) -> Option<T> {
        let out = catch_unwind(AssertUnwindSafe(|| call(self))).ok();
        if out.is_none() {
            self.lent.clear();
            if matches!(self.trust, Trust::Whole | Trust::Stale) {
                self.trust = Trust::Broken;
            }
            self.wake();
        }
        out
    }

    fn on_mouse(&mut self, msg: u32, at: Point, mark: Mark) -> bool {
        let stamp = self.stamp(mark);
        match (take(self.source, mark.injected, mark.extra), kind(msg)) {
            (Take::Echo, _) => {
                self.echo.clear();
                false
            }
            (Take::Engine, Some(kind)) => {
                let event = self.event(kind, at);
                self.decide(event, stamp)
            }
            _ => false,
        }
    }

    fn on_key(&mut self, msg: u32, vk: u32, mark: Mark) {
        let stamp = self.stamp(mark);
        if esc(self.source, msg, vk, mark.injected, mark.extra) {
            // Esc is never dropped, whatever the engine answers.
            let _ = self.decide(PointerEvent::Esc, stamp);
        }
    }

    /// When the callback runs. The hooks count as current up to this tick, never up to the event's own stamp, which a
    /// program that injects input can date ahead.
    fn stamp(&mut self, mark: Mark) -> Stamp {
        let stamp = Stamp {
            t0: self.os.now(),
            tick: self.os.tick(),
            time: mark.time,
        };
        self.health.saw(stamp.tick);
        stamp
    }

    /// The event for `kind` at `at`; a press says whether it landed on our window, and its screen's DPI.
    fn event(&self, kind: Kind, at: Point) -> PointerEvent {
        match kind {
            Kind::Down(button) => PointerEvent::Down {
                button,
                at,
                ours: self.os.ours(at),
                dpi: self.os.dpi(at),
            },
            Kind::Up(button) => PointerEvent::Up { button, at },
            Kind::Move => PointerEvent::Move { at },
        }
    }

    /// Gives `event` to the engine at its own time and queues what must wait behind our input; true to drop it. A full
    /// queue lets it pass without the engine, which then lets go at the next turn.
    fn decide(&mut self, event: PointerEvent, stamp: Stamp) -> bool {
        if matches!(self.trust, Trust::Broken | Trust::Done) {
            return false;
        }
        if self.queue.full() {
            self.overflow();
            return false;
        }
        let (at, waited) = event_at(stamp.t0, stamp.tick, stamp.time);
        self.stats.waited(waited);
        let behind = self.behind(stamp.t0);
        let before = self.engine.deadline();
        self.lent.clear();
        let mut dropped = self.engine.on(event, at, &mut self.lent);
        self.trim();
        if behind
            && !dropped
            && let Some(again) = event.act()
        {
            // A move with no room left is dropped too; the next move, or a button's own point, takes its place.
            let _ = self.queue.put(again);
            dropped = true;
        }
        let cut = self.queue.extend(&self.lent);
        self.stats.overran(cut);
        if !self.queue.is_empty() || self.engine.deadline() != before {
            self.wake();
        }
        let took = self.os.now().saturating_since(stamp.t0).as_micros();
        self.stats.timed(u64::try_from(took).unwrap_or(u64::MAX));
        dropped
    }

    /// Drops and counts what the engine pushed past its `max_acts`; a debug build then fails as if it panicked.
    fn trim(&mut self) {
        let over = self.lent.len().saturating_sub(self.max);
        if over > 0 {
            self.stats.overran(over);
            self.lent.truncate(self.max);
        }
        debug_assert!(over == 0, "the pointer engine pushed past its max_acts");
    }

    /// True while our input is queued or not back yet; a wait past `echo_ms` ends and counts as a lost echo.
    fn behind(&mut self, now: Mono) -> bool {
        let wait = self.echo.check(now);
        if wait == Wait::Lost {
            self.stats.echo_lost();
        }
        wait == Wait::Waiting || !self.queue.is_empty()
    }

    /// The queue is full: events pass in order without the engine, whose view is then stale; counted once per overflow.
    fn overflow(&mut self) {
        if self.queue.overflow() {
            self.stats.echo_lost();
        }
        if self.trust == Trust::Whole {
            self.trust = Trust::Stale;
        }
        self.wake();
    }

    /// Asks for a turn, once until it runs.
    fn wake(&mut self) {
        if !self.woken {
            self.woken = self.os.wake();
        }
    }
}
