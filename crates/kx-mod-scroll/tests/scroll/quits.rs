//! What a stop leaves: the outline only while peeking, nothing after a quit, and one stats line of numbers.

use crate::logs::{Fields, MESSAGE, lines_of};
use crate::rig::{APP, Heard, Rig, T0, after, hidden, still};
use kx_mod_scroll::{Msg, STOPPED_LINE};
use kx_module_api::ports::Dir::{Down, Left, Up};
use kx_module_api::ports::{Reach, ScrollChanged, ScrollError};
use tracing::Level;

#[test]
fn a_stop_while_peeking_keeps_the_outline() {
    let mut rig = Rig::new();
    rig.point(APP);
    rig.send(Msg::Peek(true));
    rig.tap_at(Down, T0);
    rig.take();
    rig.send(Msg::Stop);
    assert_eq!(rig.take(), [still()], "the pointer is still on the pad");
    rig.send(Msg::Peek(false));
    assert_eq!(rig.take(), [hidden()]);
}

#[test]
fn a_peek_while_scrolling_changes_nothing_until_the_stop() {
    let mut rig = Rig::new();
    rig.scrolling(Down);
    rig.send(Msg::Peek(true));
    rig.send(Msg::Peek(false));
    assert_eq!(rig.take(), [], "the outline shows while scrolling");
    rig.send(Msg::Stop);
    assert_eq!(rig.take(), [still(), hidden()]);
}

#[test]
fn quitting_while_scrolling_clears_the_outline_and_closes_the_run() {
    let mut rig = Rig::new();
    rig.scrolling(Down);
    assert!(!rig.send(Msg::Quit), "the actor is done");
    let off = [Heard::Changed(ScrollChanged::OFF), hidden()];
    assert_eq!(rig.take(), off);
    let target = &rig.target;
    assert_eq!((target.running(), target.closes()), (false, 1));
}

#[test]
fn quitting_before_any_need_opens_nothing() {
    let mut rig = Rig::new();
    rig.begin();
    rig.take();
    assert!(!rig.send(Msg::Quit));
    assert_eq!(rig.take(), [Heard::Changed(ScrollChanged::OFF), hidden()]);
    assert_eq!(rig.target.opens(), 0);
}

/// The stats line's fields in order of the counts a run below makes.
const COUNTS: [&str; 12] = [
    "taps", "ticks", "turns", "refused", "user", "no_point", "nothing", "at_end", "end", "press",
    "limit", "gone",
];

/// Taps, ticks, turns, a refusal and one stop of each kind but `limit`, `nothing` and `gone`.
fn busy(rig: &mut Rig) {
    rig.tap_at(Down, T0);
    rig.point(APP);
    rig.target.set_reach(Left, Reach::End);
    rig.tap_at(Left, after(1));
    rig.tap_at(Down, after(2));
    rig.tick_at(rig.ticks(1));
    rig.target.set_turn_error(Some(ScrollError::Refused));
    rig.tick_at(rig.ticks(2));
    rig.target.set_turn_error(None);
    rig.send(Msg::Pressed(false));
    rig.tap_at(Up, rig.ticks(3));
    rig.target.set_reach(Up, Reach::End);
    rig.tick_at(rig.checks(2));
    rig.tap_at(Down, rig.checks(3));
    rig.send(Msg::Stop);
    rig.send(Msg::Quit);
}

#[test]
fn the_stopped_line_holds_counts_only() {
    let mut rig = Rig::new();
    let lines = lines_of(|| busy(&mut rig));
    let stopped: Vec<&Fields> = lines
        .iter()
        .filter(|(l, f)| *l == Level::INFO && f[MESSAGE] == STOPPED_LINE)
        .map(|(_, f)| f)
        .collect();
    assert_eq!(stopped.len(), 1, "{lines:?}");
    let got: Vec<(&str, &str)> = COUNTS
        .iter()
        .map(|&k| (k, stopped[0].get(k).map_or("", String::as_str)))
        .collect();
    let want = [
        ("taps", "5"),
        ("ticks", "3"),
        ("turns", "1"),
        ("refused", "1"),
        ("user", "1"),
        ("no_point", "1"),
        ("nothing", "0"),
        ("at_end", "1"),
        ("end", "1"),
        ("press", "1"),
        ("limit", "0"),
        ("gone", "0"),
    ];
    assert_eq!(got, want);
    assert_eq!(
        stopped[0].len(),
        COUNTS.len() + 1,
        "the counts and the message"
    );
    let area = APP.area.unwrap();
    let parts = [APP.at.x, APP.at.y, area.x, area.y, area.w, area.h].map(|n| n.to_string());
    for value in lines.iter().flat_map(|(_, f)| f.values()) {
        assert!(parts.iter().all(|p| !value.contains(p.as_str())), "{value}");
    }
    assert_eq!(rig.take().last(), Some(&hidden()));
}
