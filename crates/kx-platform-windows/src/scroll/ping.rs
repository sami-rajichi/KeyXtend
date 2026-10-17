//! The target's pings (ADR-0029): a window that leaves a ping unanswered gets no turn or read until it answers, as
//! Windows marks an app hung only after seconds. Pure; `win` sends the pings from a thread of the run's own.

use std::time::{Duration, Instant};

/// The ping out and how the last one ended.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub struct Pings {
    /// The window asked and when, while its answer is due.
    out: Option<(u64, Instant)>,
    /// The window last asked, and whether it answered in time.
    last: Option<(u64, bool)>,
}

impl Pings {
    /// Starts a ping to `window` at `now`; false while another is out, as one goes at a time.
    pub fn ask(&mut self, window: u64, now: Instant) -> bool {
        if self.out.is_some() {
            return false;
        }
        self.out = Some((window, now));
        true
    }

    /// The ping out ended, `answered` in time or not.
    pub fn done(&mut self, answered: bool) {
        if let Some((window, _)) = self.out.take() {
            self.last = Some((window, answered));
        }
    }

    /// True while a ping is out.
    #[must_use]
    pub const fn out(&self) -> bool {
        self.out.is_some()
    }

    /// True while `window` owes a ping for longer than `wait`, or left its last one unanswered.
    #[must_use]
    pub fn busy(&self, window: u64, now: Instant, wait: Duration) -> bool {
        let late =
            |(w, at): (u64, Instant)| w == window && now.saturating_duration_since(at) > wait;
        self.out.is_some_and(late) || self.last == Some((window, false))
    }
}

#[cfg(test)]
mod tests;
