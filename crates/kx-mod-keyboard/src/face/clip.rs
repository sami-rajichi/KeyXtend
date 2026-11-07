//! The clipboard on the face (spec §7, ADR-0034): the clip key and the history chord ask for its panel, and one plain
//! paste goes out when the clipboard module asks. As in `bus.rs`, nothing is sent while the face's lock is held.

use super::Face;
use kx_module_api::ports::{ClipOpen, Pasted};

impl Face {
    /// The clip key or the history chord: asks the clipboard module to open or close its panel.
    pub(super) fn clip_open(&self) {
        let bus = self.lock().bus();
        // `ClipOpen` needs no capability, so none is refused.
        let _ = bus.publish(&ClipOpen);
    }

    /// The clipboard module set the clipboard for paste `round`: every latch lets go, one plain paste goes to the app
    /// in front, and the module hears whether all of it went; then the bus hears the latches and the piece. After the
    /// stop nothing is sent and the answer is no.
    pub(super) fn paste(&self, round: u32) {
        let (sent, bus) = {
            let mut inner = self.lock();
            (inner.paste(), inner.bus())
        };
        // The module holds `WriteClipboard`, which the answer needs, or its start would have failed.
        let _ = bus.publish(&Pasted {
            round,
            ok: sent == Some(true),
        });
        if sent.is_some() {
            self.pour();
            self.tell(false);
        }
    }
}
