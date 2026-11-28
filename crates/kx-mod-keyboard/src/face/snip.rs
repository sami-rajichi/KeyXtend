//! Snip on the face (ADR-0056, ADR-0057): the Snip key asks the snip module to start or end a capture and aims at its
//! windows, the key's light follows `SnipLit`, and the windows take the keys as a panel does while their `SnipLine`
//! shows and the user's press aimed at them. As in `bus.rs`, nothing is sent while the face's lock is held.

use super::{Face, STATE};
use kx_module_api::ports::{FaceReply, PanelLine, SnipLit, SnipStart};

impl Face {
    /// The Snip key: asks the snip module to start or end a capture, keeping every latch, and aims at Snip's windows;
    /// no panel closes. The state redraws, so a light the module tells at once shows at once.
    pub(super) fn snip_key(&self) -> FaceReply {
        let bus = {
            let mut inner = self.lock();
            inner.snip_key(self.outside());
            inner.bus()
        };
        // The module holds `Snip`, which this event needs, or its start would have failed.
        let _ = bus.publish(&SnipStart);
        STATE
    }

    /// A press on Snip's windows: their line takes the keys while it shows.
    pub(super) fn aim_snip(&self) -> FaceReply {
        self.lock().aim_snip(self.outside())
    }

    /// The snip module tells the key's light: lit while a capture runs.
    pub(super) fn heard_snip_lit(&self, lit: SnipLit) {
        self.lock().heard_snip_lit(lit);
    }

    /// The snip module tells which line its windows show, `None` while none takes keys; prediction and the mouse
    /// module hear what changed.
    pub(super) fn heard_snip_line(&self, line: Option<&PanelLine>) {
        self.lock().heard_snip(line);
        self.pour();
        self.tell(false);
    }
}
