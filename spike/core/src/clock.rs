//! The QPC clock in microseconds, shared by target-window and the harness so G2 times match.

use windows::Win32::System::Performance::{QueryPerformanceCounter, QueryPerformanceFrequency};

/// Microseconds per second.
const US: i128 = 1_000_000;

/// `count` ticks at `freq` ticks per second, in microseconds.
fn to_us(count: i64, freq: i64) -> i64 {
    (i128::from(count) * US / i128::from(freq.max(1))) as i64
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
    }
}
