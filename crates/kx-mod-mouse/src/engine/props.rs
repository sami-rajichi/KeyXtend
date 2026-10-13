//! Properties over random well-formed input and commands, checked against what the OS sees: the events passed and
//! the input sent.

use super::*;
use kx_module_api::ports::{Button, Toggle};
use proptest::collection::vec;
use proptest::prelude::*;
use proptest::sample::{select, subsequence};
use std::collections::BTreeSet;

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
/// Screen DPIs a press may come from: 100 %, 125 % and 150 %.
const DPIS: [u32; 3] = [96, 120, 144];
/// Modifiers a latch set is drawn from: Ctrl, Shift, Alt.
const MODS: [Scan; 3] = [Scan::new(0x1D), Scan::new(0x2A), Scan::new(0x38)];
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
    prop_oneof![
        select(vec![Toggle::RightClick, Toggle::Grab]).prop_map(Step::Toggle),
        any::<bool>().prop_map(Step::Rclick),
        subsequence(MODS.to_vec(), 0..=MODS.len()).prop_map(Step::Latch),
    ]
}

/// Steps, each after a random gap in µs.
fn run(step: impl Strategy<Value = Step>) -> impl Strategy<Value = Vec<(u64, Step)>> {
    vec((0..=MAX_GAP_US, step), 0..=MAX_STEPS)
}

/// Physical input mixed with commands.
fn mixed() -> impl Strategy<Value = Vec<(u64, Step)>> {
    run(prop_oneof![3 => physical(), 1 => command()])
}

/// What the OS sees as down; it refuses a second press or a release of something up.
#[derive(Default)]
struct Os {
    buttons: [bool; 3],
    keys: BTreeSet<Scan>,
}

fn slot(button: Button) -> usize {
    match button {
        Button::Left => 0,
        Button::Right => 1,
        Button::Middle => 2,
    }
}

impl Os {
    fn button(&mut self, button: Button, down: bool) {
        let held = &mut self.buttons[slot(button)];
        assert_ne!(*held, down, "{button:?} went the same way twice");
        *held = down;
    }

    fn key(&mut self, key: Scan, down: bool) {
        let fresh = if down {
            self.keys.insert(key)
        } else {
            self.keys.remove(&key)
        };
        assert!(fresh, "a key went the same way twice");
    }

    fn see(&mut self, event: PointerEvent) {
        match event {
            PointerEvent::Down { button, .. } => self.button(button, true),
            PointerEvent::Up { button, .. } => self.button(button, false),
            PointerEvent::Move { .. } | PointerEvent::Esc => {}
        }
    }

    fn act(&mut self, act: PointerAct) {
        match act {
            PointerAct::Down(button, _) => self.button(button, true),
            PointerAct::Up(button, _) => self.button(button, false),
            PointerAct::KeyDown(key) => self.key(key, true),
            PointerAct::KeyUp(key) => self.key(key, false),
            PointerAct::Move(_) => {}
        }
    }

    fn idle(&self) -> bool {
        self.buttons == [false; 3] && self.keys.is_empty()
    }
}

/// The engine, the hand on the mouse, what the OS sees, and counts for the properties.
struct Rig {
    e: Engine,
    now: u64,
    at: Point,
    left: bool,
    os: Os,
    acts: Vec<PointerAct>,
    notes: Vec<Note>,
    /// Events the engine dropped.
    dropped: usize,
    /// Input the engine sent.
    sent: usize,
    /// Releases after which the OS still held that button or a modifier, outside a carry.
    unmatched: usize,
}

impl Rig {
    fn new(rclick: bool) -> Self {
        let cfg = EngineCfg {
            hold_us: HOLD,
            still_px: STILL,
            rclick,
        };
        Self {
            e: Engine::new(cfg),
            now: 0,
            at: START,
            left: false,
            os: Os::default(),
            acts: Vec::new(),
            notes: Vec::new(),
            dropped: 0,
            sent: 0,
            unmatched: 0,
        }
    }

    fn play(steps: &[(u64, Step)], rclick: bool) -> Self {
        let mut rig = Self::new(rclick);
        for (gap, step) in steps {
            rig.now += gap;
            rig.step(step);
        }
        rig
    }

    fn step(&mut self, step: &Step) {
        match *step {
            Step::Left { ours, dpi } => self.flip(ours, dpi),
            Step::Other(button) => self.click(button),
            Step::Move(dx, dy) => {
                self.at = Point {
                    x: self.at.x + dx,
                    y: self.at.y + dy,
                };
                self.event(PointerEvent::Move { at: self.at });
            }
            Step::Esc => self.event(PointerEvent::Esc),
            Step::Tick => self.e.tick(Mono::from_us(self.now), &mut self.acts),
            Step::Toggle(which) => self.e.toggle(which, &mut self.acts),
            Step::Rclick(on) => self.e.set_rclick(on, &mut self.acts),
            Step::Latch(ref latches) => self.e.set_latches(latches.clone()),
        }
        self.send();
    }

    /// Runs one event; the OS sees it unless the engine drops it, then the engine's input.
    fn event(&mut self, event: PointerEvent) {
        if self.e.on(event, Mono::from_us(self.now), &mut self.acts) {
            self.dropped += 1;
        } else {
            self.os.see(event);
        }
        self.send();
    }

    /// The OS takes the engine's input, and the notes are drained as the host does.
    fn send(&mut self) {
        self.sent += self.acts.len();
        for &act in &self.acts {
            self.os.act(act);
        }
        self.acts.clear();
        self.e.take_notes(&mut self.notes);
    }

    fn flip(&mut self, ours: bool, dpi: u32) {
        let (button, at) = (Button::Left, self.at);
        self.left = !self.left;
        if self.left {
            self.event(PointerEvent::Down {
                button,
                at,
                ours,
                dpi,
            });
            return;
        }
        self.event(PointerEvent::Up { button, at });
        let carry = self.e.grab() == GrabState::Carrying;
        let held = self.os.buttons[slot(button)] || !self.os.keys.is_empty();
        self.unmatched += usize::from(held && !carry);
    }

    fn click(&mut self, button: Button) {
        let (at, dpi) = (self.at, DPIS[0]);
        let ours = false;
        self.event(PointerEvent::Down {
            button,
            at,
            ours,
            dpi,
        });
        self.event(PointerEvent::Up { button, at });
        self.unmatched += usize::from(self.os.buttons[slot(button)]);
    }

    /// The hand lets go, then the host stops.
    fn finish(&mut self) {
        if self.left {
            self.flip(false, DPIS[0]);
        }
        self.e.release(&mut self.acts);
        self.send();
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(CASES))]

    /// Once the hand lets go and the host stops, no button and no modifier is down.
    #[test]
    fn nothing_stays_down_after_release(rclick in any::<bool>(), steps in mixed()) {
        let mut rig = Rig::play(&steps, rclick);
        rig.finish();
        prop_assert!(rig.os.idle());
    }

    /// Every press, dropped or not, is released at the OS by its own release; only a carry keeps the left down.
    #[test]
    fn every_dropped_press_gets_its_release(rclick in any::<bool>(), steps in mixed()) {
        let rig = Rig::play(&steps, rclick);
        prop_assert_eq!(rig.unmatched, 0);
    }

    /// With both modes off and no latches, every event passes and nothing is sent or noted.
    #[test]
    fn with_nothing_on_everything_passes(steps in run(physical())) {
        let mut rig = Rig::play(&steps, false);
        rig.finish();
        prop_assert_eq!((rig.dropped, rig.sent), (0, 0));
        prop_assert!(rig.notes.is_empty());
        prop_assert!(!rig.e.needs_hook());
    }
}
