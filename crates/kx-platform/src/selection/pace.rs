//! The pacing of tells, pure on the `Mono` points the thread feeds it: a burst of changes is told once, `settle_ms`
//! after its last change, and at most `looks_per_s` tells fall in any second; a later one waits for the window's end,
//! so none is dropped and the newest change wins.

use std::collections::VecDeque;
use std::time::Duration;

use kx_module_api::Mono;
use kx_module_api::ports::SelConfig;

/// The span `looks_per_s` counts tells in: one second.
const WINDOW: Duration = Duration::from_secs(1);

/// When the watch tells: the change it still owes, and when its latest tells were.
#[derive(Clone, Debug)]
pub(super) struct Pace {
    /// How long a change waits for the next one before it is told.
    settle: Duration,
    /// The most tells in any window; at least one.
    cap: usize,
    /// When the newest change not yet told came; `None` while idle.
    owed: Option<Mono>,
    /// When the latest tells were, oldest first, never more than `cap`.
    told: VecDeque<Mono>,
}

impl Pace {
    /// Pacing as `cfg` says; a cap of 0 tells once a window, so nothing is ever dropped.
    pub(super) fn new(cfg: SelConfig) -> Self {
        Self {
            settle: Duration::from_millis(u64::from(cfg.settle_ms)),
            cap: usize::from(cfg.looks_per_s.max(1)),
            owed: None,
            told: VecDeque::new(),
        }
    }

    /// A change at `now`; an earlier point never moves the newest change back.
    pub(super) fn change(&mut self, now: Mono) {
        self.owed = Some(self.owed.map_or(now, |was| was.max(now)));
    }

    /// When the change owed is due, settled and inside the cap; `None` while idle, so the thread waits for an event.
    pub(super) fn due(&self) -> Option<Mono> {
        let settled = self.owed?.saturating_add(self.settle);
        let full = self.told.len() >= self.cap;
        let free = self.told.front().filter(|_| full);
        Some(free.map_or(settled, |t| t.saturating_add(WINDOW).max(settled)))
    }

    /// Whether to tell at `now`: true once the change owed is due, which is then told and counted.
    pub(super) fn tell(&mut self, now: Mono) -> bool {
        if self.due().is_none_or(|due| due > now) {
            return false;
        }
        self.owed = None;
        if self.told.len() >= self.cap {
            self.told.pop_front();
        }
        self.told.push_back(now);
        true
    }
}

#[cfg(test)]
mod tests;
