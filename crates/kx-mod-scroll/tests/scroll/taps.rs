//! A tap while still: it starts slow, or pages our list, or says why it cannot scroll; peeks and asks never scroll.

use crate::logs::lines_of;
use crate::rig::{APP, LIST, Rig, T0, after, hidden, outline, state, still, told};
use kx_mod_scroll::Msg;
use kx_module_api::keys::{SCROLL_END, SCROLL_FAILED, SCROLL_NONE, SCROLL_POINT};
use kx_module_api::ports::Dir::{Down, Left, Right, Up};
use kx_module_api::ports::{Reach, ScrollError, ScrollPage};
use tracing::Level;

use crate::rig::Heard::Page;

#[test]
fn the_start_says_it_is_live_and_still() {
    let mut rig = Rig::new();
    rig.begin();
    assert_eq!(rig.take(), [still()]);
}

#[test]
fn a_tap_starts_slow_and_outlines_the_target() {
    let mut rig = Rig::new();
    rig.begin();
    rig.take();
    rig.point(APP);
    rig.tap_at(Down, T0);
    assert_eq!(rig.take(), [state(Some(Down), false), outline(APP)]);
    assert!(rig.target.turns().is_empty(), "nothing is owed yet");
}

#[test]
fn a_late_ask_hears_the_state_and_the_outline_again() {
    let mut rig = Rig::new();
    rig.begin();
    rig.take();
    rig.send(Msg::Ask);
    assert_eq!(rig.take(), [still(), hidden()], "while still");
    rig.point(APP);
    rig.tap_at(Down, T0);
    rig.take();
    rig.send(Msg::Ask);
    assert_eq!(rig.take(), [state(Some(Down), false), outline(APP)]);
}

#[test]
fn a_tap_before_the_pointer_left_the_keyboard_says_point_first() {
    let mut rig = Rig::new();
    rig.begin();
    rig.take();
    rig.tap_at(Down, T0);
    assert_eq!(rig.take(), [told(SCROLL_POINT)]);
    assert_eq!(rig.actor.wait(), None, "nothing scrolls");
}

#[test]
fn a_tap_on_a_closed_window_says_nothing_to_scroll() {
    let mut rig = Rig::new();
    rig.target.push_aim(Err(ScrollError::Gone));
    rig.tap_at(Down, T0);
    assert_eq!(rig.take(), [told(SCROLL_NONE)]);
    assert_eq!(rig.actor.wait(), None);
}

#[test]
fn a_target_at_its_end_both_ways_says_nothing_to_scroll() {
    let mut rig = Rig::new();
    rig.point(APP);
    rig.target.set_reach(Down, Reach::End);
    rig.target.set_reach(Up, Reach::End);
    rig.tap_at(Down, T0);
    assert_eq!(rig.take(), [told(SCROLL_NONE)]);
    assert_eq!(rig.actor.wait(), None);
}

#[test]
fn a_target_at_its_end_that_way_says_already_at_the_end() {
    let mut rig = Rig::new();
    rig.point(APP);
    rig.target.set_reach(Left, Reach::End);
    rig.tap_at(Left, T0);
    assert_eq!(rig.take(), [told(SCROLL_END)]);
    assert_eq!(rig.actor.wait(), None);
    rig.tap_at(Right, after(1));
    assert_eq!(rig.take(), [state(Some(Right), false), outline(APP)]);
}

#[test]
fn with_stop_at_end_off_a_tap_at_the_end_starts_quietly() {
    let mut rig = Rig::with(|host| host.put("stop_at_end", false.into()));
    rig.point(APP);
    rig.target.set_reach(Down, Reach::End);
    rig.tap_at(Down, T0);
    assert_eq!(rig.take(), [state(Some(Down), false), outline(APP)]);
}

#[test]
fn a_refused_aim_at_a_tap_starts_and_the_next_tick_aims_again() {
    let mut rig = Rig::new();
    rig.target.push_aim(Err(ScrollError::Refused));
    rig.tap_at(Down, T0);
    assert_eq!(rig.take(), [state(Some(Down), false)], "no outline yet");
    rig.point(APP);
    let at = rig.ticks(1);
    rig.tick_at(at);
    assert_eq!(rig.take(), [outline(APP)]);
    assert_eq!(rig.target.turns().len(), 1, "it turns at once");
}

/// A rig whose OS refused the first aim of a tap down, with `stop_at_end` as given and what it published forgotten.
fn refused(end_stop: bool) -> Rig {
    let mut rig = Rig::with(|host| host.put("stop_at_end", end_stop.into()));
    rig.target.push_aim(Err(ScrollError::Refused));
    rig.tap_at(Down, T0);
    rig.take();
    rig
}

#[test]
fn a_check_that_finds_the_first_target_runs_the_taps_checks() {
    for (end_stop, back, key) in [
        (true, Reach::More, SCROLL_END),
        (false, Reach::End, SCROLL_NONE),
    ] {
        let mut rig = refused(end_stop);
        rig.point(APP);
        rig.target.set_reach(Down, Reach::End);
        rig.target.set_reach(Up, back);
        let at = rig.ticks(1);
        rig.tick_at(at);
        let want = [outline(APP), still(), hidden(), told(key)];
        assert_eq!(rig.take(), want, "stop_at_end {end_stop}");
        assert!(rig.target.turns().is_empty(), "nothing glides into the end");
    }
}

#[test]
fn a_check_whose_first_aim_fails_says_why() {
    for (err, key) in [
        (ScrollError::NoTarget, SCROLL_POINT),
        (ScrollError::Gone, SCROLL_NONE),
    ] {
        let mut rig = refused(true);
        rig.target.push_aim(Err(err));
        let at = rig.ticks(1);
        rig.tick_at(at);
        assert_eq!(rig.take(), [still(), hidden(), told(key)], "{err:?}");
    }
}

#[test]
fn a_faster_tap_that_finds_the_first_target_runs_the_taps_checks() {
    let mut rig = refused(true);
    rig.point(APP);
    rig.target.set_reach(Down, Reach::End);
    rig.tap_at(Down, after(1));
    let want = [outline(APP), still(), hidden(), told(SCROLL_END)];
    assert_eq!(rig.take(), want);
}

#[test]
fn a_first_aim_that_finds_our_list_turns_it_a_page_and_stops_quietly() {
    for (tap, back) in [(None, false), (Some(Down), false), (Some(Up), true)] {
        let mut rig = refused(true);
        rig.point(LIST);
        let at = rig.ticks(1);
        if let Some(dir) = tap {
            rig.tap_at(dir, at);
        } else {
            rig.tick_at(at);
        }
        let want = [Page(ScrollPage { back }), still(), hidden()];
        assert_eq!(rig.take(), want, "{tap:?}");
        assert_eq!(rig.actor.wait(), None, "{tap:?}");
        assert!(rig.target.turns().is_empty());
    }
}

#[test]
fn taps_on_our_list_turn_it_one_page_each() {
    let mut rig = Rig::new();
    rig.begin();
    rig.take();
    rig.point(LIST);
    for (dir, back) in [
        (Down, false),
        (Down, false),
        (Up, true),
        (Left, true),
        (Right, false),
    ] {
        rig.tap_at(dir, T0);
        assert_eq!(rig.take(), [Page(ScrollPage { back })], "{dir:?}");
        assert_eq!(rig.actor.wait(), None, "the pad stays still");
    }
    assert!(rig.target.turns().is_empty());
}

#[test]
fn a_peek_outlines_the_target_without_scrolling() {
    let mut rig = Rig::new();
    rig.begin();
    rig.take();
    rig.point(APP);
    rig.send(Msg::Peek(true));
    assert_eq!(rig.take(), [outline(APP)]);
    assert_eq!(rig.actor.wait(), None);
    assert!(rig.target.running(), "the first peek opened the run");
    rig.send(Msg::Peek(false));
    assert_eq!(rig.take(), [hidden()]);
    assert_eq!(
        (rig.target.running(), rig.target.opens()),
        (true, 1),
        "kept"
    );
    assert!(rig.target.turns().is_empty());
}

#[test]
fn a_peek_before_the_pointer_left_the_keyboard_shows_no_outline() {
    let mut rig = Rig::new();
    rig.send(Msg::Peek(true));
    assert_eq!(rig.take(), [hidden()]);
}

#[test]
fn a_tap_while_peeking_keeps_the_outline_it_shows() {
    let mut rig = Rig::new();
    rig.point(APP);
    rig.send(Msg::Peek(true));
    rig.take();
    rig.tap_at(Down, T0);
    assert_eq!(rig.take(), [state(Some(Down), false)]);
}

#[test]
fn a_target_that_does_not_open_tells_each_tap_and_logs_once() {
    let mut rig = Rig::new();
    rig.point(APP);
    let lines = lines_of(|| {
        for at in [T0, after(1)] {
            rig.target.fail_next(ScrollError::Refused);
            rig.tap_at(Down, at);
        }
    });
    let failed = told(SCROLL_FAILED);
    assert_eq!(rig.take(), [failed.clone(), failed], "each tap hears why");
    let warned: Vec<_> = lines.iter().filter(|(l, _)| *l == Level::WARN).collect();
    assert_eq!(warned.len(), 1, "once per streak: {lines:?}");
    assert_eq!(rig.actor.wait(), None);
    rig.tap_at(Down, after(2));
    assert_eq!(rig.take(), [state(Some(Down), false), outline(APP)]);
}

#[test]
fn a_peek_at_a_target_that_does_not_open_only_hides_the_outline() {
    let mut rig = Rig::new();
    rig.point(APP);
    rig.target.fail_next(ScrollError::Refused);
    rig.send(Msg::Peek(true));
    assert_eq!(rig.take(), [hidden()], "no notice for a hover");
}

#[test]
fn a_press_or_a_stop_while_still_does_nothing() {
    let mut rig = Rig::new();
    rig.point(APP);
    for msg in [Msg::Pressed(false), Msg::Pressed(true), Msg::Stop] {
        assert!(rig.send(msg));
    }
    assert_eq!(rig.take(), []);
    assert_eq!(rig.target.aims(), 0, "nothing was aimed at");
}
