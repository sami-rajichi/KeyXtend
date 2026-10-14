//! The commands the module sends between events: the mode toggles, Right-click, the latches and letting go.

use super::{Engine, MAX_LATCHES, Mode, Note, State};
use kx_module_api::ports::{PointerAct, Scan, Toggle};

impl Engine {
    /// Flips Right-click or Grab; a held press goes through first, and a carry is cancelled.
    pub fn toggle(&mut self, which: Toggle, acts: &mut Vec<PointerAct>) {
        let before = self.view();
        match (which, self.mode) {
            (Toggle::Grab, Mode::Grab { .. }) => self.end_grab(acts),
            (Toggle::Grab, _) => self.arm(acts),
            (Toggle::RightClick, Mode::Grab { .. }) => {
                self.end_grab(acts);
                self.mode = Mode::RightClick;
            }
            (Toggle::RightClick, Mode::RightClick) => {
                self.unhold(acts);
                self.mode = Mode::Off;
            }
            (Toggle::RightClick, Mode::Off) => self.mode = Mode::RightClick,
        }
        self.note_state(before);
    }

    /// Sets Right-click as the user sees it; off also stops it coming back after Grab.
    pub fn set_rclick(&mut self, on: bool, acts: &mut Vec<PointerAct>) {
        if self.rclick() != on {
            self.toggle(Toggle::RightClick, acts);
        }
        if let Mode::Grab { back } = &mut self.mode {
            *back &= on;
        }
    }

    /// Takes the keyboard's latched modifiers for the next click, the first `MAX_LATCHES` in order; a click in
    /// progress keeps its own.
    pub fn set_latches(&mut self, mut latches: Vec<Scan>) {
        latches.truncate(MAX_LATCHES);
        self.latches = latches;
    }

    /// Lets go of everything held, so nothing stays down: a carry is cancelled with Esc first, as `cancel` does, so it
    /// never drops where the pointer is, and a held-back press is forgotten.
    pub fn release(&mut self, acts: &mut Vec<PointerAct>) {
        let before = self.view();
        match self.state {
            State::Pending { .. } => self.note(Note::HoldEnd),
            State::Carrying { .. } => self.cancel(self.last, false, acts),
            State::Idle | State::AwaitUp => {}
        }
        self.mods_up(acts);
        self.state = State::Idle;
        self.note_state(before);
    }

    /// Arms Grab, with Right-click off until it ends; a held press goes through first.
    fn arm(&mut self, acts: &mut Vec<PointerAct>) {
        self.unhold(acts);
        let back = self.rclick();
        self.mode = Mode::Grab { back };
    }

    /// Ends Grab, cancelling a carry.
    fn end_grab(&mut self, acts: &mut Vec<PointerAct>) {
        match self.state {
            State::Carrying { held } => self.cancel(self.last, held, acts),
            _ => self.quit_grab(),
        }
    }

    /// Lets a held press through, as a mode switch does.
    fn unhold(&mut self, acts: &mut Vec<PointerAct>) {
        if let State::Pending { p0, .. } = self.state {
            self.replay(p0, acts);
        }
    }
}
