//! Grab (spec §5.1): the next press outside picks up at once and the next click drops; a click on our windows, Esc
//! or a toggle cancels.

use super::{ESC, Engine, Mode, Note, State};
use kx_module_api::ports::{Button, Point, PointerAct, PointerEvent};

impl Engine {
    /// Picks up at `at`: the press goes through inside its modifiers, and its release will be dropped.
    pub(super) fn pick(&mut self, at: Point, acts: &mut Vec<PointerAct>) -> bool {
        let dropped = self.press_through(at, acts);
        self.state = State::Carrying { held: true };
        self.note(Note::Picked { at });
        dropped
    }

    /// Grab holds the left button; `held` while the grabbing press is still physically down.
    pub(super) fn carrying(
        &mut self,
        event: PointerEvent,
        held: bool,
        acts: &mut Vec<PointerAct>,
    ) -> bool {
        match event {
            PointerEvent::Up {
                button: Button::Left,
                ..
            } if held => {
                self.state = State::Carrying { held: false };
                true
            }
            PointerEvent::Down {
                button: Button::Left,
                at,
                ours,
                ..
            } => {
                if ours {
                    self.cancel(at, true, acts);
                } else {
                    self.note(Note::Dropped { at });
                    self.let_go(at, true, acts);
                }
                true
            }
            PointerEvent::Esc => {
                // The Esc itself reaches the app, so only the button is let go.
                self.note(Note::Cancelled);
                self.let_go(self.last, held, acts);
                false
            }
            _ => false,
        }
    }

    /// Cancels the carry: Esc to the app, then the button let go at `at`; `held` awaits the physical release.
    pub(super) fn cancel(&mut self, at: Point, held: bool, acts: &mut Vec<PointerAct>) {
        acts.extend([PointerAct::KeyDown(ESC), PointerAct::KeyUp(ESC)]);
        self.note(Note::Cancelled);
        self.let_go(at, held, acts);
    }

    /// Lets go of the carried button at `at` and of its modifiers; Grab ends.
    pub(super) fn let_go(&mut self, at: Point, held: bool, acts: &mut Vec<PointerAct>) {
        acts.push(PointerAct::Up(Button::Left, at));
        self.mods_up(acts);
        self.state = if held { State::AwaitUp } else { State::Idle };
        self.quit_grab();
    }

    /// Grab turns off, and Right-click comes back if it was on before.
    pub(super) fn quit_grab(&mut self) {
        if let Mode::Grab { back } = self.mode {
            self.mode = if back { Mode::RightClick } else { Mode::Off };
        }
    }
}
