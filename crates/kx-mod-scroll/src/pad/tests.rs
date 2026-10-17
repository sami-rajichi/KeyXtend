//! The taps of spec §5.2 and the Stop button.

use super::*;
use Change::{Began, Faster, Stopped, Turned};
use kx_module_api::ports::Dir::{Down, Left, Right, Up};

/// Scrolling `dir` at the slow speed.
pub(super) fn slow(dir: Dir) -> Pad {
    Pad::Scrolling { dir, fast: false }
}

/// Scrolling `dir` at the fast speed.
pub(super) fn fast(dir: Dir) -> Pad {
    Pad::Scrolling { dir, fast: true }
}

/// The pad after tapping each of `dirs` from still, with each tap's change.
fn taps(dirs: &[Dir]) -> (Pad, Vec<Change>) {
    let mut pad = Pad::default();
    let changes = dirs.iter().map(|&dir| pad.tap(dir)).collect();
    (pad, changes)
}

/// Every pair of two different ways.
fn turns() -> impl Iterator<Item = (Dir, Dir)> {
    Dir::ALL
        .into_iter()
        .flat_map(|from| Dir::ALL.into_iter().map(move |to| (from, to)))
        .filter(|(from, to)| from != to)
}

#[test]
fn it_starts_still() {
    let pad = Pad::default();
    assert_eq!(pad, Pad::Idle);
    assert_eq!(
        (pad.dir(), pad.fast(), pad.scrolling()),
        (None, false, false)
    );
}

#[test]
fn a_tap_goes_slow_that_way() {
    for dir in Dir::ALL {
        assert_eq!(taps(&[dir]), (slow(dir), vec![Began]));
        assert_eq!((slow(dir).dir(), slow(dir).fast()), (Some(dir), false));
        assert!(slow(dir).scrolling());
    }
}

#[test]
fn the_same_tap_again_goes_fast() {
    for dir in Dir::ALL {
        assert_eq!(taps(&[dir, dir]), (fast(dir), vec![Began, Faster]));
        assert_eq!((fast(dir).dir(), fast(dir).fast()), (Some(dir), true));
    }
}

#[test]
fn a_third_tap_stops() {
    for dir in Dir::ALL {
        assert_eq!(
            taps(&[dir, dir, dir]),
            (Pad::Idle, vec![Began, Faster, Stopped])
        );
    }
}

#[test]
fn another_way_goes_slow_there_from_slow() {
    for (from, to) in turns() {
        assert_eq!(taps(&[from, to]), (slow(to), vec![Began, Turned]));
    }
}

#[test]
fn another_way_goes_slow_there_from_fast() {
    for (from, to) in turns() {
        assert_eq!(
            taps(&[from, from, to]),
            (slow(to), vec![Began, Faster, Turned])
        );
    }
}

#[test]
fn the_new_way_counts_its_own_taps() {
    assert_eq!(taps(&[Down, Up, Up]).0, fast(Up));
    assert_eq!(taps(&[Down, Down, Up, Up, Up]).0, Pad::Idle);
    assert_eq!(taps(&[Left, Right, Left]).0, slow(Left));
}

#[test]
fn stop_stops_at_either_speed() {
    for dir in Dir::ALL {
        for mut pad in [slow(dir), fast(dir)] {
            assert!(pad.stop());
            assert_eq!(pad, Pad::Idle);
        }
    }
}

#[test]
fn stop_while_still_changes_nothing() {
    let mut pad = Pad::Idle;
    assert!(!pad.stop());
    assert_eq!(pad, Pad::Idle);
}

#[test]
fn a_tap_after_a_stop_starts_slow_again() {
    let (mut pad, _) = taps(&[Up, Up]);
    pad.stop();
    assert_eq!(pad.tap(Up), Began);
    assert_eq!(pad, slow(Up));
}

#[test]
fn only_a_new_way_runs_the_tap_checks() {
    assert!(Began.new_way());
    assert!(Turned.new_way());
    assert!(!Faster.new_way());
    assert!(!Stopped.new_way());
}

#[test]
fn the_published_state_follows_the_pad() {
    let still = ScrollChanged {
        live: true,
        dir: None,
        fast: false,
    };
    assert_eq!(Pad::Idle.changed(), still);
    let left = ScrollChanged {
        dir: Some(Left),
        ..still
    };
    assert_eq!(slow(Left).changed(), left);
    let down = ScrollChanged {
        dir: Some(Down),
        fast: true,
        ..still
    };
    assert_eq!(fast(Down).changed(), down);
}
