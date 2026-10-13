//! The face on the kernel's bus (spec §4.4, §5.1): the side keys ask the mouse module to flip a mode, its state comes
//! back, and the latched set goes out for modifier + click. Handlers run on the sender's thread, so nothing is sent
//! while the face's lock is held.

use super::Face;
use kx_module_api::ports::{
    AssistAsk, AssistChanged, AssistToggle, LatchesAsk, LatchesChanged, LatchesUsed, Scan, Toggle,
};
use kx_module_api::{Event, EventError, ModuleCx};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, TryLockError};

/// The latched set the bus last heard, so it hears each change once and in order, from any thread.
#[derive(Default)]
pub(super) struct Told {
    /// A thread asks for the set to be checked; the thread holding `last` checks it.
    due: AtomicBool,
    /// `LatchesAsk` came, so the set goes out even if unchanged.
    again: AtomicBool,
    /// The set last sent, held by the one thread sending.
    last: Mutex<Vec<Scan>>,
}

impl Face {
    /// Sends the latched set when it changed since last sent, or always when `again`. A thread already sending
    /// sends it instead, so the bus hears the sets in the order they were held, and a handler may ask again.
    pub(super) fn tell(&self, again: bool) {
        let t = &self.told;
        t.again.fetch_or(again, Ordering::AcqRel);
        t.due.store(true, Ordering::Release);
        while t.due.load(Ordering::Acquire) {
            let mut last = match t.last.try_lock() {
                Ok(last) => last,
                Err(TryLockError::Poisoned(p)) => p.into_inner(),
                Err(TryLockError::WouldBlock) => return,
            };
            t.due.store(false, Ordering::Release);
            let held = self.lock().held();
            if t.again.swap(false, Ordering::AcqRel) || *last != held {
                last.clone_from(&held);
                let bus = self.lock().bus();
                // The module holds `InjectInput`, which this event needs, or its start would have failed.
                let _ = bus.publish(&LatchesChanged { held });
            }
        }
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

/// Hears the mouse module's state and its latch events for `face` until the module stops, then asks the mouse
/// module for its state, as it may have started first.
///
/// # Errors
/// `EventError::NotGranted` when the module lacks `InjectInput`, which the latch events need.
pub(crate) fn listen(cx: &mut ModuleCx<'_>, face: &Arc<Face>) -> Result<(), EventError> {
    on(cx, face, |f, e: &AssistChanged| f.lock().heard(*e))?;
    on(cx, face, |f, _: &LatchesUsed| f.used())?;
    on(cx, face, |f, _: &LatchesAsk| f.tell(true))?;
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
