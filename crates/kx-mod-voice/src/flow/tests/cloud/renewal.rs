//! A used allowance's renewal (owner, 2026-11-20): a clock time read on the calendar in the offset in force then, so
//! a summer-time change between shows its own hour (N3), and whether it falls on the day the note came (m6).

use kx_module_api::ports::{Calendar, Date, Renewal};
use kx_platform_fake::FakeCalendar;

use super::*;
use crate::flow::renewal;

/// Paris in summer and in winter, in minutes east of UTC.
const SUMMER_MIN: i16 = 120;
const WINTER_MIN: i16 = 60;

/// The calendar the flow tests' notes come on: `wall`, whose offset never moves.
struct Wall;

impl Calendar for Wall {
    fn now(&self) -> LocalTime {
        wall()
    }
}

/// When `sent`'s used allowance renews, read on `Wall` as the actor reads it on the calendar.
pub(in crate::flow::tests) fn renewed(sent: &Sent) -> Option<Renewal> {
    renewal(&Wall, sent.fault())
}

/// A calendar fixed at `now` whose offset moves to `later_min` from `switch_ms` on, as the end of summer time does.
struct Switch {
    now: LocalTime,
    switch_ms: i64,
    later_min: i16,
}

impl Calendar for Switch {
    fn now(&self) -> LocalTime {
        self.now
    }

    fn local(&self, unix_ms: i64) -> Option<LocalTime> {
        let before = unix_ms < self.switch_ms;
        let offset = if before {
            self.now.offset_min
        } else {
            self.later_min
        };
        LocalTime::at(unix_ms, offset)
    }
}

/// Paris at 00:30 on 25 October 2026, in summer time, which ends at 01:00 UTC.
fn paris() -> Switch {
    let day = Date::new(2026, 10, 25).unwrap();
    Switch {
        now: FakeCalendar::new(day, 0, 30, SUMMER_MIN).now(),
        switch_ms: FakeCalendar::new(day, 1, 0, 0).now().unix_ms,
        later_min: WINTER_MIN,
    }
}

/// The clock time and the today flag of a used allowance that renews `wait` after now on `calendar`.
fn renews(calendar: &dyn Calendar, wait: Duration) -> Option<(u8, u8, bool)> {
    let used = CloudFault::Allowance {
        renews_in: Some(wait),
    };
    renewal(calendar, Some(&used)).map(|r| (r.at.hour, r.at.minute, r.today))
}

#[test]
fn a_renewal_across_the_end_of_summer_time_shows_the_hour_then() {
    let five = Duration::from_hours(5);
    assert_eq!(
        renews(&paris(), five),
        Some((4, 30, true)),
        "05:30 in summer time, 04:30 once it ended"
    );
}

#[test]
fn a_renewal_says_whether_it_falls_on_the_day_the_note_came() {
    assert_eq!(
        renews(&Wall, Duration::from_mins(90)),
        Some((14, 30, true)),
        "13:00 and 90 minutes"
    );
    assert_eq!(
        renews(&Wall, Duration::from_hours(12)),
        Some((1, 0, false)),
        "tomorrow"
    );
}

#[test]
fn only_a_used_allowance_that_tells_its_wait_renews() {
    let untold = CloudFault::Allowance { renews_in: None };
    for fault in [Some(&untold), Some(&CloudFault::Key), None] {
        assert_eq!(renewal(&Wall, fault), None, "{fault:?}");
    }
}
