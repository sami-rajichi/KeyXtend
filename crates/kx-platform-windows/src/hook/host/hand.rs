//! The hand's own input: its event for the engine, and what it means for the queue and for the record of held input,
//! which must say what stays down now. The hand's release may pass our press, and the engine may drop the release of
//! the hand's own press.

use kx_module_api::ports::{Point, PointerAct, PointerEvent};

use super::Host;
use crate::hook::Os;
use crate::hook::accept::{Kind, act};

impl<O: Os> Host<O> {
    /// The event for `kind` at `at`; a press says whether it landed on our window, and its screen's DPI.
    pub(super) fn event(&self, kind: Kind, at: Point) -> PointerEvent {
        match kind {
            Kind::Down(button) => PointerEvent::Down {
                button,
                at,
                ours: self.os.ours(at),
                dpi: self.os.dpi(at),
            },
            Kind::Up(button) => PointerEvent::Up { button, at },
            Kind::Move => PointerEvent::Move { at },
        }
    }

    /// The hand's event `kind` at `at`, after acts `queued` before it, passed untouched or was `dropped`. A release
    /// dropped while the hand's own press holds its button leaves that button down, so the host lets it go itself.
    pub(super) fn heard(&mut self, kind: Kind, at: Point, queued: bool, dropped: bool) {
        if !dropped {
            self.hand.sent(&[act(kind, at)]);
            return self.passed(kind, at, queued);
        }
        let release = act(kind, at);
        if let Some(press) = self.hand.holds(release) {
            self.held.sent(&[press]);
            self.hand.forget(release);
        }
    }

    /// The user's key `act` passed, as every key does: a key-up lets go as `released` says, and a press cancels an Up
    /// owed for that key.
    pub(super) fn hand_key(&mut self, act: PointerAct) {
        match act {
            PointerAct::KeyDown(scan) => self.owed.forget(PointerAct::KeyUp(scan)),
            release => self.released(release),
        }
    }

    /// The hand's event `kind` at `at` passed untouched. Ahead of acts `queued` before it, its Up must not leave a
    /// queued press held, and it lets go of a press of ours as `released` says.
    fn passed(&mut self, kind: Kind, at: Point, queued: bool) {
        if queued {
            self.queue.passed(kind, at);
        }
        match kind {
            Kind::Up(button) => self.released(PointerAct::Up(button, at)),
            // The hand holds it again, so an owed Up would let go of the hand's own press.
            Kind::Down(button) => self.owed.forget(PointerAct::Up(button, at)),
            Kind::Move => {}
        }
    }

    /// The user's `release` passed. A press of ours it lets go is forgotten once proven ahead of it; while that press
    /// may still be out, the order is not proven, so it stays held and the next batch owes its Up.
    fn released(&mut self, release: PointerAct) {
        let Some(press) = self.held.holds(release) else {
            return;
        };
        if self.echo.out(press) {
            self.owed.sent(&[press]);
            self.echo.owed(press);
            self.wake();
        } else {
            self.held.forget(release);
        }
    }
}
