//! The local calendar (spec §7 "Limits"): the wall time a module dates entries by, which needs no capability, and pure
//! day sums, so a module says today, yesterday, a weekday or a date and sweeps by calendar days.

use crate::{Capability, ServiceId, ServiceKey};

/// Days in each month of a common year, January first.
const MONTH_DAYS: [u8; 12] = [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
/// The month that gains a day in a leap year.
const LEAP_MONTH: u8 = 2;
/// Days in a common year.
const YEAR_DAYS: i64 = 365;
/// A year divisible by this is a leap year, unless it ends a century.
const LEAP_EVERY: i64 = 4;
/// Years in a century, whose last year is a common year unless it ends an era.
const CENTURY: i64 = 100;
/// Years in an era, whose last year is a leap year.
const ERA: i64 = 400;
/// Days in a week, as many as `Weekday::ALL` holds.
const WEEK: i64 = 7;
/// The first day the day numbers count from, the Unix epoch.
const EPOCH: Date = Date {
    year: 1970,
    month: 1,
    day: 1,
};
/// The epoch's weekday.
const EPOCH_WEEKDAY: Weekday = Weekday::Thursday;

/// A day of the week, numbered as ISO 8601 numbers it.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum Weekday {
    /// Monday.
    Monday = 1,
    /// Tuesday.
    Tuesday = 2,
    /// Wednesday.
    Wednesday = 3,
    /// Thursday.
    Thursday = 4,
    /// Friday.
    Friday = 5,
    /// Saturday.
    Saturday = 6,
    /// Sunday.
    Sunday = 7,
}

impl Weekday {
    /// Every day, Monday first.
    pub const ALL: [Self; 7] = [
        Self::Monday,
        Self::Tuesday,
        Self::Wednesday,
        Self::Thursday,
        Self::Friday,
        Self::Saturday,
        Self::Sunday,
    ];

    /// The day's ISO 8601 number: 1 for Monday to 7 for Sunday.
    #[must_use]
    pub const fn number(self) -> u8 {
        self as u8
    }
}

/// A checked local date in the Gregorian calendar, from year 1; dates order as time does.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Date {
    /// The year, from 1.
    year: u16,
    /// The month, 1 to 12.
    month: u8,
    /// The day of the month, from 1.
    day: u8,
}

impl Date {
    /// The date `year`-`month`-`day`; `None` when no such day exists.
    #[must_use]
    pub fn new(year: u16, month: u8, day: u8) -> Option<Self> {
        let real = year > 0 && day > 0 && month_days(year, month).is_some_and(|n| day <= n);
        real.then_some(Self { year, month, day })
    }

    /// The year.
    #[must_use]
    pub const fn year(self) -> u16 {
        self.year
    }

    /// The month, 1 to 12.
    #[must_use]
    pub const fn month(self) -> u8 {
        self.month
    }

    /// The day of the month.
    #[must_use]
    pub const fn day(self) -> u8 {
        self.day
    }

    /// Days since 1970-01-01, below 0 before it.
    #[must_use]
    pub fn day_number(self) -> i64 {
        self.ordinal() - EPOCH.ordinal()
    }

    /// The date `n` days after 1970-01-01; `None` outside years 1 to `u16::MAX`.
    #[must_use]
    pub fn from_day_number(n: i64) -> Option<Self> {
        let target = n.checked_add(EPOCH.ordinal()).filter(|&t| t >= 0)?;
        // No year is longer than a leap year, so this guess is never past the real year.
        let mut year = target / (YEAR_DAYS + 1) + 1;
        if year > i64::from(u16::MAX) {
            return None;
        }
        while year_start(year + 1) <= target {
            year += 1;
        }
        let mut left = target - year_start(year);
        let year = u16::try_from(year).ok()?;
        for month in (1..).take(MONTH_DAYS.len()) {
            let days = i64::from(month_days(year, month)?);
            if left < days {
                return Self::new(year, month, u8::try_from(left + 1).ok()?);
            }
            left -= days;
        }
        None
    }

    /// Calendar days from `earlier` to this date: 1 when `earlier` is the day before, below 0 when it is later.
    #[must_use]
    pub fn days_since(self, earlier: Self) -> i64 {
        self.day_number() - earlier.day_number()
    }

    /// The day of the week.
    #[must_use]
    pub fn weekday(self) -> Weekday {
        let from = i64::from(EPOCH_WEEKDAY.number()) - 1;
        let at = (self.day_number() + from).rem_euclid(WEEK);
        Weekday::ALL[usize::try_from(at).unwrap_or_default()]
    }

    /// Days since 0001-01-01.
    fn ordinal(self) -> i64 {
        let months = (1..self.month).filter_map(|m| month_days(self.year, m));
        let before: i64 = months.map(i64::from).sum();
        year_start(i64::from(self.year)) + before + i64::from(self.day) - 1
    }
}

/// True when `year` has a leap day.
fn is_leap(year: i64) -> bool {
    year % LEAP_EVERY == 0 && (year % CENTURY != 0 || year % ERA == 0)
}

/// Days in `month` of `year`; `None` for a month that does not exist.
fn month_days(year: u16, month: u8) -> Option<u8> {
    let common = *MONTH_DAYS.get(usize::from(month).checked_sub(1)?)?;
    let leap = month == LEAP_MONTH && is_leap(i64::from(year));
    Some(common + u8::from(leap))
}

/// Days from 0001-01-01 to the first day of `year`.
fn year_start(year: i64) -> i64 {
    let before = year - 1;
    before * YEAR_DAYS + before / LEAP_EVERY - before / CENTURY + before / ERA
}

/// The local wall time; it jumps when the user sets the clock or the time zone changes.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct LocalTime {
    /// Milliseconds since the Unix epoch, in UTC.
    pub unix_ms: i64,
    /// The local date.
    pub date: Date,
    /// The local hour, 0 to 23.
    pub hour: u8,
    /// The local minute, 0 to 59.
    pub minute: u8,
    /// The local offset from UTC in minutes, east above 0: 60 in Paris in winter.
    pub offset_min: i16,
}

/// Tells the local wall time; the platform provides it, and tests use a fake.
pub trait Calendar: Send + Sync {
    /// The local wall time now.
    fn now(&self) -> LocalTime;
}

/// The calendar service; it needs no capability.
pub struct CalendarKey;

impl ServiceKey for CalendarKey {
    type Api = dyn Calendar;
    const ID: ServiceId = ServiceId::new("calendar");
    const CAPABILITY: Option<Capability> = None;
}

#[cfg(test)]
mod tests;
