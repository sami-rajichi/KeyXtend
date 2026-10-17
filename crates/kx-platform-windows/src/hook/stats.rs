//! A run's numbers, published through atomics so `stats` never blocks a callback; never a point or a key.

use std::sync::atomic::{AtomicU32, AtomicU64, Ordering};

use kx_module_api::ports::HookStats;

use super::timing::Histogram;

/// The counters the hook thread writes and any thread reads.
pub(crate) struct Stats {
    events: AtomicU64,
    times: Histogram,
    max_wait_ms: AtomicU32,
    reinstalls: AtomicU32,
    send_errors: AtomicU32,
    lost_echoes: AtomicU32,
    passed_inside: AtomicU32,
    /// Acts dropped for want of room; logged at the stop, as the port has no field for it.
    dropped: AtomicU32,
}

impl Stats {
    /// All counters at zero.
    pub(crate) const fn new() -> Self {
        Self {
            events: AtomicU64::new(0),
            times: Histogram::new(),
            max_wait_ms: AtomicU32::new(0),
            reinstalls: AtomicU32::new(0),
            send_errors: AtomicU32::new(0),
            lost_echoes: AtomicU32::new(0),
            passed_inside: AtomicU32::new(0),
            dropped: AtomicU32::new(0),
        }
    }

    /// Counts an event the engine decided, whose callback took `us` µs.
    pub(crate) fn timed(&self, us: u64) {
        self.events.fetch_add(1, Ordering::Relaxed);
        self.times.record(us);
    }

    /// Keeps the longest time an event waited before the hook saw it.
    pub(crate) fn waited(&self, ms: u32) {
        self.max_wait_ms.fetch_max(ms, Ordering::Relaxed);
    }

    /// Counts the hooks put back.
    pub(crate) fn reinstalled(&self) {
        bump(&self.reinstalls);
    }

    /// Counts a batch the OS refused, in whole or in part.
    pub(crate) fn send_failed(&self) {
        bump(&self.send_errors);
    }

    /// Counts our input not coming back in time, or a queue too full to keep waiting.
    pub(crate) fn echo_lost(&self) {
        bump(&self.lost_echoes);
    }

    /// Counts `n` of the user's events heard inside our send, which passed without the host.
    pub(crate) fn passed_inside(&self, n: u32) {
        add(&self.passed_inside, n);
    }

    /// Counts `n` acts dropped for want of room: past an engine's `max_acts`, or past the queue's.
    pub(crate) fn drop_acts(&self, n: usize) {
        add(&self.dropped, u32::try_from(n).unwrap_or(u32::MAX));
    }

    /// The acts dropped so far for want of room.
    pub(crate) fn dropped(&self) -> u32 {
        self.dropped.load(Ordering::Relaxed)
    }

    /// The numbers so far.
    pub(crate) fn snapshot(&self) -> HookStats {
        let load = |n: &AtomicU32| n.load(Ordering::Relaxed);
        HookStats {
            events: self.events.load(Ordering::Relaxed),
            p99_us: self.times.p99(),
            max_us: self.times.max_us(),
            max_wait_ms: load(&self.max_wait_ms),
            reinstalls: load(&self.reinstalls),
            send_errors: load(&self.send_errors),
            lost_echoes: load(&self.lost_echoes),
            passed_inside: load(&self.passed_inside),
        }
    }
}

impl Default for Stats {
    fn default() -> Self {
        Self::new()
    }
}

/// Adds one to `n`, stopping at its largest value.
fn bump(n: &AtomicU32) {
    add(n, 1);
}

/// Adds `by` to `n`, stopping at its largest value.
fn add(n: &AtomicU32, by: u32) {
    if by > 0 {
        let _ = n.fetch_update(Ordering::Relaxed, Ordering::Relaxed, |v| {
            Some(v.saturating_add(by))
        });
    }
}
