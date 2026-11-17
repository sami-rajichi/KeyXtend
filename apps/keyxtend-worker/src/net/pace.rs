//! A run's pace: the start of every request it sends, first tries, tries again and the re-send after a 429 alike, so
//! no minute holds more than `per_min` of them. A run's threads share one.

use std::collections::VecDeque;

/// The span `per_min` counts requests in, in ms.
const MINUTE_MS: u64 = 60_000;

/// When a run's requests started or will start.
pub struct Pace {
    per_min: usize,
    /// Each start, oldest first; one may lie ahead, booked by a request still waiting its turn.
    starts: VecDeque<u64>,
}

impl Pace {
    /// A pace of at most `per_min` starts a minute.
    pub fn new(per_min: usize) -> Self {
        Self {
            per_min,
            starts: VecDeque::new(),
        }
    }

    /// Books the next start at or after `now` and gives the ms to wait for it.
    pub fn turn(&mut self, now: u64) -> u64 {
        while self
            .starts
            .front()
            .is_some_and(|&s| s.saturating_add(MINUTE_MS) <= now)
        {
            self.starts.pop_front();
        }
        let last = self.starts.back().copied().unwrap_or(now);
        let full = self.starts.len().checked_sub(self.per_min);
        let free = full
            .and_then(|at| self.starts.get(at))
            .map_or(now, |s| s.saturating_add(MINUTE_MS));
        let at = now.max(last).max(free);
        self.starts.push_back(at);
        at - now
    }
}
