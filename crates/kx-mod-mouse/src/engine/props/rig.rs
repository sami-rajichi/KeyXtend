//! The model the properties run: the hand on the mouse, the engine, and what the OS sees.

use super::*;
use std::collections::BTreeSet;

/// How much the OS model forgives; keys are always strict.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub(super) enum Model {
    /// Every button press and release must match.
    #[default]
    Strict,
    /// A stray release is ignored, as after the host stopped mid-click.
    Stops,
    /// A second press is ignored too, as after a release the hook missed; Windows takes both.
    Misses,
}

/// What the OS sees as down; it refuses what its model does not forgive.
#[derive(Default)]
pub(super) struct Os {
    pub(super) buttons: [bool; 3],
    pub(super) keys: BTreeSet<Scan>,
    model: Model,
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
        let forgiven = match self.model {
            Model::Strict => false,
            Model::Stops => !down,
            Model::Misses => true,
        };
        if forgiven && *held == down {
            return;
        }
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

    pub(super) fn idle(&self) -> bool {
        self.buttons == [false; 3] && self.keys.is_empty()
    }
}

/// The engine, the hand on the mouse, what the OS sees, and counts for the properties.
pub(super) struct Rig {
    pub(super) e: Engine,
    now: u64,
    at: Point,
    left: bool,
    pub(super) os: Os,
    acts: Vec<PointerAct>,
    pub(super) notes: Vec<Note>,
    /// Events the engine dropped.
    pub(super) dropped: usize,
    /// Input the engine sent, in order.
    pub(super) sent: Vec<PointerAct>,
    /// Releases after which the OS still held that button or a modifier, outside a carry.
    pub(super) unmatched: usize,
    /// Steps after which the engine needed no hook, yet the OS held a key or a button the hand does not.
    pub(super) leaks: usize,
    /// Scrolling is on.
    scrolling: bool,
    /// Presses are wanted.
    wanted: bool,
    /// Events whose press notes differ from the one expected for a press while scrolling or wanted, else none.
    pub(super) misnoted: usize,
    /// Steps after which presses were wanted, yet the engine needed no hook.
    pub(super) unhooked: usize,
}

impl Rig {
    /// Plays `steps` on an engine with Right-click as given, against an OS that forgives as `model` says.
    pub(super) fn play(steps: &[(u64, Step)], rclick: bool, model: Model) -> Self {
        let cfg = EngineCfg {
            hold_us: HOLD,
            still_px: STILL,
            rclick,
        };
        let mut rig = Self {
            e: Engine::new(cfg),
            now: 0,
            at: START,
            left: false,
            os: Os {
                model,
                ..Os::default()
            },
            acts: Vec::new(),
            notes: Vec::new(),
            dropped: 0,
            sent: Vec::new(),
            unmatched: 0,
            leaks: 0,
            scrolling: false,
            wanted: false,
            misnoted: 0,
            unhooked: 0,
        };
        rig.more(steps);
        rig
    }

    pub(super) fn more(&mut self, steps: &[(u64, Step)]) {
        for (gap, step) in steps {
            self.now += gap;
            self.step(step);
        }
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
            Step::Scroll(on) => {
                self.scrolling = on;
                self.e.set_scrolling(on);
            }
            Step::Wanted(on) => {
                self.wanted = on;
                self.e.set_wanted(on);
            }
            Step::Release => self.e.release(&mut self.acts),
            Step::Miss => self.miss(),
        }
        self.send();
        let stray = (self.os.buttons[0] && !self.left) || self.os.buttons[1] || self.os.buttons[2];
        let held = stray || !self.os.keys.is_empty();
        self.leaks += usize::from(held && !self.e.needs_hook());
        self.unhooked += usize::from(self.wanted && !self.e.needs_hook());
    }

    /// Runs one event; the OS sees it unless the engine drops it, then the engine's input. Its press notes are checked.
    fn event(&mut self, event: PointerEvent) {
        let press = matches!(event, PointerEvent::Down { .. });
        let (want, heard) = (press && (self.scrolling || self.wanted), self.notes.len());
        if self.e.on(event, Mono::from_us(self.now), &mut self.acts) {
            self.dropped += 1;
        } else {
            self.os.see(event);
        }
        self.send();
        self.misnoted += usize::from(presses(&self.notes[heard..]) != usize::from(want));
    }

    /// The OS takes one call's input, which never passes `MAX_ACTS`, and the notes are drained as the host does.
    fn send(&mut self) {
        assert!(self.acts.len() <= MAX_ACTS, "{} acts", self.acts.len());
        self.sent.extend_from_slice(&self.acts);
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

    /// The hand lets go of the left button unseen by the engine; the OS still gets the release.
    fn miss(&mut self) {
        if self.left {
            self.left = false;
            self.os.button(Button::Left, false);
        }
    }

    /// The hand lets go of the left button.
    pub(super) fn let_go(&mut self) {
        if self.left {
            self.flip(false, DPIS[0]);
        }
    }

    /// Both modes go off, the latches clear, scrolling ends and presses are no longer wanted; then the counts start
    /// again.
    pub(super) fn calm(&mut self) {
        self.let_go();
        self.step(&Step::Scroll(false));
        self.step(&Step::Wanted(false));
        self.e.set_rclick(false, &mut self.acts);
        if self.e.grab() != GrabState::Off {
            self.e.toggle(Toggle::Grab, &mut self.acts);
        }
        self.e.set_latches(Vec::new());
        self.send();
        self.dropped = 0;
        self.sent.clear();
        self.notes.clear();
    }

    /// The hand lets go and clicks once outside, then the host stops.
    pub(super) fn recover(&mut self) {
        self.let_go();
        self.flip(false, DPIS[0]);
        self.flip(false, DPIS[0]);
        self.finish();
    }

    /// The hand lets go, then the host stops.
    pub(super) fn finish(&mut self) {
        self.let_go();
        self.e.release(&mut self.acts);
        self.send();
    }
}
