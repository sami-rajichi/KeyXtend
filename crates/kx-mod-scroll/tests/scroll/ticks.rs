//! The glide on ticks: only while scrolling, the owed lines on the latest target, a check every `check_ms`, and a
//! speed or way change that settles the owed lines first.

use crate::rig::{APP, LIST, OTHER, Rig, after, outline, same, state, turn};
use kx_mod_scroll::Msg;
use kx_module_api::ports::Dir::{Down, Up};
use kx_module_api::ports::{Aim, Rect, ScrollError};
use std::time::Duration;

#[test]
fn it_ticks_only_while_scrolling() {
    let mut rig = Rig::new();
    rig.begin();
    rig.point(APP);
    assert_eq!(rig.actor.wait(), None, "still: it waits for a message");
    rig.tick_at(rig.checks(4));
    assert_eq!((rig.target.aims(), rig.target.turns().len()), (0, 0));
    rig.tap_at(Down, rig.checks(4));
    let tick = Duration::from_millis(u64::from(rig.settings.tick_ms));
    assert_eq!(rig.actor.wait(), Some(tick), "scrolling: it ticks");
    rig.send(Msg::Stop);
    assert_eq!(rig.actor.wait(), None);
    let aims = rig.target.aims();
    rig.tick_at(rig.checks(9));
    assert_eq!(rig.target.aims(), aims, "a stray tick does nothing");
    assert!(rig.target.turns().is_empty());
}

#[test]
fn each_tick_turns_the_lines_owed_since_the_last() {
    let mut rig = Rig::new();
    rig.scrolling(Down);
    let tick = u64::from(rig.settings.tick_ms);
    rig.tick_at(rig.ticks(1));
    rig.tick_at(rig.ticks(1));
    rig.tick_at(rig.ticks(3));
    let want = [
        turn(APP, Down, rig.owed(false, tick)),
        turn(APP, Down, rig.owed(false, 2 * tick)),
    ];
    assert!(
        same(&rig.target.turns(), &want),
        "a tick owing 0 turns nothing"
    );
}

#[test]
fn it_aims_at_each_tap_and_each_check_not_each_tick() {
    let mut rig = Rig::new();
    rig.scrolling(Down);
    assert_eq!((rig.target.aims(), rig.target.reaches()), (1, 1));
    let per_check = u64::from(rig.settings.check_ms / rig.settings.tick_ms);
    for n in 1..per_check {
        rig.tick_at(rig.ticks(n));
    }
    assert_eq!((rig.target.aims(), rig.target.reaches()), (1, 1));
    rig.tick_at(rig.checks(1));
    assert_eq!((rig.target.aims(), rig.target.reaches()), (2, 2));
    rig.tap_at(Down, rig.checks(1));
    assert_eq!(rig.target.aims(), 3, "a tap aims too");
}

#[test]
fn a_retarget_moves_the_outline_and_the_turns() {
    let mut rig = Rig::new();
    rig.scrolling(Down);
    rig.point(OTHER);
    rig.tick_at(rig.ticks(1));
    rig.tick_at(rig.checks(1));
    assert_eq!(rig.take(), [outline(OTHER)]);
    let ids: Vec<_> = rig.target.take_turns().iter().map(|t| t.id).collect();
    assert_eq!(ids, [APP.id, OTHER.id], "turns follow at the check");
    rig.tick_at(rig.checks(2));
    assert_eq!(rig.take(), [], "the same target keeps its outline");
}

#[test]
fn a_new_box_on_the_same_target_moves_the_outline() {
    let mut rig = Rig::new();
    rig.scrolling(Down);
    let area = Some(Rect {
        x: 5,
        y: 6,
        w: 70,
        h: 80,
    });
    let moved = Aim { area, ..APP };
    rig.point(moved);
    rig.tick_at(rig.checks(1));
    assert_eq!(rig.take(), [outline(moved)]);
}

#[test]
fn the_pointer_on_our_windows_keeps_the_last_target() {
    let mut rig = Rig::new();
    rig.scrolling(Down);
    rig.point(LIST);
    rig.tick_at(rig.checks(1));
    assert_eq!(rig.take(), []);
    let ids: Vec<_> = rig.target.turns().iter().map(|t| t.id).collect();
    assert_eq!(ids, [APP.id]);
}

#[test]
fn a_faster_tap_settles_the_slow_lines_first() {
    let mut rig = Rig::new();
    rig.scrolling(Down);
    let tick = u64::from(rig.settings.tick_ms);
    rig.tick_at(rig.ticks(1));
    let _ = rig.target.take_turns();
    rig.tap_at(Down, after(tick + 5));
    assert_eq!(rig.take(), [state(Some(Down), true)]);
    let settled = [turn(APP, Down, rig.owed(false, 5))];
    assert!(same(&rig.target.take_turns(), &settled));
    rig.tick_at(after(2 * tick + 5));
    let fast = [turn(APP, Down, rig.owed(true, tick))];
    assert!(same(&rig.target.turns(), &fast));
}

#[test]
fn a_turn_settles_the_owed_lines_the_old_way_then_goes_slow_the_new() {
    let mut rig = Rig::new();
    rig.scrolling(Down);
    let tick = u64::from(rig.settings.tick_ms);
    rig.tap_at(Down, after(10));
    let _ = rig.target.take_turns();
    rig.take();
    rig.tap_at(Up, after(30));
    assert_eq!(rig.take(), [state(Some(Up), false)]);
    let settled = [turn(APP, Down, rig.owed(true, 20))];
    assert!(
        same(&rig.target.take_turns(), &settled),
        "fast, the old way"
    );
    rig.tick_at(after(30 + tick));
    let slow = [turn(APP, Up, rig.owed(false, tick))];
    assert!(same(&rig.target.turns(), &slow));
}

#[test]
fn a_refused_turn_drops_its_lines_and_the_next_owes_one_tick() {
    let mut rig = Rig::new();
    rig.scrolling(Down);
    let tick = u64::from(rig.settings.tick_ms);
    rig.target.set_turn_error(Some(ScrollError::Refused));
    rig.tick_at(rig.ticks(1));
    assert_eq!(rig.take(), [], "it goes on");
    assert!(rig.actor.wait().is_some());
    rig.target.set_turn_error(None);
    rig.tick_at(rig.ticks(2));
    let one = [turn(APP, Down, rig.owed(false, tick))];
    assert!(same(&rig.target.turns(), &one), "nothing piled up");
}

#[test]
fn a_long_gap_owes_one_check_at_most() {
    let mut rig = Rig::new();
    rig.scrolling(Down);
    let check = u64::from(rig.settings.check_ms);
    rig.tick_at(after(60_000));
    let capped = [turn(APP, Down, rig.owed(false, check))];
    assert!(same(&rig.target.turns(), &capped));
}
