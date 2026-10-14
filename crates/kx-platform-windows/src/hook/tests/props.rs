//! Properties of the host over random runs: the hand, the engine's plans, turns, our input's echoes, cut sends, missed
//! events and panics, then a stop, all checked on what reached the apps.

mod sim;

use std::collections::HashMap;

use kx_module_api::ports::{Button, Point, PointerAct};
use proptest::collection::vec;
use proptest::prelude::*;
use proptest::sample::select;

use super::fake::{Heard, MAX_ACTS, Plan};
use super::{CFG, CTRL, SHIFT};
use crate::hook::host::Host;
use crate::hook::queue::QUEUE_ROOM;
use sim::{End, Ev, Gesture, Sim};

/// Random runs per property.
const CASES: u32 = 1_024;
/// Most steps of a calm run: too few for the queue to fill, as a full queue lets input pass ahead by design.
const CALM_STEPS: usize = QUEUE_ROOM / (MAX_ACTS + 1);
/// Most steps of a rough run.
const ROUGH_STEPS: usize = 160;
/// Most moves the hand makes at once, past what the queue holds.
const FLOOD: usize = QUEUE_ROOM;
/// Longest pause, past the health period, so lagging hooks go back.
const MAX_PASS_MS: u32 = 2 * CFG.health_ms;
/// Where the engine's own acts land, off the hand's row.
const ENGINE_AT: Point = Point { x: -1, y: -1 };
const BUTTONS: [Button; 3] = [Button::Left, Button::Right, Button::Middle];

/// One thing that happens.
#[derive(Clone, Debug)]
enum Step {
    /// The hand acts, and its event joins the stream.
    Hand(Gesture),
    /// The hand acts while the host sends: its event lands right after our next batch.
    Slip(Gesture),
    /// The stream's next event goes through the hooks.
    Rit,
    /// The stream's next event passes the hooks by, as when Windows dropped them.
    Miss,
    /// The stream catches up, then time passes.
    Pass(u32),
    /// A wake or the deadline timer.
    Turn,
    /// The health timer.
    Check,
    /// The panic hook's let-go.
    Release,
    /// The OS takes this many acts of each batch, or all.
    Takes(Option<usize>),
    /// The engine follows a new plan.
    Plan(Script),
    /// The hand moves this many times at once.
    Flood(usize),
    /// The engine panics from now on, or stops.
    Panic(bool),
}

/// What the engine does with each call.
#[derive(Clone, Debug)]
struct Script {
    drop: bool,
    on: Vec<PointerAct>,
    settle: Vec<PointerAct>,
    release: Vec<PointerAct>,
    notes: bool,
}

impl Script {
    fn apply(&self, plan: &mut Plan) {
        plan.drop = self.drop;
        plan.on_acts.clone_from(&self.on);
        plan.settle_acts.clone_from(&self.settle);
        plan.release_acts.clone_from(&self.release);
        plan.notes = self.notes;
    }
}

fn gesture() -> impl Strategy<Value = Gesture> {
    prop_oneof![
        3 => select(BUTTONS.to_vec()).prop_map(Gesture::Flip),
        3 => Just(Gesture::Move),
        1 => Just(Gesture::Ctrl),
        1 => Just(Gesture::Esc),
    ]
}

fn acts() -> impl Strategy<Value = Vec<PointerAct>> {
    let act = prop_oneof![
        Just(PointerAct::Move(ENGINE_AT)),
        select(BUTTONS.to_vec()).prop_map(|b| PointerAct::Down(b, ENGINE_AT)),
        select(BUTTONS.to_vec()).prop_map(|b| PointerAct::Up(b, ENGINE_AT)),
        select(vec![CTRL, SHIFT]).prop_map(PointerAct::KeyDown),
        select(vec![CTRL, SHIFT]).prop_map(PointerAct::KeyUp),
    ];
    vec(act, 0..=MAX_ACTS)
}

fn script() -> impl Strategy<Value = Script> {
    (any::<bool>(), acts(), acts(), acts(), any::<bool>()).prop_map(
        |(drop, on, settle, release, notes)| Script {
            drop,
            on,
            settle,
            release,
            notes,
        },
    )
}

/// Steps where the queue never fills and the engine never fails.
fn calm() -> impl Strategy<Value = Step> {
    prop_oneof![
        4 => gesture().prop_map(Step::Hand),
        1 => gesture().prop_map(Step::Slip),
        4 => Just(Step::Rit),
        1 => (0..=MAX_PASS_MS).prop_map(Step::Pass),
        2 => Just(Step::Turn),
        1 => Just(Step::Check),
        1 => Just(Step::Release),
        1 => prop_oneof![Just(None), (0..=MAX_ACTS).prop_map(Some)].prop_map(Step::Takes),
        1 => script().prop_map(Step::Plan),
    ]
}

/// Calm steps with floods, missed events and panics.
fn rough() -> impl Strategy<Value = Step> {
    prop_oneof![
        12 => calm(),
        1 => (1..=FLOOD).prop_map(Step::Flood),
        1 => Just(Step::Miss),
        1 => any::<bool>().prop_map(Step::Panic),
    ]
}

/// Runs `steps`, then stops.
fn play(steps: &[Step]) -> End {
    let mut sim = Sim::new();
    for step in steps {
        match step {
            Step::Hand(g) => sim.gesture(*g),
            Step::Slip(g) => sim.slip(*g),
            Step::Rit => sim.step(true),
            Step::Miss => sim.step(false),
            Step::Pass(ms) => sim.pass(*ms),
            Step::Turn => sim.call(Host::turn),
            Step::Check => sim.call(Host::check),
            Step::Release => sim.call(Host::release),
            Step::Takes(n) => sim.takes(*n),
            Step::Plan(s) => s.apply(&mut sim.plan()),
            Step::Flood(n) => (0..*n).for_each(|_| sim.gesture(Gesture::Move)),
            Step::Panic(on) => sim.plan().panic = *on,
        }
    }
    sim.end()
}

/// The last event of ours for a button or key that is a press, which nothing let go after it.
fn ours_held(apps: &[Ev]) -> Option<PointerAct> {
    let mut last = HashMap::new();
    for ev in apps {
        let key = match ev.act {
            PointerAct::Down(b, _) | PointerAct::Up(b, _) => Ok(b),
            PointerAct::KeyDown(s) | PointerAct::KeyUp(s) => Err(s),
            PointerAct::Move(_) => continue,
        };
        last.insert(key, *ev);
    }
    last.into_values()
        .find(|ev| ev.ours && matches!(ev.act, PointerAct::Down(..) | PointerAct::KeyDown(_)))
        .map(|ev| ev.act)
}

/// True when the hand's events first reach the apps in the order it made them, each sent by itself or by us. A later
/// copy, or one of a lost event, is our let-go at the hand's newest point.
fn in_hand_order(end: &End) -> bool {
    let mut seen = end.lost.clone();
    let mut newest = None;
    for ev in &end.apps {
        let Some(k) = end.made.iter().position(|m| *m == ev.act) else {
            continue;
        };
        if std::mem::replace(&mut seen[k], true) {
            continue;
        }
        if newest.is_some_and(|n| k < n) {
            return false;
        }
        newest = Some(k);
    }
    true
}

/// Every run ends with nothing of ours held, the room as reserved, our input never dropped, and the engine dropped
/// once, last.
fn common(end: &End) -> Result<(), TestCaseError> {
    prop_assert_eq!(ours_held(&end.apps), None, "{:?}", end.apps);
    prop_assert!(!end.grew, "the host's room changed");
    prop_assert!(!end.ours_dropped, "a hook dropped our own input");
    let drops = end.heard.iter().filter(|h| **h == Heard::Dropped).count();
    prop_assert_eq!(drops, 1, "{:?}", end.heard);
    prop_assert_eq!(end.heard.last(), Some(&Heard::Dropped));
    Ok(())
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(CASES))]

    /// With room in the queue and a whole engine, the hand's events reach the apps in its order, so no Down follows its
    /// own Up.
    #[test]
    fn a_calm_run_keeps_the_hands_order_and_leaves_nothing_of_ours_held(
        steps in vec(calm(), 0..=CALM_STEPS),
    ) {
        let end = play(&steps);
        prop_assert!(in_hand_order(&end), "{:?}", end.apps);
        common(&end)?;
    }

    /// With full queues, missed events and panics too, nothing of ours stays held, and a given-up engine is dropped once.
    #[test]
    fn a_rough_run_leaves_nothing_of_ours_held(steps in vec(rough(), 0..=ROUGH_STEPS)) {
        common(&play(&steps))?;
    }
}
