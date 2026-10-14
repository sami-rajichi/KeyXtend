//! Properties of the host over random runs: the hand, the engine's plans, turns, our input's echoes in order, late or
//! swallowed, cut sends, missed events and panics, then a stop, all checked on what reached the apps.

mod sim;
mod steps;

use std::collections::HashMap;

use kx_module_api::ports::{Button, PointerAct};
use proptest::collection::vec;
use proptest::prelude::*;

use super::fake::{Heard, MAX_ACTS};
use crate::hook::host::Host;
use crate::hook::queue::QUEUE_ROOM;
use sim::Sim;
use steps::{Step, busy, calm, flooded, rough};

/// Random runs per property.
const CASES: u32 = 1_024;
/// Random runs of the flooded property, whose race needs several steps in a row to come up.
const FLOODED_CASES: u32 = 2 * CASES;
/// Most steps of a calm run: too few for the queue to fill, as a full queue lets input pass ahead by design.
const CALM_STEPS: usize = QUEUE_ROOM / (MAX_ACTS + 1);
/// Most steps of a rough run.
const ROUGH_STEPS: usize = 160;
/// Most steps of a flooded run, whose floods make each step long.
const FLOODED_STEPS: usize = ROUGH_STEPS / 2;
const BUTTONS: [Button; 3] = [Button::Left, Button::Right, Button::Middle];

/// What the hand does.
#[derive(Copy, Clone, Debug)]
enum Gesture {
    /// Presses a button it does not hold, else lets it go.
    Flip(Button),
    /// Moves.
    Move,
    /// Presses Ctrl when up, else lets it go.
    Ctrl,
    /// Taps Esc.
    Esc,
}

/// One event of the stream, whether we sent it and with which id, and whether it passes our hooks by.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
struct Ev {
    act: PointerAct,
    ours: bool,
    id: u16,
    unseen: bool,
}

/// What became of one of the hand's events.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum Fate {
    /// It reached the apps, by itself or as our copy.
    Kept,
    /// The engine dropped it, or the OS refused our copy, so a later copy is not the hand's.
    Lost,
    /// Our copy went out in a batch the OS cut after it, which may have cut what let it go.
    Cut,
}

/// What a run left: the apps' stream, the hand's mouse events in the order it made them, and what the engine heard.
#[derive(Debug)]
struct End {
    apps: Vec<Ev>,
    /// How many of `apps` came before the stop, once a last turn went out.
    settled: usize,
    made: Vec<PointerAct>,
    /// Per event of `made`.
    fate: Vec<Fate>,
    /// The buttons the hand still holds at the end.
    holds: Vec<Button>,
    heard: Vec<Heard>,
    /// The host's room changed.
    grew: bool,
    /// A hook dropped our own input.
    ours_dropped: bool,
    /// Waits for our echo that ended as lost.
    lost: u32,
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
            Step::Stall(ms) => sim.stall(*ms),
            Step::Swallow => sim.swallow(),
            Step::Turn => sim.call(Host::turn),
            Step::Check => sim.call(Host::check),
            Step::Release => sim.call(Host::release),
            Step::Takes(n) => sim.takes(*n),
            Step::Plan(s) => s.apply(&mut sim.plan()),
            Step::Flood(n) => (0..*n).for_each(|_| sim.gesture(Gesture::Move)),
            Step::Rush(n, g) => {
                (0..*n).for_each(|_| sim.gesture(Gesture::Move));
                sim.gesture(*g);
            }
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
    let mut seen: Vec<bool> = end.fate.iter().map(|f| *f == Fate::Lost).collect();
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

/// True when, before the stop, no button is left down by our copy of the hand's press after its own release reached
/// the apps, while the hand still lets it go: no Down followed its own Up. A release the engine dropped or the OS
/// refused, or a press whose batch the OS cut after it, excuses it.
fn no_press_after_its_release(end: &End) -> bool {
    let apps = end.apps.get(..end.settled).unwrap_or_default();
    BUTTONS.iter().all(|&b| {
        let last = apps.iter().rev().find(|ev| button_of(ev.act) == Some(b));
        let Some(Ev {
            act: act @ PointerAct::Down(..),
            ours: true,
            ..
        }) = last.copied()
        else {
            return true;
        };
        let Some(k) = end.made.iter().position(|m| *m == act) else {
            return true;
        };
        let up = |m: &PointerAct| matches!(m, PointerAct::Up(x, _) if *x == b);
        let own = end.made.iter().skip(k).position(up).map(|j| k + j);
        let last = end.made.iter().rposition(|m| button_of(*m) == Some(b));
        let kept = |i: Option<usize>| i.is_some_and(|i| end.fate.get(i) != Some(&Fate::Lost));
        let let_go = last.is_some_and(|u| end.made.get(u).is_some_and(up));
        end.fate.get(k) == Some(&Fate::Cut) || !(kept(own) && kept(last) && let_go)
    })
}

/// True when, after the stop, no button the hand let go stays down, even one whose release the engine dropped before
/// its own release panicked.
fn nothing_the_hand_let_go_stays_down(end: &End) -> bool {
    let let_go = BUTTONS.iter().filter(|b| !end.holds.contains(b));
    let down = |b: Button| {
        let last = end
            .apps
            .iter()
            .rev()
            .find(|ev| button_of(ev.act) == Some(b));
        last.is_some_and(|ev| matches!(ev.act, PointerAct::Down(..)))
    };
    let_go.copied().all(|b| !down(b))
}

/// The button `act` presses or lets go.
fn button_of(act: PointerAct) -> Option<Button> {
    match act {
        PointerAct::Down(b, _) | PointerAct::Up(b, _) => Some(b),
        _ => None,
    }
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
    /// own Up, nothing the hand let go stays down, and no wait is lost unless an echo was swallowed.
    #[test]
    fn a_calm_run_keeps_the_hands_order_and_leaves_nothing_of_ours_held(
        steps in vec(calm(), 0..=CALM_STEPS),
    ) {
        let end = play(&steps);
        prop_assert!(in_hand_order(&end), "{:?}", end.apps);
        if !steps.iter().any(|s| matches!(s, Step::Swallow)) {
            prop_assert_eq!(end.lost, 0, "every echo came back, so no wait is lost");
        }
        prop_assert!(nothing_the_hand_let_go_stays_down(&end), "{:?}", end.apps);
        common(&end)?;
    }

    /// With full queues, missed events and panics too, nothing of ours stays held, and a given-up engine is dropped once.
    #[test]
    fn a_rough_run_leaves_nothing_of_ours_held(steps in vec(rough(), 0..=ROUGH_STEPS)) {
        common(&play(&steps))?;
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(FLOODED_CASES))]

    /// With full queues, stalls, carries and panics, where the hand's release may pass ahead of our copy of its press, no
    /// press of ours follows its own release once a turn went out, and nothing the hand let go stays down.
    #[test]
    fn a_flooded_run_lets_no_press_of_ours_follow_its_own_release(
        start in busy(),
        mut steps in vec(flooded(), 0..=FLOODED_STEPS),
    ) {
        steps.insert(0, start);
        let end = play(&steps);
        prop_assert!(no_press_after_its_release(&end), "{:?}", end.apps);
        prop_assert!(nothing_the_hand_let_go_stays_down(&end), "{:?}", end.apps);
        common(&end)?;
    }
}
