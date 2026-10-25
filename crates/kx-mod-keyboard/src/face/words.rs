//! Word prediction on the face (ADR-0033): the typed pieces leave in order, one thread at a time and outside the lock,
//! so the bus hears each once; a picked word from the bus types on the sender's thread and blocks on nothing it holds.

use super::Face;
use kx_module_api::ports::{TypeWord, Typed};

impl Face {
    /// Tells the bus every piece typed and not yet told, through the outbox, so none goes out twice or out of order.
    pub(super) fn pour(&self) {
        self.pieces.post(|| self.send_pieces());
    }

    /// Sends the waiting pieces, oldest first, until none waits; only the sending thread calls it.
    fn send_pieces(&self) {
        loop {
            let (pieces, bus) = {
                let mut inner = self.lock();
                (inner.take_pieces(), inner.bus())
            };
            if pieces.is_empty() {
                return;
            }
            for piece in pieces {
                // The module holds `ReadFocusedText`, which this event needs, or its start would have failed.
                let _ = bus.publish(&Typed { piece });
            }
        }
    }

    /// A picked word from the prediction module, whose round that module checked: it types as keys would, then its
    /// pieces and the typing mode go out.
    pub(super) fn type_word(&self, word: &TypeWord) {
        self.lock().type_word(word.erase, &word.text);
        self.pour();
        self.tell(false);
    }

    /// The module started, or stopped when `on` is false; a picked word types only while it runs.
    pub(crate) fn set_up(&self, on: bool) {
        self.lock().set_up(on);
    }
}
