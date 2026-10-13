//! The face on the kernel's bus (spec §4.4, §5.1): the side keys ask the mouse module to flip a mode, its state comes
//! back, and the latched set goes out for modifier + click. Handlers run on the sender's thread, so nothing is sent
//! while the face's lock is held.

use super::Face;
use kx_module_api::ports::{
    AssistAsk, AssistChanged, AssistToggle, LatchesAsk, LatchesChanged, LatchesUsed, Scan, Toggle,
};
use kx_module_api::{Event, EventError, ModuleCx};
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering::SeqCst;
use std::sync::{Arc, Mutex, PoisonError};

/// The latched set the bus last heard, so it hears each change once and the newest set last, from any thread.
#[derive(Default)]
pub(super) struct Told {
    /// A thread asks for the set to be checked.
    due: AtomicBool,
    /// `LatchesAsk` came, or the module started, so the set goes out even if unchanged.
    again: AtomicBool,
    /// One thread is sending; the others leave their requests to it.
    sending: AtomicBool,
    /// The set last sent, which only the sending thread touches.
    last: Mutex<Vec<Scan>>,
}

/// Clears the `sending` flag when the sending thread stops, even by a panic, so later sets still go out.
struct Sending<'a>(&'a AtomicBool);

impl Drop for Sending<'_> {
    fn drop(&mut self) {
        self.0.store(false, SeqCst);
    }
}

impl Face {
    /// Sends the latched set when it changed since last sent, or always when `again`. One thread sends at a time and
    /// takes every request made before it stops, so the bus hears the newest set last, and a handler may ask again.
    pub(super) fn tell(&self, again: bool) {
        let t = &self.told;
        if again {
            t.again.store(true, SeqCst);
        }
        t.due.store(true, SeqCst);
        loop {
            if t.sending
                .compare_exchange(false, true, SeqCst, SeqCst)
                .is_err()
            {
                return;
            }
            let sending = Sending(&t.sending);
            while t.due.swap(false, SeqCst) {
                self.send(t.again.swap(false, SeqCst));
            }
            drop(sending);
            // A request made after the last check but before the stop found the sender busy, so it is ours.
            if !t.due.load(SeqCst) {
                return;
            }
        }
    }

    /// Sends the latched set when it differs from the last one sent, or when `again`; only the sending thread calls it.
    fn send(&self, again: bool) {
        let held = self.lock().held();
        let mut last = self
            .told
            .last
            .lock()
            .unwrap_or_else(PoisonError::into_inner);
        if !again && *last == held {
            return;
        }
        last.clone_from(&held);
        drop(last);
        let bus = self.lock().bus();
        // The module holds `InjectInput`, which this event needs, or its start would have failed.
        let _ = bus.publish(&LatchesChanged { held });
    }

    /// A click on a mouse-assist side key: asks the mouse module to flip `which` while it runs, else does nothing.
    pub(super) fn toggle(&self, which: Toggle) {
        let bus = {
            let inner = self.lock();
            inner.live().then(|| inner.bus())
        };
        if let Some(bus) = bus {
            // As in `tell`, the module holds the `InjectInput` this event needs.
            let _ = bus.publish(&AssistToggle { which });
        }
    }

    /// A mouse click used the latches: they let go as their mode says, and the bus hears it.
    fn used(&self) {
        self.lock().clicked();
        self.tell(false);
    }
}

/// Hears the mouse module's state and its latch events for `face` until the module stops, then tells the latched set
/// and asks the mouse module for its state, as either may have missed the other while one was stopped.
///
/// # Errors
/// `EventError::NotGranted` when the module lacks `InjectInput`, which the latch events need.
pub(crate) fn listen(cx: &mut ModuleCx<'_>, face: &Arc<Face>) -> Result<(), EventError> {
    on(cx, face, |f, e: &AssistChanged| f.lock().heard(*e))?;
    on(cx, face, |f, _: &LatchesUsed| f.used())?;
    on(cx, face, |f, _: &LatchesAsk| f.tell(true))?;
    face.tell(true);
    cx.bus().publish(&AssistAsk)
}

/// Calls `then` with the face for each `E` on the bus, while the face lives.
fn on<E: Event>(
    cx: &mut ModuleCx<'_>,
    face: &Arc<Face>,
    then: impl Fn(&Face, &E) + Send + Sync + 'static,
) -> Result<(), EventError> {
    let face = Arc::downgrade(face);
    cx.subscribe(move |e: &E| {
        if let Some(face) = face.upgrade() {
            then(&face, e);
        }
    })
}
