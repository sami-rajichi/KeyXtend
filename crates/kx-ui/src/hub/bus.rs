//! The hub on the kernel's bus: it hears notices, the mouse module's state, pointer and notes, the scroll module's
//! state, outline and page turns, the chip lane and the clipboard panel, each through a handler that holds the hub
//! weakly, then asks the scroll and prediction modules for their state. The keyboard's wake, power changes included, is heard only while a
//! face is attached, after the face's own handlers.

use super::{Hub, Wake, sound};
use kx_module_api::ports::{
    AssistChanged, AssistNote, LatchesUsed, PointerMoved, PowerChanged, ScrollAimed, ScrollAsk,
    ScrollChanged, ScrollPage,
};
use kx_module_api::{Bus, Event, EventError, Notice, Subscription};
use std::sync::{Arc, MutexGuard, PoisonError};

/// The kernel's bus once `listen` has it, and the keyboard's wake handlers while a face is attached.
#[derive(Default)]
pub(super) struct Heard {
    bus: Option<Bus>,
    keys: Vec<Subscription>,
}

impl Hub {
    /// Hears every notice, mouse-assist and scroll event, the chip lane and the clipboard panel on `bus` until the
    /// subscriptions drop; a pad page turns the open panel, else our open list. It keeps
    /// `bus` for the keyboard's wake, and asks the scroll and prediction modules for their state, so any start order
    /// works; the app calls it before the kernel loads.
    ///
    /// # Errors
    /// `EventError` when the bus refuses a subscription or an ask.
    pub fn listen(self: &Arc<Self>, bus: &Bus) -> Result<Vec<Subscription>, EventError> {
        self.heard().bus = Some(bus.clone());
        let mut heard = vec![
            self.hear(bus, |hub, notice: &Notice| hub.notice(notice.clone()))?,
            self.hear(bus, Self::note)?,
            self.hear(bus, |hub, now: &AssistChanged| hub.assist_changed(*now))?,
            self.hear(bus, |hub, moved: &PointerMoved| hub.pointer_moved(moved.at))?,
            self.hear(bus, |hub, now: &ScrollChanged| hub.scroll_changed(*now))?,
            self.hear(bus, |hub, aimed: &ScrollAimed| hub.scroll_aimed(*aimed))?,
            self.hear(bus, |hub, page: &ScrollPage| hub.pad_page(page.back))?,
        ];
        heard.push(self.hear_words(bus)?);
        heard.extend(self.hear_clip(bus)?);
        bus.publish(&ScrollAsk)?;
        Ok(heard)
    }

    /// Wakes the keyboard on each mouse mode change, used latch, scroll change and power change through new handlers.
    /// The face that `attach` takes started first, so these run after its own and the window reads its new state; the
    /// last ones go.
    pub(super) fn hear_keys(self: &Arc<Self>) {
        let Some(bus) = self.heard().bus.clone() else {
            return;
        };
        let keys = [
            self.hear(&bus, |hub, _: &AssistChanged| hub.wake(Wake::Keys)),
            self.hear(&bus, |hub, _: &LatchesUsed| hub.wake(Wake::Keys)),
            self.hear(&bus, |hub, _: &ScrollChanged| hub.wake(Wake::Keys)),
            self.hear(&bus, |hub, _: &PowerChanged| hub.wake(Wake::Keys)),
        ];
        let keys: Vec<Subscription> = keys
            .into_iter()
            .filter_map(|k| {
                k.map_err(|e| tracing::warn!(error = %e, "the keys will light at the next relabel"))
                    .ok()
            })
            .collect();
        let last = std::mem::replace(&mut self.heard().keys, keys);
        drop(last);
    }

    /// Lets the keyboard's wake handlers go, outside the lock.
    pub(super) fn unhear_keys(&self) {
        let last = std::mem::take(&mut self.heard().keys);
        drop(last);
    }

    fn heard(&self) -> MutexGuard<'_, Heard> {
        self.heard.lock().unwrap_or_else(PoisonError::into_inner)
    }

    /// Calls `f` on the hub for every `E` on `bus`, holding the hub weakly.
    pub(super) fn hear<E: Event>(
        self: &Arc<Self>,
        bus: &Bus,
        f: impl Fn(&Self, &E) + Send + Sync + 'static,
    ) -> Result<Subscription, EventError> {
        let hub = Arc::downgrade(self);
        bus.subscribe(move |event: &E| {
            if let Some(hub) = hub.upgrade() {
                f(&hub, event);
            }
        })
    }

    /// Hands a mouse-assist note to each part of the hub that follows one: the sounds and the overlay.
    fn note(&self, note: &AssistNote) {
        sound::on_note(self, note);
        self.assist_note(note);
    }
}
