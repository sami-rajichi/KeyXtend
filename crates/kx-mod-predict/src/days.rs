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
        let since = SystemTime::now().duration_since(UNIX_EPOCH);
        day_of(since.map_or(0, |d| d.as_secs()))
    }
}

/// The day holding `secs` seconds after the epoch; a clock set before it reads day 0.
fn day_of(secs: u64) -> u32 {
    u32::try_from(secs / S_PER_DAY).unwrap_or(u32::MAX)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_day_starts_at_utc_midnight() {
        assert_eq!(day_of(0), 0);
        assert_eq!(day_of(S_PER_DAY - 1), 0);
        assert_eq!(day_of(S_PER_DAY), 1);
        assert_eq!(day_of(u64::MAX), u32::MAX, "far ahead stops at the top");
    }

    #[test]
    fn the_system_calendar_is_past_the_designs_date() {
        let designed = 20_722;
        assert!(SystemDays.today() >= designed, "2026-09-26 or later");
    }
}
