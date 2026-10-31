//! The fake calendar, held to the port's helpers: a wall time the test sets and moves, across midnight, a month end
//! and a summer-time change, always the one `LocalTime::at` gives.

use super::*;
use kx_module_api::ports::Weekday;
use std::sync::Arc;
use std::thread;

/// Paris in summer.
const PARIS_SUMMER: i16 = 120;
/// A minute.
const MINUTE: Duration = Duration::from_secs(60);
/// An hour.
const HOUR: Duration = Duration::from_secs(3_600);

/// A date the test knows is real.
fn date(year: u16, month: u8, day: u8) -> Date {
    Date::new(year, month, day).unwrap()
}

/// The fake's date, hour and minute now.
fn wall(cal: &FakeCalendar) -> (Date, u8, u8) {
    let now = cal.now();
    (now.date, now.hour, now.minute)
}

#[test]
fn the_wall_time_set_is_the_one_the_ports_helper_gives() {
    let cal = FakeCalendar::new(date(2026, 10, 31), 23, 59, PARIS);
    let now = cal.now();
    assert_eq!(wall(&cal), (date(2026, 10, 31), 23, 59));
    assert_eq!(now.offset_min, PARIS);
    assert_eq!(LocalTime::at(now.unix_ms, PARIS), Some(now));
    assert_eq!(now.date.weekday(), Weekday::Saturday);
    let utc = FakeCalendar::new(date(1970, 1, 1), 0, 0, 0);
    assert_eq!(utc.now().unix_ms, 0, "the epoch in UTC");
}

#[test]
fn advancing_crosses_midnight_and_a_month_end() {
    let cal = FakeCalendar::new(date(2026, 10, 31), 23, 59, PARIS);
    let before = cal.now().unix_ms;
    cal.advance(MINUTE);
    assert_eq!(wall(&cal), (date(2026, 11, 1), 0, 0));
    assert_eq!(cal.now().date.weekday(), Weekday::Sunday);
    assert_eq!(cal.now().unix_ms - before, 60_000);
}

#[test]
fn a_time_zone_change_keeps_the_moment_and_moves_the_wall() {
    let cal = FakeCalendar::new(date(2026, 3, 29), 1, 30, PARIS);
    let moment = cal.now().unix_ms;
    cal.set_offset(PARIS_SUMMER);
    assert_eq!(cal.now().unix_ms, moment);
    assert_eq!(wall(&cal), (date(2026, 3, 29), 2, 30));
    assert_eq!(cal.now().offset_min, PARIS_SUMMER);
}

#[test]
fn setting_the_clock_jumps_the_wall_and_keeps_the_offset() {
    let cal = FakeCalendar::new(date(2026, 10, 31), 12, 0, PARIS);
    cal.set(date(2026, 10, 30), 8, 15);
    assert_eq!(wall(&cal), (date(2026, 10, 30), 8, 15), "back a day");
    assert_eq!(cal.now().offset_min, PARIS);
    cal.advance(HOUR);
    assert_eq!(wall(&cal), (date(2026, 10, 30), 9, 15));
}

#[test]
fn a_west_offset_puts_the_utc_moment_after_the_wall() {
    let cal = FakeCalendar::new(date(2026, 1, 1), 0, 0, -PARIS);
    let utc = LocalTime::at(cal.now().unix_ms, 0).unwrap();
    assert_eq!((utc.date, utc.hour, utc.minute), (date(2026, 1, 1), 1, 0));
}

#[test]
fn clones_share_one_wall_time_across_threads() {
    let cal = FakeCalendar::new(date(2026, 10, 31), 10, 0, PARIS);
    let port: Arc<dyn Calendar> = Arc::new(cal.clone());
    cal.advance(HOUR);
    let seen = thread::spawn(move || port.now()).join().unwrap();
    assert_eq!(seen, cal.now());
    assert_eq!(seen.hour, 11);
}

#[test]
#[should_panic(expected = "no such wall time")]
fn an_hour_past_the_day_is_a_test_mistake() {
    let _ = FakeCalendar::new(date(2026, 10, 31), 24, 0, PARIS);
}

#[test]
#[should_panic(expected = "no such wall time")]
fn a_minute_past_the_hour_is_a_test_mistake() {
    FakeCalendar::new(date(2026, 10, 31), 10, 0, PARIS).set(date(2026, 10, 31), 10, 60);
}
