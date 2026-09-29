//! Monotonic time and the clock port that reads it.

use std::time::Duration;

/// Microseconds in one millisecond.
pub const US_PER_MS: u32 = 1_000;
/// Microseconds in one second.
pub const US_PER_S: u32 = 1_000_000;

/// A point in monotonic time: whole microseconds since an arbitrary start.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Mono(u64);

impl Mono {
    /// The point `us` microseconds after the start.
    #[must_use]
    pub const fn from_us(us: u64) -> Self {
        Self(us)
    }

    /// Microseconds since the start.
    #[must_use]
    pub const fn as_us(self) -> u64 {
        self.0
    }

    /// Time elapsed since `earlier`, or zero when `earlier` is later.
    #[must_use]
    pub const fn saturating_since(self, earlier: Self) -> Duration {
        Duration::from_micros(self.0.saturating_sub(earlier.0))
    }

    /// This point moved forward by `by`, clamped at the largest point.
    #[must_use]
    pub fn saturating_add(self, by: Duration) -> Self {
        let us = u64::try_from(by.as_micros()).unwrap_or(u64::MAX);
        Self(self.0.saturating_add(us))
    }
}

/// The source of monotonic time. The platform adapter gives the real clock; tests use a fake one.
pub trait Clock: Send + Sync {
    /// The current point in monotonic time.
    fn now(&self) -> Mono;
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    struct Fixed(Mono);

    impl Clock for Fixed {
        fn now(&self) -> Mono {
            self.0
        }
    }

    #[test]
    fn since_an_earlier_point_is_the_gap() {
        let (early, late) = (Mono::from_us(1_500), Mono::from_us(4_000));
        assert_eq!(late.saturating_since(early), Duration::from_micros(2_500));
    }

    #[test]
    fn since_the_same_point_is_zero() {
        let t = Mono::from_us(7);
        assert_eq!(t.saturating_since(t), Duration::ZERO);
    }

    #[test]
    fn since_a_later_point_is_zero() {
        let (early, late) = (Mono::from_us(1), Mono::from_us(9));
        assert_eq!(early.saturating_since(late), Duration::ZERO);
    }

    #[test]
    fn add_moves_forward_by_whole_microseconds() {
        let t = Mono::from_us(10).saturating_add(Duration::from_millis(2));
        assert_eq!(t.as_us(), 10 + u64::from(US_PER_MS) * 2);
    }

    #[test]
    fn add_near_the_top_clamps() {
        let t = Mono::from_us(u64::MAX - 5).saturating_add(Duration::from_micros(10));
        assert_eq!(t.as_us(), u64::MAX);
    }

    #[test]
    fn add_a_duration_beyond_u64_micros_clamps() {
        let t = Mono::default().saturating_add(Duration::from_secs(u64::MAX));
        assert_eq!(t.as_us(), u64::MAX);
    }

    #[test]
    fn units_relate_by_a_thousand() {
        assert_eq!(US_PER_MS, 1_000);
        assert_eq!(US_PER_S / US_PER_MS, 1_000);
    }

    #[test]
    fn a_clock_is_usable_as_a_trait_object() {
        let clock: &dyn Clock = &Fixed(Mono::from_us(42));
        assert_eq!(clock.now(), Mono::from_us(42));
    }
}
