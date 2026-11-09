//! Properties over random toggles, latches, scrolling, wishes for presses, asks and physical input: the hook runs only
//! while a mode, a latch, scrolling, wanted presses or a press needs it, the keys show off whenever it does not run,
//! and a stop leaves no key or button down the hand let go. Presses and releases are told exactly while an asker or
//! the panel wants them.

use crate::rig::{DPI, Heard, Rig, T0, down, up, wants};
use kx_mod_mouse::{MAX_LATCHES, Msg};
use kx_module_api::Mono;
use kx_module_api::ports::{
    AssistChanged, Button, GrabState, Point, PointerAct, PointerEvent, PointerPressed,
    PointerReleased, PressAsker, Scan, Toggle,
};
use proptest::collection::vec;
use proptest::prelude::*;
use proptest::sample::{select, subsequence};
use std::collections::{HashMap, HashSet};

/// Random runs per property.
const CASES: u32 = 512;
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
/// A click outside our windows as told on the bus: its press, then its release.
const CLICK: [Heard; 2] = [
    Heard::Pressed(PointerPressed { ours: false }),
    Heard::Released(PointerReleased { ours: false }),
];

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
    /// The pad starts or stops scrolling.
    Scroll(bool),
    /// An asker wants presses told, or no longer.
    Presses(PressAsker, bool),
    /// An open panel wants presses told, or no longer.
    Panel(bool),
    /// The keyboard asks for the state.
    Ask,
}

/// Any asker.
fn asker() -> impl Strategy<Value = PressAsker> {
    select(PressAsker::ALL.to_vec())
}

fn step() -> impl Strategy<Value = Step> {
    let mods: Vec<Scan> = (MOD_BASE..).take(MAX_LATCHES + 1).map(Scan::new).collect();
    // Presses and toggles weigh most, so runs often end mid-hold or mid-carry.
    prop_oneof![
        4 => any::<bool>().prop_map(|ours| Step::Left { ours }),
        2 => (-REACH..=REACH, -REACH..=REACH).prop_map(|(dx, dy)| Step::Move(dx, dy)),
        1 => Just(Step::Esc),
        3 => select(vec![Toggle::RightClick, Toggle::Grab]).prop_map(Step::Toggle),
        1 => subsequence(mods.clone(), 0..=mods.len()).prop_map(Step::Latch),
        1 => any::<bool>().prop_map(Step::Scroll),
        1 => (asker(), any::<bool>()).prop_map(|(asker, on)| Step::Presses(asker, on)),
        1 => any::<bool>().prop_map(Step::Panel),
        1 => Just(Step::Ask),
    ]
}

/// One wish for presses, on or off.
#[derive(Copy, Clone, Debug)]
enum Wish {
    /// An asker's.
    Asker(PressAsker, bool),
    /// The open panel's.
    Panel(bool),
}

fn wish() -> impl Strategy<Value = Wish> {
    prop_oneof![
        (asker(), any::<bool>()).prop_map(|(asker, on)| Wish::Asker(asker, on)),
        any::<bool>().prop_map(Wish::Panel),
    ]
}

/// Keeps `asker`'s wish in the model's set: on adds it, off takes it out, and each is idempotent.
fn keep(askers: &mut HashSet<PressAsker>, asker: PressAsker, on: bool) {
    if on {
        askers.insert(asker);
    } else {
        askers.remove(&asker);
    }
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
            Step::Left { ours } => {
                self.down = false;
                PointerEvent::Up { button, at, ours }
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
        Step::Scroll(on) => Some(Msg::Scroll(*on)),
        Step::Presses(asker, on) => Some(Msg::Presses(wants(*asker, *on))),
        Step::Panel(on) => Some(Msg::PanelPresses(*on)),
        Step::Ask => Some(Msg::Ask),
        _ => None,
    }
}

/// What the OS sees: the buttons down, and each key pressed minus released.
#[derive(Default)]
struct Os {
    buttons: HashSet<Button>,
    keys: HashMap<Scan, i64>,
}

impl Os {
    /// Takes our input, in order.
    fn send(&mut self, acts: Vec<PointerAct>) {
        for act in acts {
            match act {
                PointerAct::Down(button, _) => {
                    self.buttons.insert(button);
                }
                PointerAct::Up(button, _) => {
                    self.buttons.remove(&button);
                }
                PointerAct::KeyDown(key) => *self.keys.entry(key).or_default() += 1,
                PointerAct::KeyUp(key) => *self.keys.entry(key).or_default() -= 1,
                PointerAct::Move(_) => {}
            }
        }
        self.keys.retain(|_, n| *n != 0);
    }

    /// Takes a physical event that reached it.
    fn pass(&mut self, event: PointerEvent) {
        match event {
            PointerEvent::Down { button, .. } => {
                self.buttons.insert(button);
            }
            PointerEvent::Up { button, .. } => {
                self.buttons.remove(&button);
            }
            PointerEvent::Move { .. } | PointerEvent::Esc => {}
        }
    }
}

/// Runs one step at `now`, then settles the hook as its wake or deadline would and pumps.
fn run(rig: &mut Rig, os: &mut Os, hand: &mut Hand, step: &Step, now: Mono) {
    match (hand.event(step), message(step)) {
        (Some(event), _) if rig.hook.running() => {
            let dropped = rig.hook.feed(event, now);
            // Our input goes first, and an event the hook lets through waits behind it. Every command settles in its
            // own step, so this feed's `settle` pushes nothing.
            os.send(rig.hook.take_sent());
            if !dropped {
                os.pass(event);
            }
        }
        // With no hook, physical input reaches the OS untouched.
        (Some(event), _) => os.pass(event),
        (None, Some(msg)) => {
            rig.actor.handle(msg);
        }
        (None, None) => {}
    }
    rig.settle(now);
    os.send(rig.hook.take_sent());
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(CASES))]

    #[test]
    fn the_hook_runs_while_needed_and_a_stop_leaves_nothing_down(
        steps in vec((0..=MAX_GAP_US, step()), 0..=MAX_STEPS),
    ) {
        let mut rig = Rig::new();
        rig.begin();
        let (mut hand, mut os) = (Hand { at: START, down: false }, Os::default());
        let (mut now, mut held, mut scrolling, mut panel) = (T0, false, false, false);
        let mut askers = HashSet::new();
        let mut state = AssistChanged { live: true, rclick: false, grab: GrabState::Off };
        for (gap, step) in &steps {
            now = Mono::from_us(now.as_us() + gap);
            match step {
                Step::Latch(keys) => held = !keys.is_empty(),
                Step::Scroll(on) => scrolling = *on,
                Step::Presses(asker, on) => keep(&mut askers, *asker, *on),
                Step::Panel(on) => panel = *on,
                _ => {}
            }
            run(&mut rig, &mut os, &mut hand, step, now);
            for heard in rig.take() {
                if let Heard::Changed(changed) = heard {
                    state = changed;
                }
            }
            let on = state.rclick || state.grab != GrabState::Off;
            let wanted = on || held || scrolling || !askers.is_empty() || panel;
            prop_assert!(!wanted || rig.hook.running(), "a mode, a latch, scrolling or presses need the hook");
            prop_assert!(rig.hook.running() || !on, "the keys show off while no hook runs");
            let needed = wanted || hand.down;
            prop_assert!(needed || !rig.hook.running(), "nothing needs the hook");
            prop_assert_eq!(rig.hook.starts() - rig.hook.stops(), usize::from(rig.hook.running()));
        }
        prop_assert!(!rig.send(Msg::Stop));
        prop_assert!(!rig.hook.running());
        os.send(rig.hook.take_sent());
        prop_assert_eq!(&os.keys, &HashMap::new(), "a key stays down");
        let held = |b: &Button| *b == Button::Left && hand.down;
        prop_assert!(os.buttons.iter().all(held), "a button stays down: {:?}", os.buttons);
    }

    /// For any interleaving of wishes, the hook runs, and each click's press and release is told, exactly while the
    /// set of askers is not empty or the panel wants presses.
    #[test]
    fn presses_and_releases_are_told_exactly_while_an_asker_or_the_panel_wants_them(
        wishes in vec(wish(), 0..=MAX_STEPS),
    ) {
        let mut rig = Rig::new();
        rig.begin();
        let (mut askers, mut panel) = (HashSet::new(), false);
        for &wish in &wishes {
            rig.send(match wish {
                Wish::Asker(asker, on) => {
                    keep(&mut askers, asker, on);
                    Msg::Presses(wants(asker, on))
                }
                Wish::Panel(on) => {
                    panel = on;
                    Msg::PanelPresses(on)
                }
            });
            rig.settle(T0);
            let want = !askers.is_empty() || panel;
            prop_assert_eq!(rig.hook.running(), want, "{:?} {}", askers, panel);
            rig.take();
            if want {
                prop_assert!(!rig.feed(down(START), T0));
                prop_assert!(!rig.feed(up(START), T0));
                prop_assert_eq!(rig.take(), CLICK.to_vec());
            }
        }
    }
}
