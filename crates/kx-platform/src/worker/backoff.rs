//! The wait before a failed worker starts again, shared by the call and stream hosts: it starts at `restart_ms`,
//! doubles with each failure in a row up to `restart_max_ms`, and goes back to its first wait after a good answer.

use super::{Lost, Times};
use kx_module_api::Mono;
use std::time::Duration;

/// When the next start may come, and how long the next failure holds it back.
pub(super) struct Backoff {
    /// The first wait, never past the cap.
    first: Duration,
    /// The longest wait.
    cap: Duration,
    /// No start before this point.
    calm: Mono,
    /// The wait after the next failure.
    pause: Duration,
}

impl Backoff {
    /// A backoff with `times`' waits that holds nothing back yet.
    pub(super) fn new(times: Times) -> Self {
        let cap = ms(times.restart_max_ms);
        let first = ms(times.restart_ms).min(cap);
        Self {
            first,
            cap,
            calm: Mono::default(),
            pause: first,
        }
    }

    /// True while a start must wait.
    pub(super) fn resting(&self, now: Mono) -> bool {
        now < self.calm
    }

    /// The refusal of a start at `now`: how long until one may come.
    pub(super) fn down(&self, now: Mono) -> Lost {
        Lost::Down {
            retry_in: self.calm.saturating_since(now),
        }
    }

    /// Holds the next start back from `now` by the current wait, which then doubles up to its cap.
    pub(super) fn hold(&mut self, now: Mono) {
        self.calm = now.saturating_add(self.pause);
        self.pause = self.pause.saturating_mul(2).min(self.cap);
    }

    /// Notes a good answer: the next failure waits the first wait again.
    pub(super) fn reset(&mut self) {
        self.pause = self.first;
    }

    /// Takes `times`' waits from the next failure on; a start already held back stays held.
    pub(super) fn retime(&mut self, times: Times) {
        *self = Self {
            calm: self.calm,
            ..Self::new(times)
        };
    }
}

/// `n` milliseconds.
pub(crate) fn ms(n: u32) -> Duration {
    Duration::from_millis(u64::from(n))
}
