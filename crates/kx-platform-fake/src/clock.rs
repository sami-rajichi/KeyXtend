//! A clock that moves only when the test says so.

use kx_module_api::{Clock, Mono};
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

/// A cloneable handle to one shared fake time.
#[derive(Clone, Debug)]
pub struct FakeClock(Arc<AtomicU64>);

impl FakeClock {
    /// A clock that reads `start` until it is moved.
    #[must_use]
    pub fn new(start: Mono) -> Self {
        Self(Arc::new(AtomicU64::new(start.as_us())))
    }

    /// Moves the time forward by `by`, clamped at the largest point.
    pub fn advance(&self, by: Duration) {
        let step = |us: u64| Some(Mono::from_us(us).saturating_add(by).as_us());
        // The step always returns `Some`, so the update cannot fail.
        let _ = self
            .0
            .fetch_update(Ordering::SeqCst, Ordering::SeqCst, step);
    }

    /// Jumps the time to `to`, forward or back.
    pub fn set(&self, to: Mono) {
        self.0.store(to.as_us(), Ordering::SeqCst);
    }
}

impl Clock for FakeClock {
    fn now(&self) -> Mono {
        Mono::from_us(self.0.load(Ordering::SeqCst))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_clock_stays_put_until_advanced() {
        let clock = FakeClock::new(Mono::from_us(50));
        assert_eq!(clock.now(), Mono::from_us(50));
        assert_eq!(clock.now(), Mono::from_us(50));
    }

    #[test]
    fn advance_moves_it_forward() {
        let clock = FakeClock::new(Mono::from_us(10));
        clock.advance(Duration::from_millis(2));
        assert_eq!(clock.now(), Mono::from_us(2_010));
        clock.advance(Duration::from_micros(5));
        assert_eq!(clock.now(), Mono::from_us(2_015));
    }

    #[test]
    fn set_jumps_forward_and_back() {
        let clock = FakeClock::new(Mono::from_us(100));
        clock.set(Mono::from_us(900));
        assert_eq!(clock.now(), Mono::from_us(900));
        clock.set(Mono::from_us(3));
        assert_eq!(clock.now(), Mono::from_us(3));
    }

    #[test]
    fn clones_share_one_time() {
        let a = FakeClock::new(Mono::default());
        let b = a.clone();
        a.advance(Duration::from_micros(7));
        assert_eq!(b.now(), Mono::from_us(7));
        b.set(Mono::from_us(40));
        assert_eq!(a.now(), Mono::from_us(40));
    }

    #[test]
    fn advance_near_the_end_saturates() {
        let clock = FakeClock::new(Mono::from_us(u64::MAX - 5));
        clock.advance(Duration::from_micros(10));
        assert_eq!(clock.now(), Mono::from_us(u64::MAX));
    }

    #[test]
    fn it_works_as_a_trait_object() {
        let clock = FakeClock::new(Mono::from_us(1));
        let dynamic: Arc<dyn Clock> = Arc::new(clock.clone());
        clock.advance(Duration::from_micros(4));
        assert_eq!(dynamic.now(), Mono::from_us(5));
    }
}
