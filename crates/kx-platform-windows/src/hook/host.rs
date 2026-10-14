//! The host on the hook thread (ADR-0027 decisions 2–8): the engine decides each event inside the callback, passed
//! input waits behind our own, the loop sends the engine's input, lagging hooks go back and the stop lets go. It is
//! generic over `Os`, so a fake drives it in tests.

mod hand;
mod run;
mod wait;

use std::panic::{AssertUnwindSafe, catch_unwind};
use std::sync::Arc;

use kx_module_api::Mono;
use kx_module_api::ports::{HookConfig, Point, PointerAct, PointerEngine, PointerEvent, Scan};

use super::accept::{Take, esc, kind, take, user_key};
use super::echo::Echo;
use super::health::Health;
use super::held::{Held, LET_GO};
use super::queue::Queue;
use super::stats::Stats;
use super::timing::event_at;
use super::{Mark, Os, Source};
use crate::keys::{is_replay, replay_id};

/// Batches that may be out at once before the oldest acts make room.
const FLIGHTS: usize = 2;

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
    /// A turn's input, which goes out as one batch; reserved once for owed Ups, a queue, a release, a let-go and a
    /// settle.
    out: Vec<PointerAct>,
    echo: Echo,
    health: Health,
    stats: Arc<Stats>,
    /// What our sent input holds down, and the hand's presses whose release the engine dropped, for the host's own
    /// let-go.
    held: Held,
    /// The hand's own presses that reached the OS untouched and are not let go yet.
    hand: Held,
    /// Presses the next batch owes an Up first: the hand let them go before ours was proven ahead.
    owed: Held,
    /// The newest point the hook saw.
    at: Option<Point>,
    trust: Trust,
    /// The deadline the timer is set for.
    armed: Option<Mono>,
    /// The timer fired or Windows refused it, so the next turn sets or stops it whatever the deadline.
    rearm: bool,
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
    /// It panicked and had its one release; the host dropped it.
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
        let queue = Queue::new(max);
        let out = queue
            .room()
            .saturating_add(max.saturating_mul(2))
            .saturating_add(LET_GO.saturating_mul(2));
        Self {
            lent: Vec::with_capacity(max),
            max,
            queue,
            out: Vec::with_capacity(out),
            echo: Echo::new(cfg.echo_ms, out.saturating_mul(FLIGHTS)),
            health: Health::new(cfg, seen),
            os,
            engine,
            source,
            stats,
            held: Held::default(),
            hand: Held::default(),
            owed: Held::default(),
            at: None,
            trust: Trust::Whole,
            armed: None,
            rearm: false,
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
    /// the hand's event is noted for the queue and the record of held input.
    pub(crate) fn mouse(&mut self, msg: u32, at: Point, mark: Mark) -> bool {
        self.at = Some(at);
        let queued = !self.queue.is_empty();
        let dropped = self.guard(|h| h.on_mouse(msg, at, mark)).unwrap_or(false);
        let heard = take(self.source, mark.injected, mark.extra) == Take::Engine;
        if let (true, Some(kind)) = (heard, kind(msg)) {
            self.heard(kind, at, queued, dropped);
        }
        dropped
    }

    /// Hears key message `msg` for `vk`, whose scan code reads as `scan`: an Esc press reaches the engine, and every
    /// key passes on.
    pub(crate) fn key(&mut self, msg: u32, vk: u32, scan: Scan, mark: Mark) {
        let _ = self.guard(|h| h.on_key(msg, vk, mark));
        if let Some(act) = user_key(self.source, msg, scan, mark.injected, mark.extra) {
            self.hand_key(act);
        }
    }

    /// A turn, on a wake.
    pub(crate) fn turn(&mut self) {
        let _ = self.guard(Self::run_turn);
    }

    /// The deadline timer fired and repeats, so a turn sets it again for a deadline still to come, or stops it.
    pub(crate) fn fired(&mut self) {
        self.rearm = true;
        self.turn();
    }

    /// The health timer: a turn, then the health check.
    pub(crate) fn check(&mut self) {
        let _ = self.guard(Self::run_check);
    }

    /// The panic hook's let-go: sends the queue, the engine's release and what our input still holds; the hooks stay.
    /// It asks for a turn, so the engine's notes and its idle state go out at once.
    pub(crate) fn release(&mut self) {
        let _ = self.guard(Self::run_release);
        self.wake();
    }

    /// Sends the queue, the engine's release and what our input still holds, and only then stops the timer and removes
    /// the hooks, whatever failed.
    pub(crate) fn stop(&mut self) {
        let _ = self.guard(Self::finish);
        self.os.arm(None);
        self.os.unhook();
    }

    /// The room of the lent buffer, the queue and the batch.
    #[cfg(test)]
    pub(crate) fn room(&self) -> (usize, usize, usize) {
        (self.lent.capacity(), self.queue.room(), self.out.capacity())
    }

    /// Runs `call`; a panic marks the engine broken instead of reaching Windows. Only that fall asks for the turn that
    /// lets go, so a fault that recurs never loops.
    fn guard<T>(&mut self, call: impl FnOnce(&mut Self) -> T) -> Option<T> {
        let out = catch_unwind(AssertUnwindSafe(|| call(self))).ok();
        if out.is_none() {
            self.lent.clear();
            if matches!(self.trust, Trust::Whole | Trust::Stale) {
                self.trust = Trust::Broken;
                self.wake();
            }
        }
        out
    }

    fn on_mouse(&mut self, msg: u32, at: Point, mark: Mark) -> bool {
        let stamp = self.stamp(mark);
        match (take(self.source, mark.injected, mark.extra), kind(msg)) {
            (Take::Echo, _) => {
                if is_replay(mark.extra) {
                    self.echo.back(replay_id(mark.extra));
                }
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
        if is_replay(mark.extra) {
            self.echo.back(replay_id(mark.extra));
        }
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
        self.stats.drop_acts(cut);
        self.ask_turn(before);
        let took = self.os.now().saturating_since(stamp.t0).as_micros();
        self.stats.timed(u64::try_from(took).unwrap_or(u64::MAX));
        dropped
    }

    /// Drops and counts what the engine pushed past its `max_acts`; a debug build then fails as if it panicked.
    fn trim(&mut self) {
        let over = self.lent.len().saturating_sub(self.max);
        if over > 0 {
            self.stats.drop_acts(over);
            self.lent.truncate(self.max);
        }
        debug_assert!(over == 0, "the pointer engine pushed past its max_acts");
    }

    /// Asks for a turn, once until it runs.
    fn wake(&mut self) {
        if !self.woken {
            self.woken = self.os.wake();
        }
    }
}
