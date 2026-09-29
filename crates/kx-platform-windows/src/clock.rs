//! The QPC clock: Windows' performance counter in microseconds, the time base the test tools log.

use kx_module_api::US_PER_S;

#[cfg(windows)]
pub use qpc::QpcClock;

/// `ticks` of a counter running at `freq` ticks per second, in whole microseconds, rounded down.
/// A frequency below one counts as one, negative ticks as zero, and the result stops at `u64::MAX`.
#[must_use]
pub fn ticks_to_us(ticks: i64, freq: i64) -> u64 {
    let ticks = u128::try_from(ticks).unwrap_or(0);
    let freq = u128::try_from(freq).unwrap_or(0).max(1);
    // `i64::MAX` ticks times a million still fits in 128 bits.
    u64::try_from(ticks * u128::from(US_PER_S) / freq).unwrap_or(u64::MAX)
}

#[cfg(windows)]
mod qpc {
    use kx_module_api::{Clock, Mono};
    use windows::Win32::System::Performance::{QueryPerformanceCounter, QueryPerformanceFrequency};

    use super::ticks_to_us;
    use crate::PlatformError;

    /// The monotonic clock: microseconds since Windows started, the same count `kx-gates` logs.
    #[derive(Clone, Copy, Debug)]
    pub struct QpcClock {
        /// Counter ticks per second, fixed when Windows starts.
        freq: i64,
    }

    impl QpcClock {
        /// Reads the counter's frequency once, since Windows fixes it at start-up.
        ///
        /// # Errors
        /// `PlatformError::Clock` when Windows gives no frequency.
        #[allow(unsafe_code, reason = "QueryPerformanceFrequency into a local.")]
        pub fn new() -> Result<Self, PlatformError> {
            let mut freq = 0;
            // SAFETY: the call writes one i64 into a local that outlives it.
            unsafe { QueryPerformanceFrequency(&raw mut freq) }
                .map_err(|e| PlatformError::Clock(e.into()))?;
            Ok(Self { freq })
        }
    }

    impl Clock for QpcClock {
        fn now(&self) -> Mono {
            Mono::from_us(ticks_to_us(counter(), self.freq))
        }
    }

    /// The counter's current tick; Windows XP and later never fail this call.
    #[allow(unsafe_code, reason = "QueryPerformanceCounter into a local.")]
    fn counter() -> i64 {
        let mut ticks = 0;
        // SAFETY: the call writes one i64 into a local that outlives it.
        let _ = unsafe { QueryPerformanceCounter(&raw mut ticks) };
        ticks
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn the_frequency_is_positive() {
            assert!(QpcClock::new().unwrap().freq > 0);
        }

        #[test]
        fn the_clock_reads_the_counter_in_microseconds() {
            let clock = QpcClock::new().unwrap();
            let before = ticks_to_us(counter(), clock.freq);
            let now = clock.now().as_us();
            let after = ticks_to_us(counter(), clock.freq);
            assert!(before <= now && now <= after, "{before} {now} {after}");
        }

        #[test]
        fn the_clock_never_runs_backwards_and_works_as_a_trait_object() {
            let clock: std::sync::Arc<dyn Clock> = std::sync::Arc::new(QpcClock::new().unwrap());
            let first = clock.now();
            assert!(clock.now() >= first);
            assert!(first > Mono::default());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    /// The frequency Windows 10 and 11 report on most PCs.
    const TEN_MHZ: i64 = 10_000_000;
    /// An odd frequency: the ACPI power-management timer older PCs use.
    const ACPI_PM: i64 = 3_579_545;

    #[test]
    fn ten_mhz_ticks_are_tenths_of_a_microsecond() {
        assert_eq!(ticks_to_us(TEN_MHZ, TEN_MHZ), u64::from(US_PER_S));
        assert_eq!(ticks_to_us(12_345, TEN_MHZ), 1_234);
        assert_eq!(ticks_to_us(9, TEN_MHZ), 0);
    }

    #[test]
    fn an_odd_frequency_rounds_down() {
        assert_eq!(ticks_to_us(ACPI_PM, ACPI_PM), u64::from(US_PER_S));
        // 1000 ticks at 3.579545 MHz are 279.365… µs.
        assert_eq!(ticks_to_us(1_000, ACPI_PM), 279);
        assert_eq!(ticks_to_us(7, 3), 2_333_333);
    }

    #[test]
    fn very_large_counts_do_not_overflow() {
        let max = i64::MAX.unsigned_abs();
        assert_eq!(ticks_to_us(i64::MAX, TEN_MHZ), max / 10);
        assert_eq!(ticks_to_us(i64::MAX, ACPI_PM), 2_576_688_388_288_113_658);
        assert_eq!(ticks_to_us(i64::MAX, 1), u64::MAX);
    }

    #[test]
    fn a_bad_frequency_counts_as_one_and_negative_ticks_as_zero() {
        assert_eq!(ticks_to_us(5, 0), 5 * u64::from(US_PER_S));
        assert_eq!(ticks_to_us(5, -3), 5 * u64::from(US_PER_S));
        assert_eq!(ticks_to_us(-5, TEN_MHZ), 0);
    }

    proptest! {
        #[test]
        fn time_never_runs_backwards(a in 0..i64::MAX, b in 0..i64::MAX, freq in 1..i64::MAX) {
            let (early, late) = (a.min(b), a.max(b));
            prop_assert!(ticks_to_us(early, freq) <= ticks_to_us(late, freq));
        }

        #[test]
        fn whole_seconds_convert_exactly(secs in 0..1_000_000i64, freq in 1..TEN_MHZ * 10) {
            prop_assert_eq!(ticks_to_us(secs * freq, freq), secs.unsigned_abs() * u64::from(US_PER_S));
        }
    }
}
