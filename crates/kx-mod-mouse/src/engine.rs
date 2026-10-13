//! The hold engine of spec §5.1: a still hold right-clicks, Grab picks up at once, and latched modifiers go around
//! the next click. Pure logic: the host passes each hook event with the time, then sends the input it gets back.

mod carry;
mod modes;
mod mods;
mod note;
mod press;

pub use note::Note;

use crate::MouseSettings;
use kx_module_api::ports::{Button, GrabState, Point, PointerAct, PointerEvent, Scan};
use kx_module_api::{Mono, US_PER_MS};

/// The Esc key's scan code (PC set 1), sent to cancel a carry.
pub const ESC: Scan = Scan::new(0x01);

/// The most latched modifiers a click carries: left and right Shift, Ctrl, Alt and Win.
pub const MAX_LATCHES: usize = 8;

/// The most input one call pushes: a late right-click inside every modifier (2 per latch, plus 2), then the event
/// it revealed.
pub const MAX_ACTS: usize = 2 * MAX_LATCHES + 3;

/// How many notes the buffer holds; the host drains it after every callback, and a full buffer drops new notes.
const NOTE_ROOM: usize = 32;

/// The engine's settings, which the module builds from `MouseSettings`.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct EngineCfg {
    /// A still press right-clicks after this, in µs.
    pub hold_us: u64,
    /// The still radius in logical pixels; each press scales it by its screen's DPI.
    pub still_px: u32,
    /// Right-click mode starts on.
    pub rclick: bool,
}

impl From<&MouseSettings> for EngineCfg {
    fn from(s: &MouseSettings) -> Self {
        Self {
            hold_us: u64::from(s.hold_ms).saturating_mul(u64::from(US_PER_MS)),
            still_px: s.still_px,
            rclick: s.rclick,
        }
    }
}

/// The mode the user chose; Grab turns Right-click off while it is on.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum Mode {
    /// Neither mode.
    Off,
    /// A still hold right-clicks.
    RightClick,
    /// The next press picks up; `back` turns Right-click on again when Grab ends.
    Grab { back: bool },
}

/// Where the engine is between events.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum State {
    /// Nothing is held back.
    Idle,
    /// A left press is held back since `t0` at `p0`; moves within `r` physical pixels are still.
    Pending { t0: Mono, p0: Point, r: i32 },
    /// The physical release still to come is dropped, after a fire, a drop or a cancel.
    AwaitUp,
    /// Grab holds the left button down; `held` while the grabbing press is still physically down.
    Carrying { held: bool },
}

/// The §5.1 state machine: Right-click, Grab and modifier + click.
pub struct Engine {
    hold_us: u64,
    still_px: u32,
    mode: Mode,
    state: State,
    /// The last pointer point seen.
    last: Point,
    /// The keyboard's latched modifiers.
    latches: Vec<Scan>,
    /// The modifiers of the click in progress, taken from `latches` at its press.
    mods: Vec<Scan>,
    /// `mods` are pressed at the OS.
    mods_down: bool,
    /// Notes for the module, drained by `take_notes`.
    notes: Vec<Note>,
}

impl Engine {
    /// An engine with nothing held; Right-click starts as `cfg` says.
    #[must_use]
    pub fn new(cfg: EngineCfg) -> Self {
        Self {
            hold_us: cfg.hold_us,
            still_px: cfg.still_px,
            mode: if cfg.rclick {
                Mode::RightClick
            } else {
                Mode::Off
            },
            state: State::Idle,
            last: Point::default(),
            latches: Vec::new(),
            mods: Vec::with_capacity(MAX_LATCHES),
            mods_down: false,
            notes: Vec::with_capacity(NOTE_ROOM),
        }
    }

    /// Right-click is on; it is off while Grab is on.
    #[must_use]
    pub fn rclick(&self) -> bool {
        self.mode == Mode::RightClick
    }

    /// Where Grab stands.
    #[must_use]
    pub fn grab(&self) -> GrabState {
        match (self.mode, self.state) {
            (Mode::Grab { .. }, State::Carrying { .. }) => GrabState::Carrying,
            (Mode::Grab { .. }, _) => GrabState::Armed,
            _ => GrabState::Off,
        }
    }

    /// The hook is needed: a mode is on, latches are held, a press is held back or awaited, a carry runs, or
    /// modifiers are still pressed.
    #[must_use]
    pub fn needs_hook(&self) -> bool {
        self.mode != Mode::Off
            || !self.latches.is_empty()
            || self.mods_down
            || self.state != State::Idle
    }

    /// When a held-back press fires, if one is held.
    #[must_use]
    pub fn deadline(&self) -> Option<Mono> {
        match self.state {
            State::Pending { t0, .. } => Some(self.due(t0)),
            _ => None,
        }
    }

    /// Moves the notes since the last call into `out`.
    pub fn take_notes(&mut self, out: &mut Vec<Note>) {
        out.append(&mut self.notes);
    }

    /// Decides one hook event at `now`, pushes the input to send into `acts`, and returns true to drop the event.
    /// A hold whose time has passed fires first, even if its tick is late.
    pub fn on(&mut self, event: PointerEvent, now: Mono, acts: &mut Vec<PointerAct>) -> bool {
        let before = self.view();
        let fired = self.fire_due(now, acts);
        self.track(event);
        let mut dropped = match self.state {
            State::Idle => self.idle(event, now, acts),
            State::Pending { p0, r, .. } => self.pending(event, p0, r, now, acts),
            State::AwaitUp => self.await_up(event, now, acts),
            State::Carrying { held } => self.carrying(event, held, acts),
        };
        if fired {
            dropped = behind(event, dropped, acts);
        }
        self.note_state(before);
        dropped
    }

    /// Fires a hold whose time has come at `now`.
    pub fn tick(&mut self, now: Mono, acts: &mut Vec<PointerAct>) {
        self.fire_due(now, acts);
    }

    /// When a hold that began at `t0` fires.
    fn due(&self, t0: Mono) -> Mono {
        Mono::from_us(t0.as_us().saturating_add(self.hold_us))
    }

    /// After a fire, a drop or a cancel: the physical release is dropped. A left press shows the hook missed that
    /// release, so the press starts afresh.
    fn await_up(&mut self, event: PointerEvent, now: Mono, acts: &mut Vec<PointerAct>) -> bool {
        match event {
            PointerEvent::Up {
                button: Button::Left,
                ..
            } => {
                self.state = State::Idle;
                true
            }
            PointerEvent::Down {
                button: Button::Left,
                ..
            } => {
                self.state = State::Idle;
                self.idle(event, now, acts)
            }
            _ => false,
        }
    }

    /// Keeps the last point, and notes a move while a mode is on.
    fn track(&mut self, event: PointerEvent) {
        match event {
            PointerEvent::Down { at, .. } | PointerEvent::Up { at, .. } => self.last = at,
            PointerEvent::Move { at } => {
                self.last = at;
                if self.mode != Mode::Off {
                    self.note(Note::Moved { at });
                }
            }
            PointerEvent::Esc => {}
        }
    }

    /// Right-click and Grab as the UI shows them.
    fn view(&self) -> (bool, GrabState) {
        (self.rclick(), self.grab())
    }

    /// Notes the new state when it differs from `before`.
    fn note_state(&mut self, before: (bool, GrabState)) {
        let (rclick, grab) = self.view();
        if (rclick, grab) != before {
            self.note(Note::State { rclick, grab });
        }
    }

    /// Keeps `note` within the reserved room, so the hot path never allocates; a move replaces a move just before.
    fn note(&mut self, note: Note) {
        if let (Note::Moved { .. }, Some(last @ Note::Moved { .. })) = (note, self.notes.last_mut())
        {
            *last = note;
        } else if self.notes.len() < self.notes.capacity() {
            self.notes.push(note);
        }
    }
}

/// Our input so far must reach the app first, so a passed `event` is sent again behind it; Esc still passes, as it
/// is never dropped. Returns whether the event is dropped.
fn behind(event: PointerEvent, dropped: bool, acts: &mut Vec<PointerAct>) -> bool {
    let again = match event {
        _ if dropped => return true,
        PointerEvent::Down { button, at, .. } => PointerAct::Down(button, at),
        PointerEvent::Up { button, at } => PointerAct::Up(button, at),
        PointerEvent::Move { at } => PointerAct::Move(at),
        PointerEvent::Esc => return false,
    };
    acts.push(again);
    true
}

#[cfg(test)]
mod props;
#[cfg(test)]
mod tests;
