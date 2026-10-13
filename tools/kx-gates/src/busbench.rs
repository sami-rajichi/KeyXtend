//! G8's bus publish: the kernel's own bus with as many handlers as a full app has modules, timed one publish at a time.
//! It runs in kx-gates, so in weak mode kx-gates first joins the product's capped job.
#![cfg(windows)]

use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Instant;

use kx_kernel::bus::KernelBus;
use kx_module_api::{Bus, Event};
use serde::Serialize;

use crate::stats::{P50, P99, percentile};

/// Nanoseconds in one microsecond.
const NS_PER_US: f64 = 1_000.0;

/// The timed event.
struct Tick;

impl Event for Tick {
    const NAME: &'static str = "kx-gates-tick";
}

/// What the publishes took.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct BusTimes {
    /// Publishes timed.
    pub publishes: usize,
    /// Handler calls in all; every handler hears every publish.
    pub heard: usize,
    /// The typical publish, in µs.
    pub p50_us: Option<f64>,
    /// The 99th percentile, in µs.
    pub p99_us: Option<f64>,
}

/// Times `publishes` publishes to `subscribers` handlers on a fresh kernel bus.
pub fn time(subscribers: usize, publishes: usize) -> Result<BusTimes, String> {
    let bus = Bus::new(Arc::new(KernelBus::new()));
    let heard = Arc::new(AtomicUsize::new(0));
    let mut subs = Vec::with_capacity(subscribers);
    for _ in 0..subscribers {
        let count = Arc::clone(&heard);
        let on = move |_: &Tick| {
            count.fetch_add(1, Ordering::Relaxed);
        };
        subs.push(bus.subscribe(on).map_err(|e| e.to_string())?);
    }
    let mut us = Vec::with_capacity(publishes);
    for _ in 0..publishes {
        let start = Instant::now();
        bus.publish(&Tick).map_err(|e| e.to_string())?;
        #[allow(
            clippy::cast_precision_loss,
            reason = "One publish is far below 2^53 ns."
        )]
        us.push(start.elapsed().as_nanos() as f64 / NS_PER_US);
    }
    drop(subs);
    us.sort_by(f64::total_cmp);
    Ok(BusTimes {
        publishes,
        heard: heard.load(Ordering::Relaxed),
        p50_us: percentile(&us, P50),
        p99_us: percentile(&us, P99),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_handler_hears_every_timed_publish() {
        let t = time(3, 10).expect("the bus takes handlers");
        assert_eq!((t.publishes, t.heard), (10, 30));
        let (p50, p99) = (t.p50_us.expect("timed"), t.p99_us.expect("timed"));
        assert!(p50 >= 0.0 && p99 >= p50, "{p50} {p99}");
    }

    #[test]
    fn no_publish_times_nothing() {
        let t = time(2, 0).expect("runs");
        assert_eq!((t.heard, t.p99_us), (0, None));
    }
}
