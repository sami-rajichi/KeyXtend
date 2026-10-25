//! The face on the kernel's bus (spec §4.4, §5.1, §5.2, §6, §8.5): the side keys ask the mouse module to flip a mode
//! and the power module for its dialog, the mouse, scroll and power modules' states come back, the latched set goes
//! out for modifier + click, the typing mode for word prediction, and picked words come in. Handlers run on the
//! sender's thread, so nothing is sent while the face's lock is held.

use super::Face;
use super::outbox::Outbox;
use kx_module_api::ports::{
    AssistAsk, AssistChanged, AssistToggle, LatchesAsk, LatchesChanged, LatchesUsed, PowerAsk,
    PowerAsked, PowerChanged, Scan, ScrollAsk, ScrollChanged, Toggle, TypeWord, TypingMode,
};
use kx_module_api::{Event, EventError, ModuleCx};
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering::SeqCst;
use std::sync::{Arc, Mutex, PoisonError};

/// The latched set and typing mode the bus last heard, so it hears each change once and the newest last, from any
/// thread.
#[derive(Default)]
pub(super) struct Told {
    /// Requests to check them, one sending thread at a time.
    outbox: Outbox,
    /// `LatchesAsk` came, or the module started, so the set goes out even if unchanged.
    again: AtomicBool,
    /// The module started, so the mode goes out even if unchanged.
    fresh: AtomicBool,
    /// What was last sent, which only the sending thread touches.
    last: Mutex<Last>,
}

/// The latched set and typing mode last sent.
#[derive(Default)]
struct Last {
    held: Vec<Scan>,
    mode: TypingMode,
}

impl Face {
    /// Sends the latched set and the typing mode when they changed since last sent, the set always when `again`,
    /// through the outbox, so the bus hears the newest last and a handler may ask again.
    pub(super) fn tell(&self, again: bool) {
        let t = &self.told;
        if again {
            t.again.store(true, SeqCst);
        }
        let flags = || (t.again.swap(false, SeqCst), t.fresh.swap(false, SeqCst));
        t.outbox.post(|| self.send(flags()));
    }

    /// At start, the latched set and the typing mode go out even when unchanged.
    fn tell_all(&self) {
        self.told.fresh.store(true, SeqCst);
        self.tell(true);
    }

    /// Sends the latched set when it differs from the last sent or `again`, then the mode when it differs or `fresh`;
    /// only the sending thread calls it.
    fn send(&self, (again, fresh): (bool, bool)) {
        let (held, mode, bus) = {
            let inner = self.lock();
            (inner.held(), inner.mode(), inner.bus())
        };
        let mut last = self
            .told
            .last
            .lock()
            .unwrap_or_else(PoisonError::into_inner);
        let latches = again || last.held != held;
        let moved = fresh || last.mode != mode;
        last.held.clone_from(&held);
        last.mode = mode;
        drop(last);
        if latches {
            // The module holds `InjectInput`, which this event needs, or its start would have failed.
            let _ = bus.publish(&LatchesChanged { held });
        }
        if moved {
            // It holds `ReadFocusedText` too, which the mode needs.
            let _ = bus.publish(&mode);
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

    /// A click on the Power key: asks the power module for the system's dialog while it runs, else does nothing.
    pub(super) fn power(&self) {
        let bus = {
            let inner = self.lock();
            inner.powers().then(|| inner.bus())
        };
        if let Some(bus) = bus {
            // The power events need no capability, so none is refused.
            let _ = bus.publish(&PowerAsked);
        }
    }

    /// A mouse click used the latches: they let go as their mode says, and the bus hears it.
    fn used(&self) {
        self.lock().clicked();
        self.tell(false);
    }

    /// Forgets the mouse, scroll and power modules' states, as each may have stopped unheard while the keyboard was
    /// down, so their keys and the pad dim until the answers; the next live peek also tells a pointer that left.
    fn forget(&self) {
        {
            let mut inner = self.lock();
            inner.heard(AssistChanged::OFF);
            inner.heard_scroll(ScrollChanged::OFF);
            inner.heard_power(PowerChanged::OFF);
        }
        self.peek_again();
    }
}

/// Hears the mouse, scroll and power modules' states, the latch events and picked words for `face` until the module
/// stops, then tells the latched set and typing mode, forgets the three modules' states and asks for them, as each may
/// have missed the others while it was down.
///
/// # Errors
/// `EventError::NotGranted` when the module lacks `InjectInput`, which the latch events need, or `ReadFocusedText`,
/// which picked words need.
pub(crate) fn listen(cx: &mut ModuleCx<'_>, face: &Arc<Face>) -> Result<(), EventError> {
    on(cx, face, |f, e: &AssistChanged| f.lock().heard(*e))?;
    on(cx, face, |f, _: &LatchesUsed| f.used())?;
    on(cx, face, |f, _: &LatchesAsk| f.tell(true))?;
    on(cx, face, |f, e: &ScrollChanged| f.scrolled(*e))?;
    on(cx, face, |f, e: &PowerChanged| f.lock().heard_power(*e))?;
    on(cx, face, |f, e: &TypeWord| f.type_word(e))?;
    face.tell_all();
    face.forget();
    cx.bus().publish(&AssistAsk)?;
    cx.bus().publish(&ScrollAsk)?;
    cx.bus().publish(&PowerAsk)
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
