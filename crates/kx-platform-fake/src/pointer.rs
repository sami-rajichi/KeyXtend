//! A fake mouse hook: the test feeds events and settles on its own thread, and time comes only from the test.

use crate::lock;
use kx_module_api::Mono;
use kx_module_api::ports::{
    HookConfig, HookError, HookRun, HookStats, PointerAct, PointerEngine, PointerEvent, PointerHook,
};
use std::fmt;
use std::sync::{Arc, Mutex};

/// What a test bug hears when it drives the hook while no run is live.
const NO_RUN: &str = "no pointer engine runs: start the fake hook first";

/// What an engine hears when it pushes past its `max_acts` or reallocates the lent buffer.
const OVERRUN: &str = "the pointer engine pushed past its max_acts or reallocated the lent buffer";

/// What the hook did, and what the test set for it.
#[derive(Default)]
struct Log {
    /// The buffer lent to every engine call, reserved once per start.
    acts: Vec<PointerAct>,
    /// Its capacity at the start; any change is an engine bug.
    room: usize,
    /// The engine's `max_acts` at the start; one call pushing more is an engine bug, even within the buffer's slack.
    max_acts: usize,
    sent: Vec<PointerAct>,
    config: Option<HookConfig>,
    stats: HookStats,
    fail: Option<HookError>,
    starts: usize,
    stops: usize,
    wakes: usize,
    /// A run is held, from its start until the module drops it, even once its engine is given up.
    live: bool,
}

impl Log {
    /// Lends the buffer to one engine call, then sends what it pushed, in order.
    fn lend<R>(&mut self, call: impl FnOnce(&mut Vec<PointerAct>) -> R) -> R {
        let out = call(&mut self.acts);
        let over = self.acts.len() > self.max_acts || self.acts.capacity() != self.room;
        self.sent.append(&mut self.acts);
        if over {
            // A fresh buffer first, so the run can still stop after the test catches this.
            self.acts = Vec::with_capacity(self.room);
            panic!("{OVERRUN}");
        }
        out
    }
}

/// What the hook and its run share: the live engine, which is `Send` but not `Sync`, and the log.
#[derive(Default)]
struct Shared {
    engine: Mutex<Option<Box<dyn PointerEngine>>>,
    log: Mutex<Log>,
}

impl Shared {
    /// Takes the engine out and lets it go once, sending its acts; the caller drops it once the locks are free.
    fn release(&self) -> Option<Box<dyn PointerEngine>> {
        let mut engine = lock(&self.engine).take()?;
        lock(&self.log).lend(|acts| engine.release(acts));
        Some(engine)
    }
}

/// A pointer hook the test drives on its own thread, with no OS and no timer.
///
/// A call that needs a live run panics without one, as does an engine that pushes past its `max_acts`. Once the engine
/// is given up, events pass and nothing is called, as on the real host.
#[derive(Default)]
pub struct FakePointerHook {
    shared: Arc<Shared>,
}

impl FakePointerHook {
    /// A hook with no run, no error and default stats.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Refuses the next start that finds no live run with `error`.
    pub fn fail_next(&self, error: HookError) {
        lock(&self.shared.log).fail = Some(error);
    }

    /// Makes every run's `stats` return `stats`.
    pub fn set_stats(&self, stats: HookStats) {
        lock(&self.shared.log).stats = stats;
    }

    /// Runs `on` as the hook callback would and sends its acts, then `settle` only when the host would wake: for input,
    /// a moved deadline or notes. True when the event was dropped.
    #[must_use]
    pub fn feed(&self, event: PointerEvent, now: Mono) -> bool {
        let fed = self.drive(|engine, log| {
            let (before, sent) = (engine.deadline(), log.sent.len());
            let dropped = log.lend(|acts| engine.on(event, now, acts));
            let woken = log.sent.len() > sent || engine.deadline() != before;
            if woken || engine.has_notes() {
                log.lend(|acts| engine.settle(now, acts));
            }
            dropped
        });
        fed.unwrap_or(false)
    }

    /// Runs `settle` as a wake or a due deadline would, and sends its acts.
    pub fn settle(&self, now: Mono) {
        let _ = self.drive(|engine, log| log.lend(|acts| engine.settle(now, acts)));
    }

    /// When the live engine wants `settle` next.
    #[must_use]
    pub fn deadline(&self) -> Option<Mono> {
        self.drive(|engine, _| engine.deadline()).flatten()
    }

    /// Lets go mid-run as the host's reinstall or the panic hook's let-go does: `release` with its acts sent, and the run
    /// goes on. Then it settles at `now`, as the turn the host asks for would.
    pub fn let_go(&self, now: Mono) {
        let _ = self.drive(|engine, log| {
            log.lend(|acts| engine.release(acts));
            log.lend(|acts| engine.settle(now, acts));
        });
    }

    /// Gives up the engine as the host does after it panicked: one `release` with its acts sent, then the engine
    /// drops, while the run stays until the module drops it.
    ///
    /// # Panics
    /// Without a live run.
    pub fn give_up(&self) {
        assert!(lock(&self.shared.log).live, "{NO_RUN}");
        drop(self.shared.release());
    }

    /// Whether a run is live, from its start until the module drops it.
    #[must_use]
    pub fn running(&self) -> bool {
        lock(&self.shared.log).live
    }

    /// How many starts succeeded.
    #[must_use]
    pub fn starts(&self) -> usize {
        lock(&self.shared.log).starts
    }

    /// How many runs stopped.
    #[must_use]
    pub fn stops(&self) -> usize {
        lock(&self.shared.log).stops
    }

    /// How many wakes the runs got.
    #[must_use]
    pub fn wakes(&self) -> usize {
        lock(&self.shared.log).wakes
    }

    /// The config the latest start took, kept after it stops.
    #[must_use]
    pub fn config(&self) -> Option<HookConfig> {
        lock(&self.shared.log).config
    }

    /// Every act sent so far, oldest first.
    #[must_use]
    pub fn sent(&self) -> Vec<PointerAct> {
        lock(&self.shared.log).sent.clone()
    }

    /// Every act sent so far, oldest first, and forgets them.
    #[must_use]
    pub fn take_sent(&self) -> Vec<PointerAct> {
        std::mem::take(&mut lock(&self.shared.log).sent)
    }

    /// Runs `call` on the live engine and the log; `None` once the engine is given up.
    fn drive<R>(&self, call: impl FnOnce(&mut dyn PointerEngine, &mut Log) -> R) -> Option<R> {
        let mut slot = lock(&self.shared.engine);
        let mut log = lock(&self.shared.log);
        assert!(log.live, "{NO_RUN}");
        let engine = slot.as_mut()?;
        Some(call(engine.as_mut(), &mut log))
    }
}

impl fmt::Debug for FakePointerHook {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("FakePointerHook")
            .field("running", &self.running())
            .field("starts", &self.starts())
            .field("stops", &self.stops())
            .finish_non_exhaustive()
    }
}

impl PointerHook for FakePointerHook {
    fn start(
        &self,
        engine: Box<dyn PointerEngine>,
        config: HookConfig,
    ) -> Result<Box<dyn HookRun>, HookError> {
        let mut slot = lock(&self.shared.engine);
        let mut log = lock(&self.shared.log);
        if log.live {
            return Err(HookError::Busy);
        }
        if let Some(error) = log.fail.take() {
            return Err(error);
        }
        log.max_acts = engine.max_acts();
        log.acts = Vec::with_capacity(log.max_acts);
        log.room = log.acts.capacity();
        log.config = Some(config);
        log.starts += 1;
        log.live = true;
        *slot = Some(engine);
        let shared = Arc::clone(&self.shared);
        Ok(Box::new(FakeRun { shared }))
    }
}

/// A live fake run; dropping it releases the engine, unless it was given up, and stops.
struct FakeRun {
    shared: Arc<Shared>,
}

impl HookRun for FakeRun {
    /// Only counts, so the test decides when `settle` runs.
    fn wake(&self) {
        lock(&self.shared.log).wakes += 1;
    }

    fn stats(&self) -> HookStats {
        lock(&self.shared.log).stats
    }
}

impl Drop for FakeRun {
    fn drop(&mut self) {
        let gone = self.shared.release();
        let mut log = lock(&self.shared.log);
        log.live = false;
        log.stops += 1;
        drop(log);
        drop(gone);
    }
}

#[cfg(test)]
mod tests;
