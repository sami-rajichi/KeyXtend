//! The clipboard panel on the face (spec §7, ADR-0034): the clip key and the history chord ask for the panel, its keys
//! go to it while it is open, and one plain paste goes out when the clipboard module asks. As in `bus.rs`, nothing is
//! sent while the face's lock is held.

use super::{Face, STATE};
use crate::run::ToPanel;
use kx_module_api::ports::{ClipKey, ClipOpen, FaceReply, Pasted};

impl Face {
    /// While the clipboard panel is open, key `id` goes to it if its key list takes the key: the panel hears it, and
    /// the state redraws, as a letter lets latches go. `None` when the key types as usual: the panel is shut, the open
    /// language list drives it, or Ctrl, Alt or Win acts on a letter.
    pub(super) fn to_clip(&self, id: &str) -> Option<FaceReply> {
        let (took, bus) = {
            let mut inner = self.lock();
            if inner.listing() && self.navs.contains_key(id) {
                return None;
            }
            (inner.clip_key(id, &self.clips), inner.bus())
        };
        match took {
            ToPanel::Pass => return None,
            ToPanel::Key(key) => {
                // The module holds `ReadClipboard`, which the key needs, or its start would have failed.
                let _ = bus.publish(&ClipKey(key));
            }
            ToPanel::Nothing => {}
        }
        Some(STATE)
    }

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
