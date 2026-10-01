//! The QPC time in microseconds for kx-gates, converted by the product's `ticks_to_us` so it matches kx-target-window's log.

use kx_platform_windows::ticks_to_us;
use windows::Win32::System::Performance::{QueryPerformanceCounter, QueryPerformanceFrequency};

/// Microseconds per millisecond: the product's `kx_module_api::US_PER_MS`, as a float.
#[allow(
    clippy::cast_lossless,
    reason = "`f64::from` is not const, and a u32 always fits an f64."
)]
pub const US_PER_MS: f64 = kx_module_api::US_PER_MS as f64;

/// `count` ticks at `freq` ticks per second, in microseconds, at most `i64::MAX`.
fn to_us(count: i64, freq: i64) -> i64 {
    i64::try_from(ticks_to_us(count, freq)).unwrap_or(i64::MAX)
}

/// QPC time in microseconds.
pub fn now_us() -> i64 {
    let (mut count, mut freq) = (0i64, 0i64);
    // SAFETY: plain queries into locals.
    unsafe {
        let _ = QueryPerformanceCounter(&mut count);
        let _ = QueryPerformanceFrequency(&mut freq);
    }
    to_us(count, freq)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ticks_convert_without_overflow() {
        assert_eq!(to_us(10_000_000, 10_000_000), 1_000_000);
        assert_eq!(to_us(i64::MAX, 10_000_000), i64::MAX / 10);
        assert_eq!(to_us(5, 0), 5_000_000);
        assert_eq!(to_us(i64::MAX, 1), i64::MAX);
    }

    #[test]
    fn now_us_is_on_the_product_clock() {
        use kx_module_api::Clock;
        let clock = kx_platform_windows::QpcClock::new().unwrap();
        let before = clock.now().as_us();
        let now = now_us().unsigned_abs();
        let after = clock.now().as_us();
        assert!(before <= now && now <= after, "{before} {now} {after}");
    }
}
