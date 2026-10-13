//! Properties over random input and commands, checked against what the OS sees: the events passed and the input
//! sent. Every engine call is also checked against `MAX_ACTS`.

mod rig;

use super::*;
use kx_module_api::ports::{Button, Toggle};
use proptest::collection::vec;
use proptest::prelude::*;
use proptest::sample::{select, subsequence};
use rig::{Model, Rig};

/// Random runs per property.
const CASES: u32 = 512;
/// Most steps in one run.
const MAX_STEPS: usize = 80;
/// The hold time, in µs.
const HOLD: u64 = 1_500_000;
/// Longest gap between two steps, past the hold time so holds fire.
const MAX_GAP_US: u64 = 2 * HOLD;
/// The still radius, in logical pixels.
const STILL: u32 = 7;
/// Screen DPIs a press may come from: 100 % (the default), none reported, 125 % and 150 %.
const DPIS: [u32; 4] = [96, 0, 120, 144];
/// More modifier codes than `MAX_LATCHES`, so a long latch set is cut.
const MODS: usize = MAX_LATCHES + 2;
/// The first modifier code.
const MOD_BASE: u16 = 0x70;
/// Farthest one move goes on each axis, in pixels: past every scaled radius, and within it.
const REACH: i32 = 30;
/// Where the pointer starts.
const START: Point = Point { x: 500, y: 500 };

/// One random step: physical input or a command.
#[derive(Clone, Debug)]
enum Step {
    /// The left button flips: down (outside or on ours, at a DPI) when up, else up.
    Left { ours: bool, dpi: u32 },
    /// Another button clicks: down, then up.
    Other(Button),
    /// The pointer moves by this much.
    Move(i32, i32),
    /// The Esc key goes down.
    Esc,
    /// The host's timer runs.
    Tick,
    /// A mode toggle.
    Toggle(Toggle),
    /// Right-click is set.
    Rclick(bool),
    /// The latches change.
    Latch(Vec<Scan>),
    /// The host stops mid-run and starts again with the same engine.
    Release,
    /// The hook misses the left button's release.
    Miss,
}

fn physical() -> impl Strategy<Value = Step> {
    prop_oneof![
        (any::<bool>(), select(DPIS.to_vec())).prop_map(|(ours, dpi)| Step::Left { ours, dpi }),
        select(vec![Button::Right, Button::Middle]).prop_map(Step::Other),
        (-REACH..=REACH, -REACH..=REACH).prop_map(|(dx, dy)| Step::Move(dx, dy)),
        Just(Step::Esc),
        Just(Step::Tick),
    ]
}

fn command() -> impl Strategy<Value = Step> {
    let mods: Vec<Scan> = (MOD_BASE..).take(MODS).map(Scan::new).collect();
    prop_oneof![
        select(vec![Toggle::RightClick, Toggle::Grab]).prop_map(Step::Toggle),
        any::<bool>().prop_map(Step::Rclick),
        subsequence(mods, 0..=MODS).prop_map(Step::Latch),
    ]
}

/// Steps, each after a random gap in µs.
fn run(step: impl Strategy<Value = Step>) -> impl Strategy<Value = Vec<(u64, Step)>> {
    vec((0..=MAX_GAP_US, step), 0..=MAX_STEPS)
}

/// Well-formed physical input mixed with commands.
fn mixed() -> impl Strategy<Value = Vec<(u64, Step)>> {
    run(prop_oneof![3 => physical(), 1 => command()])
}

/// Mixed input with `lapses` (host stops, missed releases) now and then.
fn rough(lapses: Vec<Step>) -> impl Strategy<Value = Vec<(u64, Step)>> {
    run(prop_oneof![6 => physical(), 2 => command(), 1 => select(lapses)])
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(CASES))]

    /// Once the hand lets go and the host stops, no button and no modifier is down.
    #[test]
    fn nothing_stays_down_after_release(rclick in any::<bool>(), steps in mixed()) {
        let mut rig = Rig::play(&steps, rclick, Model::Strict);
        rig.finish();
        prop_assert!(rig.os.idle());
    }

    /// Every press, dropped or not, is released at the OS by its own release; only a carry keeps the left down.
    #[test]
    fn every_dropped_press_gets_its_release(rclick in any::<bool>(), steps in mixed()) {
        let rig = Rig::play(&steps, rclick, Model::Strict);
        prop_assert_eq!(rig.unmatched, 0);
    }

    /// Whenever the engine needs no hook, the OS holds no modifier and no button the hand does not.
    #[test]
    fn no_hook_needed_means_nothing_held(rclick in any::<bool>(), steps in mixed()) {
        let rig = Rig::play(&steps, rclick, Model::Strict);
        prop_assert_eq!(rig.leaks, 0);
    }

    /// With both modes off and no latches, every event passes and nothing is sent or noted.
    #[test]
    fn with_nothing_on_everything_passes(steps in run(physical())) {
        let mut rig = Rig::play(&steps, false, Model::Strict);
        rig.finish();
        prop_assert_eq!((rig.dropped, rig.sent), (0, 0));
        prop_assert!(rig.notes.is_empty());
        prop_assert!(!rig.e.needs_hook());
    }

    /// After any run, turning both modes off and clearing the latches lets everything pass again.
    #[test]
    fn calmed_down_everything_passes(
        rclick in any::<bool>(),
        steps in mixed(),
        after in run(physical()),
    ) {
        let mut rig = Rig::play(&steps, rclick, Model::Strict);
        rig.calm();
        rig.more(&after);
        rig.finish();
        prop_assert_eq!((rig.dropped, rig.sent), (0, 0));
        prop_assert!(rig.notes.is_empty());
        prop_assert!(!rig.e.needs_hook());
    }

    /// After host stops mid-run the engine keeps working: nothing sticks, and no hook needed means nothing held.
    #[test]
    fn nothing_sticks_after_a_stop_mid_run(
        rclick in any::<bool>(),
        steps in rough(vec![Step::Release]),
    ) {
        let mut rig = Rig::play(&steps, rclick, Model::Stops);
        rig.finish();
        prop_assert!(rig.os.idle());
        prop_assert_eq!(rig.leaks, 0);
    }

    /// After releases the hook missed, the next click leaves nothing stuck (spec 5.1).
    #[test]
    fn the_next_click_recovers_from_a_missed_release(
        rclick in any::<bool>(),
        steps in rough(vec![Step::Release, Step::Miss]),
    ) {
        let mut rig = Rig::play(&steps, rclick, Model::Misses);
        rig.recover();
        prop_assert!(rig.os.idle());
    }
}
