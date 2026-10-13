//! Properties over random toggles, latches, asks and physical input: the hook runs while a mode or a latch needs it,
//! the keyboard's state is off whenever it does not, and a stop leaves no key down.

use crate::rig::{DPI, Heard, Rig, T0};
use kx_mod_mouse::{MAX_LATCHES, Msg};
use kx_module_api::Mono;
use kx_module_api::ports::{
    AssistChanged, Button, GrabState, Point, PointerAct, PointerEvent, Scan, Toggle,
};
use proptest::collection::vec;
use proptest::prelude::*;
use proptest::sample::{select, subsequence};
use std::collections::HashMap;

/// Random runs per property.
const CASES: u32 = 256;
/// Most steps in one run.
const MAX_STEPS: usize = 60;
/// Longest gap between two steps, in µs: past the longest hold the settings allow, so holds fire.
const MAX_GAP_US: u64 = 4_000_000;
/// Farthest one move goes on each axis, in pixels: past the still radius, and within it.
const REACH: i32 = 30;
/// The first modifier code; there are more codes than `MAX_LATCHES`.
const MOD_BASE: u16 = 0x70;
/// Where the pointer starts.
const START: Point = Point { x: 500, y: 500 };

/// One random step.
#[derive(Clone, Debug)]
enum Step {
    /// The left button flips, outside our windows or on them.
    Left { ours: bool },
    /// The pointer moves by this much.
    Move(i32, i32),
    /// The Esc key goes down.
    Esc,
    /// The keyboard flips a mode.
    Toggle(Toggle),
    /// The keyboard's latches change.
    Latch(Vec<Scan>),
    /// The keyboard asks for the state.
    Ask,
}

fn step() -> impl Strategy<Value = Step> {
    let mods: Vec<Scan> = (MOD_BASE..).take(MAX_LATCHES + 1).map(Scan::new).collect();
    prop_oneof![
        any::<bool>().prop_map(|ours| Step::Left { ours }),
        (-REACH..=REACH, -REACH..=REACH).prop_map(|(dx, dy)| Step::Move(dx, dy)),
        Just(Step::Esc),
        select(vec![Toggle::RightClick, Toggle::Grab]).prop_map(Step::Toggle),
        subsequence(mods.clone(), 0..=mods.len()).prop_map(Step::Latch),
        Just(Step::Ask),
    ]
}

/// The physical mouse: where it is and whether the left button is down.
struct Hand {
    at: Point,
    down: bool,
}

impl Hand {
    /// The event `step` makes, if it is physical input.
    fn event(&mut self, step: &Step) -> Option<PointerEvent> {
        let button = Button::Left;
        let at = self.at;
        Some(match *step {
            Step::Left { ours } if !self.down => {
                self.down = true;
                PointerEvent::Down {
                    button,
                    at,
                    ours,
                    dpi: DPI,
                }
            }
            Step::Left { .. } => {
                self.down = false;
                PointerEvent::Up { button, at }
            }
            Step::Move(dx, dy) => {
                self.at = Point {
                    x: at.x + dx,
                    y: at.y + dy,
                };
                PointerEvent::Move { at: self.at }
            }
            Step::Esc => PointerEvent::Esc,
            _ => return None,
        })
    }
}

/// The message `step` sends the actor, if it is a command.
fn message(step: &Step) -> Option<Msg> {
    match step {
        Step::Toggle(which) => Some(Msg::Toggle(*which)),
        Step::Latch(held) => Some(Msg::Latches(held.clone())),
        Step::Ask => Some(Msg::Ask),
        _ => None,
    }
}

/// Runs one step at `now`, then settles the hook as its wake or deadline would and pumps.
fn run(rig: &mut Rig, hand: &mut Hand, step: &Step, now: Mono) {
    match (hand.event(step), message(step)) {
        // The hook sees physical input only while it runs.
        (Some(event), _) => {
            if rig.hook.running() {
                let _ = rig.hook.feed(event, now);
            }
        }
        (None, Some(msg)) => {
            rig.actor.handle(msg);
        }
        (None, None) => {}
    }
    rig.settle(now);
}

/// Each key code pressed minus released in `sent`.
fn keys_down(sent: &[PointerAct]) -> HashMap<Scan, i64> {
    let mut down = HashMap::new();
    for act in sent {
        match act {
            PointerAct::KeyDown(key) => *down.entry(*key).or_default() += 1,
            PointerAct::KeyUp(key) => *down.entry(*key).or_default() -= 1,
            _ => {}
        }
    }
    down.retain(|_, n| *n != 0);
    down
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(CASES))]

    #[test]
    fn the_hook_runs_while_needed_and_a_stop_leaves_no_key_down(
        steps in vec((0..=MAX_GAP_US, step()), 0..=MAX_STEPS),
    ) {
        let mut rig = Rig::new();
        rig.begin();
        let mut hand = Hand { at: START, down: false };
        let (mut now, mut held) = (T0, false);
        let mut state = AssistChanged { live: true, rclick: false, grab: GrabState::Off };
        for (gap, step) in &steps {
            now = Mono::from_us(now.as_us() + gap);
            if let Step::Latch(keys) = step {
                held = !keys.is_empty();
            }
            run(&mut rig, &mut hand, step, now);
            for heard in rig.take() {
                if let Heard::Changed(changed) = heard {
                    state = changed;
                }
            }
            let on = state.rclick || state.grab != GrabState::Off;
            prop_assert!(!(on || held) || rig.hook.running(), "a mode or a latch needs the hook");
            prop_assert!(rig.hook.running() || !on, "the keys show off while no hook runs");
            prop_assert_eq!(rig.hook.starts() - rig.hook.stops(), usize::from(rig.hook.running()));
        }
        prop_assert!(!rig.send(Msg::Stop));
        prop_assert!(!rig.hook.running());
        prop_assert_eq!(keys_down(&rig.hook.sent()), HashMap::new());
    }
}
