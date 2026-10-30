//! The calendar port: checked dates, day sums across month ends, year ends and leap days, weekdays, and its open gate.

use super::*;
use proptest::prelude::*;

/// The day number of 0001-01-01, the first date there is.
const FIRST_DAY: i64 = -719_162;
/// A day number past this century, for the property test.
const LATE_DAY: i64 = 200_000;

/// A date the test knows is real.
fn date(year: u16, month: u8, day: u8) -> Date {
    Date::new(year, month, day).unwrap()
}

#[test]
fn the_calendar_needs_no_capability() {
    assert_eq!(CalendarKey::ID.as_str(), "calendar");
    assert_eq!(CalendarKey::CAPABILITY, None);
}

#[test]
fn a_date_is_checked() {
    assert_eq!(date(2026, 10, 31).year(), 2026);
    assert_eq!(
        (date(2026, 10, 31).month(), date(2026, 10, 31).day()),
        (10, 31)
    );
    let bad = [
        (2026, 0, 1),
        (2026, 13, 1),
        (2026, 1, 0),
        (2026, 4, 31),
        (2026, 2, 29),
        (2100, 2, 29),
        (0, 1, 1),
    ];
    for (y, m, d) in bad {
        assert_eq!(Date::new(y, m, d), None, "{y}-{m}-{d}");
    }
    assert!(Date::new(2024, 2, 29).is_some(), "a leap day");
    assert!(Date::new(2000, 2, 29).is_some(), "a leap century");
}

#[test]
fn the_epoch_is_day_zero_and_a_thursday() {
    let epoch = date(1970, 1, 1);
    assert_eq!(epoch.day_number(), 0);
    assert_eq!(epoch.weekday(), Weekday::Thursday);
    assert_eq!(date(1969, 12, 31).day_number(), -1);
    assert_eq!(date(1, 1, 1).day_number(), FIRST_DAY);
}

#[test]
fn days_since_crosses_month_and_year_ends() {
    assert_eq!(date(2026, 11, 1).days_since(date(2026, 10, 31)), 1);
    assert_eq!(date(2027, 1, 1).days_since(date(2026, 12, 31)), 1);
    assert_eq!(date(2026, 10, 25).days_since(date(2026, 10, 31)), -6);
    assert_eq!(date(2027, 1, 1).days_since(date(2026, 1, 1)), 365);
}

#[test]
fn a_leap_day_is_a_day_of_its_own() {
    assert_eq!(date(2024, 3, 1).days_since(date(2024, 2, 28)), 2);
    assert_eq!(date(2023, 3, 1).days_since(date(2023, 2, 28)), 1);
    assert_eq!(
        date(2100, 3, 1).days_since(date(2100, 2, 28)),
        1,
        "2100 skips it"
    );
    assert_eq!(date(2025, 1, 1).days_since(date(2024, 1, 1)), 366);
    assert_eq!(
        date(2024, 2, 29).day_number(),
        19_782,
        "the UTC day of 2024-02-29"
    );
}

#[test]
fn weekdays_follow_the_calendar() {
    assert_eq!(date(2026, 10, 31).weekday(), Weekday::Saturday);
    assert_eq!(date(2024, 2, 29).weekday(), Weekday::Thursday);
    assert_eq!(date(2000, 1, 1).weekday(), Weekday::Saturday);
    assert_eq!(Weekday::ALL.map(Weekday::number), [1, 2, 3, 4, 5, 6, 7]);
    assert_eq!(i64::try_from(Weekday::ALL.len()), Ok(WEEK));
}

#[test]
fn a_day_number_outside_the_years_is_no_date() {
    assert_eq!(Date::from_day_number(FIRST_DAY), Some(date(1, 1, 1)));
    assert_eq!(Date::from_day_number(FIRST_DAY - 1), None);
    let last = date(u16::MAX, 12, 31);
    assert_eq!(Date::from_day_number(last.day_number()), Some(last));
    assert_eq!(Date::from_day_number(last.day_number() + 1), None);
}

proptest! {
    #[test]
    fn a_day_number_maps_back_to_its_date_and_the_next_day_follows(n in FIRST_DAY..LATE_DAY) {
        let day = Date::from_day_number(n).unwrap();
        prop_assert_eq!(day.day_number(), n);
        let next = Date::from_day_number(n + 1).unwrap();
        prop_assert!(next > day);
        prop_assert_eq!(next.days_since(day), 1);
        prop_assert_eq!(next.weekday().number(), day.weekday().number() % 7 + 1);
    }
}
