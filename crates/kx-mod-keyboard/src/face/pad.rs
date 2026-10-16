//! The scroll pad on the bus (spec §5.2, ADR-0029): its arrows and Stop ask the scroll module to scroll or stop, and
//! the pointer on it asks for the outline. As in `bus.rs`, nothing is sent while the face's lock is held.

use super::Face;
use super::outbox::Outbox;
use kx_module_api::ports::{Dir, FaceReply, ScrollChanged, ScrollPeek, ScrollStop, ScrollTap};
use std::sync::{Mutex, PoisonError};

/// What the scroll module last heard of the pointer on the pad, so it hears each change once and the newest last.
#[derive(Default)]
pub(super) struct Peeks {
    /// Requests to check the pointer, one sending thread at a time.
    outbox: Outbox,
    /// The pointer is on the pad as the running module last heard; only the sending thread touches it.
    heard: Mutex<bool>,
}

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

    /// The pointer went onto the pad or off it.
    pub(super) fn hover(&self, on: bool) -> FaceReply {
        self.lock().hovered(on);
        self.peek();
        FaceReply::default()
    }

    /// The scroll module's new state; one that comes live while the pointer rests on the pad hears so.
    pub(super) fn scrolled(&self, now: ScrollChanged) {
        self.lock().heard_scroll(now);
        self.peek();
    }

    /// Tells the scroll module where the pointer is, through the outbox, so the last peek it hears is the newest.
    fn peek(&self) {
        self.peeks.outbox.post(|| self.send_peek());
    }

    /// Sends the pointer's place when it differs from what the running module last heard; a module that stops forgets
    /// it, and nothing goes out while it is off. Only the sending thread calls it.
    fn send_peek(&self) {
        let (live, on, bus) = {
            let inner = self.lock();
            (inner.scrolls(), inner.hovering(), inner.bus())
        };
        let mut heard = self
            .peeks
            .heard
            .lock()
            .unwrap_or_else(PoisonError::into_inner);
        if *heard == (live && on) {
            return;
        }
        *heard = live && on;
        drop(heard);
        if live {
            // As with the clicks, `ScrollPeek` needs no capability.
            let _ = bus.publish(&ScrollPeek { on });
        }
    }
}
