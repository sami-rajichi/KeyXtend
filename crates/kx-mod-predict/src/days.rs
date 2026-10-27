//! The day number the learner stamps on what it learns and purges by: UTC days since the Unix epoch, injected so tests
//! never read the real calendar.

use std::time::{SystemTime, UNIX_EPOCH};

/// Seconds in a UTC day.
const S_PER_DAY: u64 = 86_400;

/// Tells today's day number.
pub trait Days: Send + Sync {
    /// Today's day number.
    fn today(&self) -> u32;
}

/// The system calendar: UTC days since the Unix epoch.
#[derive(Debug, Default, Clone, Copy)]
pub struct SystemDays;

impl Days for SystemDays {
    fn today(&self) -> u32 {
        day_at(SystemTime::now())
    }
}

/// The UTC day holding `time`; a clock set before the epoch reads day 0.
fn day_at(time: SystemTime) -> u32 {
    day_of(time.duration_since(UNIX_EPOCH).map_or(0, |d| d.as_secs()))
}

/// The day holding `secs` seconds after the epoch; far ahead stops at the top.
fn day_of(secs: u64) -> u32 {
    u32::try_from(secs / S_PER_DAY).unwrap_or(u32::MAX)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    /// 2024-02-29T00:00:00Z, a leap day.
    const LEAP_S: u64 = 1_709_164_800;
    /// Its UTC day number.
    const LEAP_DAY: u32 = 19_782;

    /// The time `secs` seconds after the epoch.
    fn at(secs: u64) -> SystemTime {
        UNIX_EPOCH + Duration::from_secs(secs)
    }

    #[test]
    fn a_day_starts_at_utc_midnight() {
        assert_eq!(day_at(UNIX_EPOCH), 0);
        assert_eq!(day_at(at(S_PER_DAY - 1)), 0);
        assert_eq!(day_at(at(S_PER_DAY)), 1);
    }

    #[test]
    fn a_leap_day_is_a_whole_day_of_its_own() {
        assert_eq!(day_at(at(LEAP_S - 1)), LEAP_DAY - 1);
        assert_eq!(day_at(at(LEAP_S)), LEAP_DAY);
        assert_eq!(day_at(at(LEAP_S + S_PER_DAY - 1)), LEAP_DAY);
        assert_eq!(day_at(at(LEAP_S + S_PER_DAY)), LEAP_DAY + 1);
    }

    #[test]
    fn a_clock_before_the_epoch_reads_day_zero() {
        assert_eq!(day_at(UNIX_EPOCH - Duration::from_secs(1)), 0);
        assert_eq!(day_at(UNIX_EPOCH - Duration::from_secs(S_PER_DAY * 400)), 0);
    }

    #[test]
    fn far_ahead_stops_at_the_top() {
        assert_eq!(day_of(u64::MAX), u32::MAX);
    }
}
