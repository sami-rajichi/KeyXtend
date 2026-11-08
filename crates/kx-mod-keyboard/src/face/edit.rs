//! The selection chips' edit keys on the face (ADR-0041 decision 4): the keyboard stays the only typist, so the
//! selection module asks it for Copy, Cut or Paste. As in `bus.rs`, nothing is sent while the face's lock is held.

use super::Face;
use kx_module_api::ports::{EditAsk, EditDone};

impl Face {
    /// The selection module asks for `ask`: its edit goes to the app in front with every latch left as it was, the
    /// module hears whether all of it went, then the bus hears the break. After the stop, or while a fill waits, nothing
    /// is sent and the answer is no.
    pub(super) fn edit(&self, ask: EditAsk) {
        let (sent, bus) = {
            let mut inner = self.lock();
            (inner.edit(ask.edit, ask.terminal), inner.bus())
        };
        // The module holds `Selection`, which the answer needs, or its start would have failed.
        let _ = bus.publish(&EditDone {
            round: ask.round,
            ok: sent == Some(true),
        });
        if sent.is_some() {
            self.pour();
            self.tell(false);
        }
    }
}
