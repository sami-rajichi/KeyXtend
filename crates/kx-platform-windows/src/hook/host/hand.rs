//! The hand's own input that reached the OS untouched: what it means for the queue and for the record of held input
//! (ledger ruling 22).

use kx_module_api::ports::Point;

use super::Host;
use crate::hook::Os;
use crate::hook::accept::Kind;

impl<O: Os> Host<O> {
    /// The hand's event `kind` at `at` passed untouched. Ahead of acts `queued` before it, its Up must not leave a
    /// queued press held; with nothing of ours in flight, it lets go of that button in the record too.
    pub(super) fn passed(&mut self, kind: Kind, at: Point, queued: bool) {
        if queued {
            self.queue.passed(kind, at);
        }
        if let Kind::Up(button) = kind
            && self.idle()
        {
            self.held.forget_button(button);
        }
    }

    /// True when nothing of ours is in flight: nothing is queued and no echo is awaited.
    pub(super) fn idle(&self) -> bool {
        self.queue.is_empty() && self.echo.is_clear()
    }
}
