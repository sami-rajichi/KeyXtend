//! Properties over any run of taps and stops.

use super::tests::{fast, slow};
use super::*;
use proptest::collection::vec;
use proptest::prelude::*;
use proptest::sample::select;

/// Random runs per property.
const CASES: u32 = 512;
/// Most steps in one run.
const MAX_STEPS: usize = 60;

/// One press on the pad.
#[derive(Copy, Clone, Debug)]
enum Step {
    /// An arrow.
    Tap(Dir),
    /// The Stop button, or a stop rule.
    Stop,
}

fn way() -> impl Strategy<Value = Dir> {
    select(Dir::ALL.to_vec())
}

fn run() -> impl Strategy<Value = Vec<Step>> {
    vec(
        prop_oneof![4 => way().prop_map(Step::Tap), 1 => Just(Step::Stop)],
        0..=MAX_STEPS,
    )
}

/// The pad after `steps` from still.
fn play(steps: &[Step]) -> Pad {
    let mut pad = Pad::default();
    for &step in steps {
        match step {
            Step::Tap(dir) => {
                pad.tap(dir);
            }
            Step::Stop => {
                pad.stop();
            }
        }
    }
    pad
}

/// Whether `change` is what a tap on `dir` does from `before` to `after`.
fn fits(before: Pad, dir: Dir, after: Pad, change: Change) -> bool {
    match change {
        Change::Began => before == Pad::Idle && after == slow(dir),
        Change::Faster => before == slow(dir) && after == fast(dir),
        Change::Stopped => before == fast(dir) && after == Pad::Idle,
        Change::Turned => before.scrolling() && before.dir() != Some(dir) && after == slow(dir),
    }
}

/// Whether the pad's views agree: a way and a speed only while scrolling.
fn whole(pad: Pad) -> bool {
    pad.dir().is_some() == pad.scrolling() && (!pad.fast() || pad.scrolling())
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(CASES))]

    /// Each tap's change agrees with the states before and after it, and each stop says whether it was scrolling.
    #[test]
    fn each_step_says_what_it_changed(steps in run()) {
        let mut pad = Pad::default();
        for step in steps {
            let before = pad;
            match step {
                Step::Tap(dir) => {
                    let change = pad.tap(dir);
                    prop_assert!(fits(before, dir, pad, change), "{before:?} {dir:?} {pad:?} {change:?}");
                }
                Step::Stop => prop_assert_eq!(pad.stop(), before.scrolling()),
            }
            prop_assert!(whole(pad), "{pad:?}");
        }
    }

    /// After any run and a stop, three taps of one way go slow, fast, then still.
    #[test]
    fn three_taps_of_one_way_from_still_return_to_still(steps in run(), dir in way()) {
        let mut pad = play(&steps);
        pad.stop();
        let changes = [pad.tap(dir), pad.tap(dir), pad.tap(dir)];
        prop_assert_eq!(changes, [Change::Began, Change::Faster, Change::Stopped]);
        prop_assert_eq!(pad, Pad::Idle);
    }

    /// From any state, at most three taps of one way bring the pad to rest.
    #[test]
    fn tapping_one_way_always_comes_to_rest(steps in run(), dir in way()) {
        let mut pad = play(&steps);
        let rested = (0..3).any(|_| pad.tap(dir) == Change::Stopped);
        prop_assert!(rested);
        prop_assert_eq!(pad, Pad::Idle);
    }

    /// After any run, a stop leaves the pad still.
    #[test]
    fn every_stop_leaves_it_still(steps in run()) {
        let mut pad = play(&steps);
        let was = pad.scrolling();
        prop_assert_eq!(pad.stop(), was);
        prop_assert_eq!(pad, Pad::Idle);
        prop_assert!(!pad.stop());
    }
}
