//! Each stop rule of spec §5.2: the tap's checks, the end, a press, the limit and a lost target.

use super::*;
use Halt::{AtEnd, End, Gone, Limit, NoPoint, Nothing, Press};
use kx_module_api::US_PER_S;
use kx_module_api::ports::Reach::{More, Unknown};
use kx_module_api::ports::ScrollError::{NoTarget, Refused};
use std::cell::Cell;

/// The default settings: both stops on, a 120 s limit.
pub(super) fn defaults() -> ScrollSettings {
    ScrollSettings::from_defaults().unwrap()
}

/// The defaults with `stop_at_end` off, as for a feed that loads more at its end.
fn no_end_stop() -> ScrollSettings {
    ScrollSettings {
        stop_at_end: false,
        ..defaults()
    }
}

/// The point `n` s after the start.
fn secs(n: u64) -> Mono {
    Mono::from_us(n * u64::from(US_PER_S))
}

/// A reach reading that must never happen.
fn unread() -> Reach {
    panic!("the other way was read")
}

#[test]
fn only_a_tap_that_cannot_scroll_is_told() {
    for halt in [NoPoint, Nothing, AtEnd] {
        assert!(halt.told(), "{halt:?}");
    }
    for halt in [End, Press, Limit, Gone] {
        assert!(!halt.told(), "{halt:?}");
    }
}

#[test]
fn a_tap_with_no_point_yet_says_point_first() {
    assert_eq!(Halt::unaimed(NoTarget), Some(NoPoint));
}

#[test]
fn a_tap_on_a_closed_window_says_nothing_to_scroll() {
    assert_eq!(Halt::unaimed(ScrollError::Gone), Some(Nothing));
}

#[test]
fn a_refused_aim_at_a_tap_does_not_stop() {
    assert_eq!(Halt::unaimed(Refused), None);
}

#[test]
fn the_end_both_ways_says_nothing_to_scroll() {
    let stops = Stops::from(&defaults());
    assert_eq!(stops.blocked(Reach::End, || Reach::End), Some(Nothing));
}

#[test]
fn the_end_only_that_way_says_already_at_the_end() {
    let stops = Stops::from(&defaults());
    assert_eq!(stops.blocked(Reach::End, || More), Some(AtEnd));
    assert_eq!(stops.blocked(Reach::End, || Unknown), Some(AtEnd));
}

#[test]
fn with_stop_at_end_off_the_end_only_that_way_starts_quietly() {
    let stops = Stops::from(&no_end_stop());
    assert_eq!(stops.blocked(Reach::End, || More), None);
    assert_eq!(stops.blocked(Reach::End, || Unknown), None);
}

#[test]
fn with_stop_at_end_off_the_end_both_ways_still_says_nothing_to_scroll() {
    let stops = Stops::from(&no_end_stop());
    assert_eq!(stops.blocked(Reach::End, || Reach::End), Some(Nothing));
}

#[test]
fn a_target_that_can_move_that_way_starts_without_reading_the_other() {
    for s in [defaults(), no_end_stop()] {
        let stops = Stops::from(&s);
        assert_eq!(stops.blocked(More, unread), None);
        assert_eq!(stops.blocked(Unknown, unread), None);
    }
}

#[test]
fn the_other_way_is_read_once_at_most() {
    for s in [defaults(), no_end_stop()] {
        let reads = Cell::new(0);
        let back = || {
            reads.set(reads.get() + 1);
            More
        };
        Stops::from(&s).blocked(Reach::End, back);
        assert_eq!(reads.get(), 1);
    }
}

#[test]
fn the_settings_give_the_stops() {
    let stops = Stops::from(&defaults());
    assert_eq!(stops.reached(Reach::End), Some(End));
    assert_eq!(stops.pressed(false), Some(Press));
    assert_eq!(stops.timed_out(secs(120)), Some(Limit));
}

#[test]
fn reaching_the_end_stops_quietly() {
    let stops = Stops::from(&defaults());
    assert_eq!(stops.reached(Reach::End), Some(End));
    assert_eq!(stops.reached(More), None);
    assert_eq!(stops.reached(Unknown), None);
}

#[test]
fn with_stop_at_end_off_the_end_goes_on() {
    assert_eq!(Stops::from(&no_end_stop()).reached(Reach::End), None);
}

#[test]
fn a_press_outside_our_windows_stops() {
    assert_eq!(Stops::from(&defaults()).pressed(false), Some(Press));
}

#[test]
fn a_press_on_our_windows_never_stops() {
    assert_eq!(Stops::from(&defaults()).pressed(true), None);
}

#[test]
fn with_stop_on_press_off_a_press_goes_on() {
    let stops = Stops::from(&ScrollSettings {
        stop_on_press: false,
        ..defaults()
    });
    assert_eq!(stops.pressed(false), None);
    assert_eq!(stops.pressed(true), None);
}

#[test]
fn the_limit_counts_from_the_last_pad_tap() {
    let mut stops = Stops::from(&defaults());
    stops.tapped(secs(10));
    assert_eq!(stops.timed_out(secs(129)), None);
    assert_eq!(stops.timed_out(secs(130)), Some(Limit));
    stops.tapped(secs(100));
    assert_eq!(stops.timed_out(secs(219)), None);
    assert_eq!(stops.timed_out(secs(220)), Some(Limit));
}

#[test]
fn the_limit_waits_to_the_microsecond() {
    let mut stops = Stops::from(&defaults());
    stops.tapped(secs(1));
    let due = secs(121);
    assert_eq!(stops.timed_out(Mono::from_us(due.as_us() - 1)), None);
    assert_eq!(stops.timed_out(due), Some(Limit));
}

#[test]
fn a_time_before_the_last_tap_is_within_the_limit() {
    let mut stops = Stops::from(&defaults());
    stops.tapped(secs(500));
    assert_eq!(stops.timed_out(secs(0)), None);
}

#[test]
fn no_limit_never_times_out() {
    let mut stops = Stops::from(&ScrollSettings {
        limit_s: 0,
        ..defaults()
    });
    stops.tapped(secs(0));
    assert_eq!(stops.timed_out(Mono::from_us(u64::MAX)), None);
}

#[test]
fn a_closed_or_lost_target_stops_quietly() {
    assert_eq!(Halt::failed(ScrollError::Gone), Some(Gone));
    assert_eq!(Halt::failed(NoTarget), Some(Gone));
}

#[test]
fn a_refused_step_does_not_stop() {
    assert_eq!(Halt::failed(Refused), None);
}
