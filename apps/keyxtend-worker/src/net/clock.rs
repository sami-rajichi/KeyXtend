//! The speech job's time: now, and a wait, so the pacing within `per_min` and the wait after a 429 are tested with a
//! still clock and never sleep.

use std::thread;
use std::time::{Duration, Instant};

/// The job's time, shared by its request threads.
pub trait Clock: Send + Sync {
    /// Milliseconds since the job started.
    fn now_ms(&self) -> u64;

    /// Waits `ms` milliseconds.
    fn wait(&self, ms: u64);
}

/// The real clock, from the job's start.
pub struct Live(Instant);

impl Live {
    /// A clock starting now.
    pub fn new() -> Self {
        Self(Instant::now())
    }
}

impl Clock for Live {
    fn now_ms(&self) -> u64 {
        u64::try_from(self.0.elapsed().as_millis()).unwrap_or(u64::MAX)
    }

    fn wait(&self, ms: u64) {
        thread::sleep(Duration::from_millis(ms));
    }
}
