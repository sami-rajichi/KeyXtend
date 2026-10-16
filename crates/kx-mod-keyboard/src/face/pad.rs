//! The scroll pad on the bus (spec §5.2, ADR-0029): its arrows and Stop ask the scroll module to scroll or stop, and
//! the pointer on it asks for the outline. As in `bus.rs`, nothing is sent while the face's lock is held.

use super::Face;
use kx_module_api::ports::{Dir, FaceReply, ScrollChanged, ScrollPeek, ScrollStop, ScrollTap};

impl Face {
    /// A click on an arrow, or on Stop when `dir` is none: asks the scroll module while it runs, else does nothing.
    /// Nothing redraws now; the module's answer does.
    pub(super) fn pad(&self, dir: Option<Dir>) -> FaceReply {
        let bus = {
            let inner = self.lock();
            inner.scrolls().then(|| inner.bus())
        };
        let Some(bus) = bus else {
            return FaceReply::default();
        };
        // The pad's events need no capability, so none is refused.
        let _ = match dir {
            Some(dir) => bus.publish(&ScrollTap { dir }),
            None => bus.publish(&ScrollStop),
        };
        FaceReply::default()
    }

    /// The pointer went onto the pad or off it: the running scroll module hears each change once.
    pub(super) fn hover(&self, on: bool) -> FaceReply {
        let bus = {
            let mut inner = self.lock();
            inner.hovered(on).then(|| inner.bus())
        };
        if let Some(bus) = bus {
            let _ = bus.publish(&ScrollPeek { on });
        }
        FaceReply::default()
    }

    /// The scroll module's new state; a module that comes live while the pointer rests on the pad hears so.
    pub(super) fn scrolled(&self, now: ScrollChanged) {
        let bus = {
            let mut inner = self.lock();
            inner.heard_scroll(now).then(|| inner.bus())
        };
        if let Some(bus) = bus {
            let _ = bus.publish(&ScrollPeek { on: true });
        }
    }
}
