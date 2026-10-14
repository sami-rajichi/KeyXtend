//! The hand's own input that reached the OS untouched: what it means for the queue and for the record of held input
//! (ledger ruling 22), whose presses the hand's release may have passed.

use kx_module_api::ports::{Point, PointerAct};

use super::Host;
use crate::hook::Os;
use crate::hook::accept::Kind;

impl<O: Os> Host<O> {
    /// The hand's event `kind` at `at` passed untouched. Ahead of acts `queued` before it, its Up must not leave a
    /// queued press held, and it lets go of a press of ours as `released` says.
    pub(super) fn passed(&mut self, kind: Kind, at: Point, queued: bool) {
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

    /// The user's key `act` passed, as every key does: a key-up lets go as `released` says, and a press cancels an Up
    /// owed for that key.
    pub(super) fn hand_key(&mut self, act: PointerAct) {
        match act {
            PointerAct::KeyDown(scan) => self.owed.forget(PointerAct::KeyUp(scan)),
            release => self.released(release),
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
