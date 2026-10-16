//! Hook timing: the deadline timer's rounding, an event's own time, and a fixed histogram of callback times.

use std::sync::atomic::{AtomicU32, AtomicU64, Ordering};

use kx_module_api::{Mono, US_PER_MS};

/// One extra ms, so the deadline timer never fires early.
const ROUND_UP_MS: u64 = 1;
/// Each histogram bucket's width, in µs: fine enough to read a p99 well under 1 ms.
const BUCKET_US: u64 = 10;
/// Histogram buckets: 2 ms in steps of `BUCKET_US`; the last one also takes every longer callback.
const BUCKETS: usize = 200;
/// The percentile `p99` reads, out of `PERCENT`.
const PERCENTILE: u64 = 99;
/// The whole, for `PERCENTILE`.
const PERCENT: u64 = 100;

/// Milliseconds from `now` until `due`, plus one so the timer never fires early; a past deadline gives 1 ms.
pub(crate) fn timer_ms(due: Mono, now: Mono) -> u32 {
    let ms = due.as_us().saturating_sub(now.as_us()) / u64::from(US_PER_MS);
    u32::try_from(ms.saturating_add(ROUND_UP_MS)).unwrap_or(u32::MAX)
}

/// When an event stamped `time` happened, seen at `now` while Windows' tick count reads `tick`, and how many ms it
/// waited for the hook (P1's `event_time`). A stamp ahead of the tick count reads as no wait.
pub(crate) fn event_at(now: Mono, tick: u32, time: u32) -> (Mono, u32) {
    let waited = u32::try_from(tick.wrapping_sub(time).cast_signed()).unwrap_or(0);
    let us = u64::from(waited) * u64::from(US_PER_MS);
    (Mono::from_us(now.as_us().saturating_sub(us)), waited)
}

/// Callback times in fixed buckets of atomics, so recording never allocates, locks or waits.
pub(crate) struct Histogram {
    buckets: [AtomicU64; BUCKETS],
    max_us: AtomicU32,
}

impl Histogram {
    /// An empty histogram.
    pub(crate) const fn new() -> Self {
        Self {
            buckets: [const { AtomicU64::new(0) }; BUCKETS],
            max_us: AtomicU32::new(0),
        }
    }

    /// Counts one callback that took `us` µs.
    pub(crate) fn record(&self, us: u64) {
        let index = usize::try_from(us / BUCKET_US).map_or(BUCKETS - 1, |i| i.min(BUCKETS - 1));
        if let Some(bucket) = self.buckets.get(index) {
            bucket.fetch_add(1, Ordering::Relaxed);
        }
        let us = u32::try_from(us).unwrap_or(u32::MAX);
        self.max_us.fetch_max(us, Ordering::Relaxed);
    }

    /// The longest callback, in µs.
    pub(crate) fn max_us(&self) -> u32 {
        self.max_us.load(Ordering::Relaxed)
    }

    /// The 99th percentile in µs: the upper edge of its bucket, never above the longest callback; 0 with no samples.
    pub(crate) fn p99(&self) -> u32 {
        let counts = self.buckets.each_ref().map(|b| b.load(Ordering::Relaxed));
        let total: u64 = counts.iter().sum();
        let rank = total.saturating_mul(PERCENTILE).div_ceil(PERCENT);
        let mut seen = 0u64;
        for (index, count) in counts.iter().enumerate() {
            seen = seen.saturating_add(*count);
            if *count > 0 && seen >= rank {
                return edge(index).min(self.max_us());
            }
        }
        0
    }
}

/// The upper edge of bucket `index` in µs; the last bucket has none.
fn edge(index: usize) -> u32 {
    if index + 1 >= BUCKETS {
        return u32::MAX;
    }
    let edge = u64::try_from(index + 1).map_or(u64::MAX, |n| n.saturating_mul(BUCKET_US));
    u32::try_from(edge).unwrap_or(u32::MAX)
}
