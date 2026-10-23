//! The Shortcuts key on the face (spec §4.2): it shows or hides the layer, and showing it while the pad scrolls stops
//! the pad, whose Stop leaves the screen, and ends the pointer's hover on it. While it shows, the side keys it hides
//! are no keys. As in `bus.rs`, nothing is sent while the face's lock is held.

use super::inner::Inner;
use super::{Face, REDRAW};
use kx_module_api::ports::{FaceReply, ScrollStop};

impl Face {
    /// A click on the Shortcuts key: shows or hides the layer and redraws both; with nothing to show it does nothing.
    /// Right-click and Grab keep their modes, which the badge shows.
    pub(super) fn flip(&self) -> FaceReply {
        let (flipped, stop) = {
            let mut inner = self.lock();
            let flipped = inner.flip();
            let stop = flipped && inner.layered() && inner.scrolling();
            (flipped, stop.then(|| inner.bus()))
        };
        if let Some(bus) = stop {
            // The pad's events need no capability, so none is refused.
            let _ = bus.publish(&ScrollStop);
        }
        if !flipped {
            return FaceReply::default();
        }
        // The pad hid under a pointer resting on it: the scroll module hears it left.
        self.peek();
        REDRAW
    }

    /// True while the layer shows and hides side key `id`: every side key but the Shortcuts key.
    pub(super) fn hides(&self, inner: &Inner, id: &str) -> bool {
        inner.layered() && self.spec.layer_hides(id)
    }
}
