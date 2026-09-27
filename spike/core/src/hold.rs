//! The hold engine of spec §5.1: a still left hold becomes a right-click or a grab.
//!
//! Pure logic: the host passes each hook event and the time in µs, and injects what comes back.

/// The Esc key, sent to cancel a drag.
pub const ESC: u16 = 0x1B;

/// A screen point in physical pixels.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Pt {
    /// Pixels from the left of the virtual desktop.
    pub x: i32,
    /// Pixels from the top of the virtual desktop.
    pub y: i32,
}

/// A mouse button the engine knows.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Button {
    /// The main button.
    Left,
    /// The context-menu button.
    Right,
    /// The wheel button.
    Middle,
}

/// What a still left hold turns into.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    /// Every event passes untouched.
    Off,
    /// A still hold becomes a right-click; short clicks and drags stay normal.
    RightClick,
    /// A still hold latches the left button until the next click drops it.
    Grab,
}

/// One input event, as the hook sees it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Event {
    /// A button press; `ours` when it lands on one of our own windows.
    Down(Button, Pt, bool),
    /// A button release.
    Up(Button, Pt),
    /// The pointer moved.
    Move(Pt),
    /// The Esc key went down.
    Esc,
    /// The host's timer fired.
    Tick,
}

/// Input the host injects after the hook returns, in order.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Act {
    /// Moves the pointer there.
    Move(Pt),
    /// Presses a button there.
    Down(Button, Pt),
    /// Releases a button there.
    Up(Button, Pt),
    /// Presses a virtual key, such as a latched modifier.
    KeyDown(u16),
    /// Releases a virtual key.
    KeyUp(u16),
}

/// The engine's answer to one event: drop it or let it through, then inject `acts`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Step {
    /// Drop the event instead of letting it through.
    pub swallow: bool,
    /// Input to inject afterwards, in order.
    pub acts: Vec<Act>,
}

/// Where the engine is between events.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum State {
    Idle,
    /// A left press is held back since `t0` at `p0`.
    Pending {
        t0: i64,
        p0: Pt,
    },
    /// The hold fired; the physical release is still to come and is dropped.
    AwaitUp,
    /// Grab holds the left button down; `held` while the grabbing press is still down.
    Carrying {
        held: bool,
    },
}

/// The §5.1 state machine with its hold time and still radius.
#[derive(Debug, Clone)]
pub struct Engine {
    hold_us: i64,
    still_px: i32,
    mode: Mode,
    /// The mode to come back to when a grab ends.
    before_grab: Mode,
    state: State,
    /// The last pointer position seen.
    last: Pt,
    /// Modifiers held down until the next left click ends.
    latched: Vec<u16>,
    /// The last left press landed on our own window, so its release keeps latched modifiers.
    down_ours: bool,
}

/// Lets the event through, then injects `acts`.
fn pass(acts: Vec<Act>) -> Step {
    Step {
        swallow: false,
        acts,
    }
}

/// Drops the event, then injects `acts`.
fn eat(acts: Vec<Act>) -> Step {
    Step {
        swallow: true,
        acts,
    }
}

impl Engine {
    /// An engine in `Off` mode; a hold fires after `hold_us`, and moves up to `still_px` count as still.
    pub fn new(hold_us: i64, still_px: i32) -> Self {
        Self {
            hold_us,
            still_px,
            mode: Mode::Off,
            before_grab: Mode::Off,
            state: State::Idle,
            last: Pt::default(),
            latched: Vec::new(),
            down_ours: false,
        }
    }

    /// The current mode.
    pub fn mode(&self) -> Mode {
        self.mode
    }

    /// When the host should send `Tick`, if a press is held back.
    pub fn deadline(&self) -> Option<i64> {
        match self.state {
            State::Pending { t0, .. } => Some(t0.saturating_add(self.hold_us)),
            _ => None,
        }
    }

    /// Switches mode; a press held back is let through first, and a carried button is released.
    pub fn set_mode(&mut self, mode: Mode) -> Vec<Act> {
        if mode == self.mode {
            return Vec::new();
        }
        if mode == Mode::Grab {
            self.before_grab = self.mode;
        }
        self.mode = mode;
        match self.state {
            State::Pending { p0, .. } => {
                self.state = State::Idle;
                vec![Act::Down(Button::Left, p0)]
            }
            State::Carrying { held } => self.drop_carry(held, self.last),
            _ => Vec::new(),
        }
    }

    /// Turns the engine off and lets go of everything it holds: a held press, a carry, latched keys.
    pub fn reset(&mut self) -> Vec<Act> {
        let mut acts = self.set_mode(Mode::Off);
        acts.extend(self.release_latched());
        acts
    }

    /// Holds modifier `vk` down until the next left click ends.
    pub fn latch(&mut self, vk: u16) -> Vec<Act> {
        if self.latched.contains(&vk) {
            return Vec::new();
        }
        self.latched.push(vk);
        vec![Act::KeyDown(vk)]
    }

    /// Handles one event at time `now` (µs); a hold whose time has passed fires first, even if its tick is late.
    pub fn on(&mut self, event: Event, now: i64) -> Step {
        if let Event::Down(_, at, _) | Event::Up(_, at) | Event::Move(at) = event {
            self.last = at;
        }
        let mut acts = Vec::new();
        if let State::Pending { p0, .. } = self.state
            && self.deadline().is_some_and(|due| now >= due)
        {
            acts = self.fire(p0);
        }
        let mut step = match self.state {
            State::Idle => self.idle(event, now),
            State::Pending { p0, .. } => self.pending(event, p0),
            State::AwaitUp => self.await_up(event),
            State::Carrying { held } => self.carrying(event, held),
        };
        acts.append(&mut step.acts);
        step.acts = acts;
        step
    }

    fn idle(&mut self, event: Event, now: i64) -> Step {
        match event {
            Event::Down(Button::Left, p0, ours) => {
                self.down_ours = ours;
                if ours || self.mode == Mode::Off {
                    return pass(Vec::new());
                }
                self.state = State::Pending { t0: now, p0 };
                eat(Vec::new())
            }
            Event::Up(Button::Left, _) if self.down_ours => {
                self.down_ours = false;
                pass(Vec::new())
            }
            Event::Up(Button::Left, _) => pass(self.release_latched()),
            _ => pass(Vec::new()),
        }
    }

    /// A left press is held back at `p0`, and its hold time has not passed yet.
    fn pending(&mut self, event: Event, p0: Pt) -> Step {
        let left = Button::Left;
        match event {
            Event::Up(Button::Left, _) => {
                self.state = State::Idle;
                let mut acts = vec![Act::Down(left, p0), Act::Up(left, p0)];
                acts.extend(self.release_latched());
                eat(acts)
            }
            Event::Move(to) if self.moved(p0, to) => {
                self.state = State::Idle;
                eat(vec![Act::Down(left, p0), Act::Move(to)])
            }
            Event::Down(button, at, _) => {
                self.state = State::Idle;
                eat(vec![Act::Down(left, p0), Act::Down(button, at)])
            }
            _ => pass(Vec::new()),
        }
    }

    /// The hold fired at `p0`: Grab latches the left button, otherwise a right-click.
    fn fire(&mut self, p0: Pt) -> Vec<Act> {
        if self.mode == Mode::Grab {
            self.state = State::Carrying { held: true };
            return vec![Act::Down(Button::Left, p0)];
        }
        self.state = State::AwaitUp;
        let mut acts = vec![Act::Down(Button::Right, p0), Act::Up(Button::Right, p0)];
        acts.extend(self.release_latched());
        acts
    }

    fn await_up(&mut self, event: Event) -> Step {
        match event {
            Event::Up(Button::Left, _) => {
                self.state = State::Idle;
                eat(Vec::new())
            }
            _ => pass(Vec::new()),
        }
    }

    /// Grab holds the button: the next click drops it there, Esc drops it where the pointer is.
    ///
    /// A click on our own window cancels instead: Esc stops the drag before the button is let go.
    fn carrying(&mut self, event: Event, held: bool) -> Step {
        match event {
            Event::Up(Button::Left, _) if held => {
                self.state = State::Carrying { held: false };
                eat(Vec::new())
            }
            Event::Down(Button::Left, at, ours) => {
                self.mode = self.before_grab;
                let mut acts = match ours {
                    true => vec![Act::KeyDown(ESC), Act::KeyUp(ESC)],
                    false => Vec::new(),
                };
                acts.extend(self.drop_carry(true, at));
                eat(acts)
            }
            Event::Esc => {
                self.mode = self.before_grab;
                pass(self.drop_carry(held, self.last))
            }
            _ => pass(Vec::new()),
        }
    }

    /// Releases the carried button at `at`; waits for the physical release while `held`.
    fn drop_carry(&mut self, held: bool, at: Pt) -> Vec<Act> {
        self.state = if held { State::AwaitUp } else { State::Idle };
        let mut acts = vec![Act::Up(Button::Left, at)];
        acts.extend(self.release_latched());
        acts
    }

    fn release_latched(&mut self) -> Vec<Act> {
        self.latched.drain(..).map(Act::KeyUp).collect()
    }

    /// True when `to` is farther than the still radius from `p0`.
    fn moved(&self, p0: Pt, to: Pt) -> bool {
        let dx = i64::from(to.x) - i64::from(p0.x);
        let dy = i64::from(to.y) - i64::from(p0.y);
        let r = i64::from(self.still_px);
        dx.saturating_mul(dx).saturating_add(dy.saturating_mul(dy)) > r * r
    }
}

#[cfg(test)]
mod tests;
