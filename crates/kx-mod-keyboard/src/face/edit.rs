//! The selection chips' edit keys on the face (ADR-0041 decision 4): the keyboard stays the only typist, so the
//! selection module asks it for Copy, Cut or Paste. As in `bus.rs`, nothing is sent while the face's lock is held.

use super::Face;
use kx_module_api::ports::EditAsk;

impl Face {
    /// The selection module asks for `ask`: its edit goes to the app in front with every latch left as it was. The
    /// pieces' sender then tells, in order, the pieces typed before it, the answer and the edit's break, from whichever
    /// thread holds it. After the stop, or while a fill waits, nothing is sent and the answer is no.
    pub(super) fn edit(&self, ask: EditAsk) {
        let sent = self.lock().edit(ask);
        self.pour();
        if sent {
            self.tell(false);
        }
    }
}
