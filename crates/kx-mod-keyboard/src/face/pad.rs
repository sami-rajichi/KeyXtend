//! The scroll pad on the bus (spec §5.2, ADR-0029): its arrows and Stop ask the scroll module to scroll or stop, and
//! the pointer on it asks for the outline. As in `bus.rs`, nothing is sent while the face's lock is held.

use super::Face;
use super::outbox::Outbox;
use kx_module_api::ports::{
    Dir, FaceError, FaceReply, ScrollChanged, ScrollPeek, ScrollStop, ScrollTap,
};
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering::SeqCst;
use std::sync::{Mutex, PoisonError};

/// What the scroll module last heard of the pointer on the pad, so it hears each change once and the newest last.
#[derive(Default)]
pub(super) struct Peeks {
    /// Requests to check the pointer, one sending thread at a time.
    outbox: Outbox,
    /// The keyboard started, so the next live send also tells a pointer that left.
    again: AtomicBool,
    /// What the module last heard, and in which run; only the sending thread touches it.
    heard: Mutex<Peeked>,
}

/// The pointer as one run of the scroll module last heard it.
#[derive(Copy, Clone, Default)]
struct Peeked {
    /// The run, as `Inner` counts them.
    run: u64,
    /// The pointer is on the pad.
    on: bool,
    /// The last peek sent, whichever run heard it, as a module may run on unheard while the keyboard is down.
    sent: bool,
}

impl Face {
    /// A click on an arrow, or on Stop when `dir` is none: asks the scroll module while it runs, else does nothing.
    /// The module's answer redraws, not this click; while the layer hides the pad, it is no key.
    pub(super) fn pad(&self, dir: Option<Dir>) -> Result<FaceReply, FaceError> {
        let bus = {
            let inner = self.lock();
            if inner.layered() {
                return Err(FaceError::UnknownKey);
            }
            inner.scrolls().then(|| inner.bus())
        };
        let Some(bus) = bus else {
            return Ok(FaceReply::default());
        };
        // The pad's events need no capability, so none is refused.
        let _ = match dir {
            Some(dir) => bus.publish(&ScrollTap { dir }),
            None => bus.publish(&ScrollStop),
        };
        Ok(FaceReply::default())
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
    pub(super) fn peek(&self) {
        self.peeks.outbox.post(|| self.send_peek());
    }

    /// Makes the next live send also tell a pointer that left, as the keyboard starts: the scroll module may have run
    /// on unheard, still thinking the pointer is on the pad.
    pub(super) fn peek_again(&self) {
        self.peeks.again.store(true, SeqCst);
    }

    /// Sends the pointer's place when it differs from what this run of the module last heard; a new run has heard
    /// nothing, even when its stop and start merged into one send, and nothing goes out while it is off. The first live
    /// send after a start also tells a pointer that left, if the last peek sent said it was on.
    fn send_peek(&self) {
        let (live, on, run, bus) = {
            let inner = self.lock();
            (
                inner.scrolls(),
                inner.hovering(),
                inner.scroll_run(),
                inner.bus(),
            )
        };
        let mut heard = self
            .peeks
            .heard
            .lock()
            .unwrap_or_else(PoisonError::into_inner);
        let again = live && self.peeks.again.swap(false, SeqCst);
        let known = heard.on && heard.run == run;
        let send = live && (known != on || (again && heard.sent));
        *heard = Peeked {
            run,
            on: live && on,
            sent: if send { on } else { heard.sent },
        };
        drop(heard);
        if send {
            // As with the clicks, `ScrollPeek` needs no capability.
            let _ = bus.publish(&ScrollPeek { on });
        }
    }
}
