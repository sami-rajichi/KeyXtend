//! The mouse module's presses: the actor wants them while Undo insert can act, so a click elsewhere ends it.

use kx_module_api::ports::{PressAsker, PressesWanted};

use super::Actor;

impl Actor {
    /// Tells the mouse module whether presses are wanted, when that changed.
    pub(super) fn want_presses(&mut self, on: bool) {
        if std::mem::replace(&mut self.presses, on) != on {
            self.publish(&PressesWanted {
                asker: PressAsker::Voice,
                on,
            });
        }
    }

    /// A late mouse module hears the wish again, while there is one.
    pub(super) fn retell_presses(&self) {
        if self.presses {
            self.publish(&PressesWanted {
                asker: PressAsker::Voice,
                on: true,
            });
        }
    }
}
