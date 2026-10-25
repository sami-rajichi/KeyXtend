//! Properties over random input and commands, checked against what the OS sees and the notes the ring follows.
//! Every engine call is also checked against `MAX_ACTS`.

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
    /// Scrolling starts or ends.
    Scroll(bool),
    /// Presses become wanted, or no longer.
    Wanted(bool),
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
        any::<bool>().prop_map(Step::Scroll),
        any::<bool>().prop_map(Step::Wanted),
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

/// Whether each `HoldStart` in `notes` is closed by exactly one `HoldEnd` or `Fired` before the next, none left open.
fn rings_pair(notes: &[Note]) -> bool {
    let mut open = false;
    for note in notes {
        match note {
            Note::HoldStart { .. } if open => return false,
            Note::HoldEnd | Note::Fired { .. } if !open => return false,
            Note::HoldStart { .. } => open = true,
            Note::HoldEnd | Note::Fired { .. } => open = false,
            _ => {}
        }
    }
    !open
}

/// How many presses `notes` tell.
fn presses(notes: &[Note]) -> usize {
    notes
        .iter()
        .filter(|n| matches!(n, Note::Pressed { .. }))
        .count()
}

/// `notes` with every press note left out.
fn unpressed(notes: &[Note]) -> Vec<Note> {
    let kept = |n: &&Note| !matches!(n, Note::Pressed { .. });
    notes.iter().filter(kept).copied().collect()
}

/// `steps` with presses never wanted.
fn unwanted(steps: &[(u64, Step)]) -> Vec<(u64, Step)> {
    let off = |step: &Step| match step {
        Step::Wanted(_) => Step::Wanted(false),
        other => other.clone(),
    };
    steps.iter().map(|(gap, step)| (*gap, off(step))).collect()
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
        prop_assert_eq!((rig.dropped, rig.sent.len()), (0, 0));
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
        prop_assert_eq!((rig.dropped, rig.sent.len()), (0, 0));
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

    /// While scrolling with no mode and no latch, every event passes, nothing is sent, and each press is noted once.
    #[test]
    fn scrolling_alone_passes_everything_and_notes_each_press_once(steps in run(physical())) {
        let mut rig = Rig::play(&[(0, Step::Scroll(true))], false, Model::Strict);
        rig.more(&steps);
        prop_assert_eq!((rig.dropped, rig.sent.len()), (0, 0));
        prop_assert!(rig.e.needs_hook());
        prop_assert_eq!(rig.misnoted, 0, "{:?}", rig.notes);
        let badge = |n: &Note| matches!(n, Note::Pressed { .. } | Note::Moved { .. });
        prop_assert!(rig.notes.iter().all(badge), "{:?}", rig.notes);
    }

    /// While presses are wanted with no mode, latch or scroll, every event passes, nothing is sent, and only presses
    /// are noted, each once.
    #[test]
    fn wanted_alone_passes_everything_and_notes_each_press_once(steps in run(physical())) {
        let mut rig = Rig::play(&[(0, Step::Wanted(true))], false, Model::Strict);
        rig.more(&steps);
        prop_assert_eq!((rig.dropped, rig.sent.len()), (0, 0));
        prop_assert!(rig.e.needs_hook());
        prop_assert_eq!(rig.misnoted, 0, "{:?}", rig.notes);
        prop_assert!(rig.notes.iter().all(|n| matches!(n, Note::Pressed { .. })), "{:?}", rig.notes);
    }

    /// Each physical press is noted once, at its own event, while scrolling or wanted and never otherwise, whatever the
    /// modes do with it; while presses are wanted the engine always needs the hook.
    #[test]
    fn presses_are_noted_once_and_only_while_scrolling_or_wanted(
        rclick in any::<bool>(),
        steps in mixed(),
    ) {
        let mut rig = Rig::play(&steps, rclick, Model::Strict);
        prop_assert_eq!(rig.unhooked, 0, "wanted, yet no hook needed");
        rig.finish();
        prop_assert_eq!(rig.misnoted, 0, "{:?}", rig.notes);
    }

    /// Wanting presses never changes what happens to an event: the same input in order, drops, OS state and other
    /// notes as without it.
    #[test]
    fn wanted_changes_no_event(rclick in any::<bool>(), steps in mixed()) {
        let mut with = Rig::play(&steps, rclick, Model::Strict);
        let mut without = Rig::play(&unwanted(&steps), rclick, Model::Strict);
        prop_assert_eq!(&with.os.buttons, &without.os.buttons);
        prop_assert_eq!(&with.os.keys, &without.os.keys);
        with.finish();
        without.finish();
        prop_assert_eq!(&with.sent, &without.sent);
        prop_assert_eq!(with.dropped, without.dropped);
        prop_assert_eq!(unpressed(&with.notes), unpressed(&without.notes));
    }

    /// Each hold's ring is closed once, by an end or a fire, before the next starts, so the overlay never keeps one.
    #[test]
    fn each_ring_closes_once(rclick in any::<bool>(), steps in mixed()) {
        let mut rig = Rig::play(&steps, rclick, Model::Strict);
        rig.finish();
        prop_assert!(rings_pair(&rig.notes), "{:?}", rig.notes);
    }

    /// The same after host stops and releases the hook missed.
    #[test]
    fn each_ring_closes_once_after_lapses(
        rclick in any::<bool>(),
        steps in rough(vec![Step::Release, Step::Miss]),
    ) {
        let mut rig = Rig::play(&steps, rclick, Model::Misses);
        rig.finish();
        prop_assert!(rings_pair(&rig.notes), "{:?}", rig.notes);
    }
}
