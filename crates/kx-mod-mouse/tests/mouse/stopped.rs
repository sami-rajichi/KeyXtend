//! The line each hook stop logs: the hook's numbers and no point, at info for a run a mode or a latch held and at
//! debug for one only scrolling or presses held.

use crate::logs::{Fields, MESSAGE, lines_of};
use crate::rig::{A, B, Rig, SHIFT, T0, T1, T2, down, moved, up};
use kx_mod_mouse::{Msg, STOPPED_LINE};
use kx_module_api::ports::{HookStats, Toggle};
use tracing::Level;

/// Stats whose numbers appear nowhere else.
const STATS: HookStats = HookStats {
    events: 101,
    p99_us: 202,
    max_us: 303,
    max_wait_ms: 404,
    reinstalls: 5,
    send_errors: 6,
    lost_echoes: 7,
    passed_inside: 8,
};

#[test]
fn the_stopped_line_logs_the_hook_numbers_and_no_point() {
    let lines = lines_of(|| {
        let mut rig = Rig::saved_rclick();
        rig.hook.set_stats(STATS);
        rig.begin();
        let _ = rig.feed(moved(B), T0);
        let _ = rig.feed(down(A), T1);
        rig.send(Msg::Stop);
    });
    let stopped: Vec<_> = lines
        .iter()
        .filter(|(_, f)| f[MESSAGE] == STOPPED_LINE)
        .collect();
    assert_eq!(stopped.len(), 1, "{lines:?}");
    let (level, fields) = stopped[0];
    assert_eq!(*level, Level::INFO);
    let numbers = [
        ("events", STATS.events),
        ("p99_us", STATS.p99_us.into()),
        ("max_us", STATS.max_us.into()),
        ("max_wait_ms", STATS.max_wait_ms.into()),
        ("reinstalls", STATS.reinstalls.into()),
        ("send_errors", STATS.send_errors.into()),
        ("lost_echoes", STATS.lost_echoes.into()),
        ("passed_inside", STATS.passed_inside.into()),
    ];
    let mut want: Vec<_> = numbers.map(|(k, v)| (k, v.to_string())).to_vec();
    want.push((MESSAGE, STOPPED_LINE.to_owned()));
    want.sort_unstable();
    let got: Vec<_> = fields.iter().map(|(k, v)| (*k, v.clone())).collect();
    assert_eq!(got, want);
    let points = [A.x, A.y, B.x, B.y].map(|n| n.to_string());
    for line in lines.iter().flat_map(|(_, f)| f.values()) {
        assert!(points.iter().all(|p| !line.contains(p.as_str())), "{line}");
    }
}

/// The levels of the stopped lines a fresh rig logs while `play` runs.
fn stop_levels(play: impl FnOnce(&mut Rig)) -> Vec<Level> {
    let lines = lines_of(|| {
        let mut rig = Rig::new();
        rig.begin();
        play(&mut rig);
    });
    let stopped = |f: &Fields| f.get(MESSAGE).is_some_and(|m| m == STOPPED_LINE);
    lines
        .iter()
        .filter(|(_, f)| stopped(f))
        .map(|(l, _)| *l)
        .collect()
}

/// A run held only by `msg` turned on and then off.
fn quiet(msg: fn(bool) -> Msg) -> impl FnOnce(&mut Rig) {
    move |rig: &mut Rig| {
        rig.send(msg(true));
        rig.send(msg(false));
        rig.settle(T0);
    }
}

#[test]
fn a_run_a_mode_or_a_latch_held_logs_at_info() {
    // Grab ends at its drop, so this run's only command is the toggle that started it.
    let started = stop_levels(|rig| {
        rig.toggle(Toggle::Grab);
        rig.settle(T0);
        for (event, now) in [(down(A), T1), (up(A), T1), (down(B), T2), (up(B), T2)] {
            let _ = rig.feed(event, now);
        }
        assert!(!rig.hook.running(), "the drop ended it");
    });
    let toggled = stop_levels(|rig| {
        rig.send(Msg::Presses(true));
        rig.toggle(Toggle::Grab);
        rig.toggle(Toggle::Grab);
        rig.send(Msg::Presses(false));
        rig.settle(T0);
    });
    let latched = stop_levels(|rig| {
        rig.send(Msg::Latches(vec![SHIFT]));
        rig.send(Msg::Latches(Vec::new()));
        rig.settle(T0);
    });
    let info = vec![Level::INFO];
    let want = [info.clone(), info.clone(), info];
    assert_eq!(
        [started, toggled, latched],
        want,
        "the user aimed at the hook"
    );
}

#[test]
fn a_run_only_scrolling_or_presses_held_logs_at_debug() {
    let scrolled = stop_levels(quiet(Msg::Scroll));
    let pressed = stop_levels(quiet(Msg::Presses));
    let debug = vec![Level::DEBUG];
    assert_eq!(
        [scrolled, pressed],
        [debug.clone(), debug],
        "they end at every word"
    );
}
