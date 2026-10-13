//! The hub on the kernel's bus: it hears notices and mouse-assist notes, each through a handler that holds the hub
//! weakly, and hands every note to the parts that follow one.

use super::{Hub, sound};
use kx_module_api::ports::AssistNote;
use kx_module_api::{Bus, Event, EventError, Notice, Subscription};
use std::sync::Arc;

impl Hub {
    /// Hears every notice and mouse-assist note on `bus` until the subscriptions drop; the app calls it before the
    /// kernel loads.
    ///
    /// # Errors
    /// `EventError` when the bus refuses a subscription.
    pub fn listen(self: &Arc<Self>, bus: &Bus) -> Result<Vec<Subscription>, EventError> {
        Ok(vec![
            self.hear(bus, |hub, notice: &Notice| hub.notice(notice.clone()))?,
            self.hear(bus, Self::note)?,
        ])
    }

    /// Calls `f` on the hub for every `E` on `bus`, holding the hub weakly.
    fn hear<E: Event>(
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

    /// Hands a mouse-assist note to each part of the hub that follows one.
    fn note(&self, note: &AssistNote) {
        sound::on_note(self, note);
    }
}
