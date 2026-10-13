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
    sent: Vec<PointerAct>,
    config: Option<HookConfig>,
    stats: HookStats,
    fail: Option<HookError>,
    starts: usize,
    stops: usize,
    wakes: usize,
}

impl Log {
    /// Lends the buffer to one engine call, then sends what it pushed, in order.
    fn lend<R>(&mut self, call: impl FnOnce(&mut Vec<PointerAct>) -> R) -> R {
        let out = call(&mut self.acts);
        let grew = self.acts.capacity() != self.room;
        self.sent.append(&mut self.acts);
        if grew {
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

/// A pointer hook the test drives on its own thread, with no OS and no timer.
///
/// A call that needs a live engine panics without one, as does an engine that pushes past its `max_acts`.
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

    /// Runs `on` as the hook callback would, sends its acts, then runs `settle`; true when the event was dropped.
    #[must_use]
    pub fn feed(&self, event: PointerEvent, now: Mono) -> bool {
        self.drive(|engine, log| {
            let dropped = log.lend(|acts| engine.on(event, now, acts));
            log.lend(|acts| engine.settle(now, acts));
            dropped
        })
    }

    /// Runs `settle` as a wake or a due deadline would, and sends its acts.
    pub fn settle(&self, now: Mono) {
        self.drive(|engine, log| log.lend(|acts| engine.settle(now, acts)));
    }

    /// When the live engine wants `settle` next.
    #[must_use]
    pub fn deadline(&self) -> Option<Mono> {
        self.drive(|engine, _| engine.deadline())
    }

    /// Whether a run is live.
    #[must_use]
    pub fn running(&self) -> bool {
        lock(&self.shared.engine).is_some()
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

    /// Runs `call` on the live engine and the log.
    fn drive<R>(&self, call: impl FnOnce(&mut dyn PointerEngine, &mut Log) -> R) -> R {
        let mut live = lock(&self.shared.engine);
        let Some(engine) = live.as_mut() else {
            panic!("{NO_RUN}");
        };
        call(engine.as_mut(), &mut lock(&self.shared.log))
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
        let mut live = lock(&self.shared.engine);
        if live.is_some() {
            return Err(HookError::Busy);
        }
        let mut log = lock(&self.shared.log);
        if let Some(error) = log.fail.take() {
            return Err(error);
        }
        log.acts = Vec::with_capacity(engine.max_acts());
        log.room = log.acts.capacity();
        log.config = Some(config);
        log.starts += 1;
        *live = Some(engine);
        let shared = Arc::clone(&self.shared);
        Ok(Box::new(FakeRun { shared }))
    }
}

/// A live fake run; dropping it releases the engine and stops.
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
        let Some(mut engine) = lock(&self.shared.engine).take() else {
            return;
        };
        let mut log = lock(&self.shared.log);
        log.lend(|acts| engine.release(acts));
        log.stops += 1;
    }
}

#[cfg(test)]
mod tests;
