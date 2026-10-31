//! A fake calendar: a wall time the test sets and moves, always the one `LocalTime::at` gives for its moment and
//! offset, so it agrees with the port's day sums. It moves apart from `FakeClock`, so a midnight test advances both.

use crate::lock;
use kx_module_api::MS_PER_S;
use kx_module_api::ports::{Calendar, Date, LocalTime};
use std::sync::{Arc, Mutex};
use std::time::Duration;

/// Seconds in a minute.
const S_PER_MIN: i64 = 60;
/// Minutes in an hour.
const MIN_PER_HOUR: i64 = 60;
/// Hours in a day.
const HOURS_PER_DAY: i64 = 24;
/// What a test hears when it asks for a wall time `LocalTime` cannot hold.
const NO_SUCH: &str = "no such wall time";

/// A calendar shared by its clones; its wall time moves only when the test says so, apart from `FakeClock`, so a
/// midnight test advances both.
#[derive(Clone, Debug)]
pub struct FakeCalendar {
    now: Arc<Mutex<LocalTime>>,
}

impl FakeCalendar {
    /// A calendar at `hour`:`minute` on `date`, `offset_min` minutes east of UTC.
    ///
    /// # Panics
    /// When no such wall time exists, such as hour 24: a test's mistake.
    #[must_use]
    pub fn new(date: Date, hour: u8, minute: u8, offset_min: i16) -> Self {
        let now = wall(date, hour, minute, offset_min);
        Self {
            now: Arc::new(Mutex::new(now)),
        }
    }

    /// Sets the clock to `hour`:`minute` on `date`, forward or back, keeping the offset, as a user does.
    ///
    /// # Panics
    /// When no such wall time exists: a test's mistake.
    pub fn set(&self, date: Date, hour: u8, minute: u8) {
        let mut now = lock(&self.now);
        *now = wall(date, hour, minute, now.offset_min);
    }

    /// Moves the time zone to `offset_min`, as a summer-time change does: the same moment, another wall time.
    ///
    /// # Panics
    /// Past the years a `Date` holds: a test's mistake.
    pub fn set_offset(&self, offset_min: i16) {
        let mut now = lock(&self.now);
        *now = checked(LocalTime::at(now.unix_ms, offset_min));
    }

    /// Moves the moment forward by `by`, keeping the offset; parts under a millisecond are dropped.
    ///
    /// # Panics
    /// Past the years a `Date` holds: a test's mistake.
    pub fn advance(&self, by: Duration) {
        let mut now = lock(&self.now);
        let ms = i64::try_from(by.as_millis()).ok();
        let to = ms.and_then(|ms| now.unix_ms.checked_add(ms));
        *now = checked(to.and_then(|to| LocalTime::at(to, now.offset_min)));
    }
}

impl Calendar for FakeCalendar {
    fn now(&self) -> LocalTime {
        *lock(&self.now)
    }
}

/// The wall time asked for, checked against what `LocalTime::at` makes of its moment.
fn wall(date: Date, hour: u8, minute: u8, offset_min: i16) -> LocalTime {
    let day_min = HOURS_PER_DAY * MIN_PER_HOUR;
    let of_day = i64::from(hour) * MIN_PER_HOUR + i64::from(minute);
    let utc_min = date.day_number() * day_min + of_day - i64::from(offset_min);
    let at = LocalTime::at(utc_min * S_PER_MIN * i64::from(MS_PER_S), offset_min);
    checked(at.filter(|t| (t.date, t.hour, t.minute) == (date, hour, minute)))
}

/// `at`, or a test's mistake.
fn checked(at: Option<LocalTime>) -> LocalTime {
    at.unwrap_or_else(|| panic!("{NO_SUCH}"))
}

#[cfg(test)]
mod tests;
