//! Each way a scroll stops: the third tap, Stop, the end, a press, the limit, a closed target and a turn onto an end.

use crate::rig::{APP, OTHER, Rig, after, hidden, state, still, told};
use kx_mod_scroll::Msg;
use kx_module_api::MS_PER_S;
use kx_module_api::keys::{SCROLL_END, SCROLL_NONE, SCROLL_POINT};
use kx_module_api::ports::Dir::{Down, Up};
use kx_module_api::ports::{Reach, ScrollError};

/// The stop every quiet halt shows: still, and the outline hidden.
fn stopped() -> [crate::rig::Heard; 2] {
    [still(), hidden()]
}

#[test]
fn the_third_tap_stops() {
    let mut rig = Rig::new();
    rig.scrolling(Down);
    rig.tap_at(Down, after(1));
    assert_eq!(rig.take(), [state(Some(Down), true)]);
    rig.tap_at(Down, after(2));
    assert_eq!(rig.take(), stopped());
    assert_eq!(rig.actor.wait(), None);
}

#[test]
fn the_stop_button_stops_at_either_speed() {
    for fast in [false, true] {
        let mut rig = Rig::new();
        rig.scrolling(Down);
        if fast {
            rig.tap_at(Down, after(1));
        }
        rig.take();
        rig.send(Msg::Stop);
        assert_eq!(rig.take(), stopped(), "fast {fast}");
        assert_eq!(rig.actor.wait(), None);
    }
}

#[test]
fn reaching_the_end_stops_quietly_at_the_next_check() {
    let mut rig = Rig::new();
    rig.scrolling(Down);
    rig.target.set_reach(Down, Reach::End);
    rig.tick_at(rig.ticks(1));
    assert_eq!(rig.take(), [], "read only at a check");
    rig.tick_at(rig.checks(1));
    assert_eq!(rig.take(), stopped());
    assert_eq!(rig.target.turns().len(), 1, "nothing turns past the end");
}

#[test]
fn with_stop_at_end_off_it_runs_past_the_end() {
    let mut rig = Rig::with(|host| host.put("stop_at_end", false.into()));
    rig.scrolling(Down);
    rig.target.set_reach(Down, Reach::End);
    rig.tick_at(rig.checks(1));
    assert_eq!(rig.take(), []);
    assert_eq!(rig.target.turns().len(), 1);
}

#[test]
fn a_press_outside_our_windows_stops_and_one_on_them_does_not() {
    let mut rig = Rig::new();
    rig.scrolling(Down);
    rig.send(Msg::Pressed(true));
    assert_eq!(rig.take(), [], "a press on the pad");
    rig.send(Msg::Pressed(false));
    assert_eq!(rig.take(), stopped());
}

#[test]
fn with_stop_on_press_off_a_press_goes_on() {
    let mut rig = Rig::with(|host| host.put("stop_on_press", false.into()));
    rig.scrolling(Down);
    rig.send(Msg::Pressed(false));
    assert_eq!(rig.take(), []);
    assert!(rig.actor.wait().is_some());
}

#[test]
fn the_limit_counts_from_the_last_pad_tap() {
    let mut rig = Rig::new();
    rig.scrolling(Down);
    let limit = u64::from(rig.settings.limit_s * MS_PER_S);
    let late = limit / 4;
    rig.tap_at(Down, after(late));
    rig.take();
    rig.tick_at(after(limit));
    assert_eq!(rig.take(), [], "the tap restarted the count");
    rig.tick_at(after(late + limit));
    assert_eq!(rig.take(), stopped());
}

#[test]
fn with_no_limit_it_scrolls_on() {
    let mut rig = Rig::with(|host| host.put("limit_s", 0.into()));
    rig.scrolling(Down);
    rig.tick_at(after(24 * 3_600 * u64::from(MS_PER_S)));
    assert_eq!(rig.take(), []);
    assert!(rig.actor.wait().is_some());
}

#[test]
fn a_closed_target_stops_quietly() {
    let mut rig = Rig::new();
    rig.scrolling(Down);
    rig.target.set_turn_error(Some(ScrollError::Gone));
    rig.tick_at(rig.ticks(1));
    assert_eq!(rig.take(), stopped(), "at a turn");
    let mut rig = Rig::new();
    rig.scrolling(Down);
    rig.target.push_aim(Err(ScrollError::Gone));
    rig.tick_at(rig.checks(1));
    assert_eq!(rig.take(), stopped(), "at a check");
}

/// A target that closes just before a check stops, though another window now lies at its point: that one is not what
/// the user pointed at.
#[test]
fn a_target_that_closed_before_a_check_stops_though_a_window_is_there_now() {
    let mut rig = Rig::new();
    rig.scrolling(Down);
    rig.target.close(APP.id);
    rig.point(OTHER);
    rig.tick_at(rig.checks(1));
    assert_eq!(rig.take(), stopped());
    assert!(rig.target.turns().iter().all(|t| t.id != OTHER.id));
}

/// A refused aim, or one with no point, as with the pointer resting on our banner, keeps gliding the last target.
#[test]
fn a_refused_or_pointless_aim_at_a_check_keeps_the_target() {
    for err in [ScrollError::Refused, ScrollError::NoTarget] {
        let mut rig = Rig::new();
        rig.scrolling(Down);
        rig.target.push_aim(Err(err));
        rig.tick_at(rig.checks(1));
        assert_eq!(rig.take(), [], "{err:?}");
        let (check, tick) = (rig.settings.check_ms, rig.settings.tick_ms);
        rig.tick_at(after(u64::from(check + tick)));
        let ids: Vec<_> = rig.target.turns().iter().map(|t| t.id).collect();
        assert_eq!(ids, [APP.id, APP.id], "{err:?}");
    }
}

#[test]
fn a_turn_onto_an_end_stops_and_says_already_at_the_end() {
    let mut rig = Rig::new();
    rig.scrolling(Down);
    rig.target.set_reach(Up, Reach::End);
    rig.tap_at(Up, after(1));
    assert_eq!(rig.take(), [still(), hidden(), told(SCROLL_END)]);
    assert_eq!(rig.actor.wait(), None);
}

#[test]
fn a_turn_where_the_window_closed_stops_and_says_nothing_to_scroll() {
    let mut rig = Rig::new();
    rig.scrolling(Down);
    rig.target.push_aim(Err(ScrollError::Gone));
    rig.tap_at(Up, after(1));
    assert_eq!(rig.take(), [still(), hidden(), told(SCROLL_NONE)]);
}

#[test]
fn a_faster_tap_whose_aim_fails_stops_and_says_why() {
    for (err, key) in [
        (ScrollError::Gone, SCROLL_NONE),
        (ScrollError::NoTarget, SCROLL_POINT),
    ] {
        let mut rig = Rig::new();
        rig.scrolling(Down);
        rig.target.push_aim(Err(err));
        rig.tap_at(Down, after(1));
        assert_eq!(rig.take(), [still(), hidden(), told(key)], "{err:?}");
        assert_eq!(rig.actor.wait(), None);
    }
}
