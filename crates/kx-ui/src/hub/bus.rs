//! The hub on the kernel's bus: it hears notices, the mouse module's state, pointer and notes, the scroll module's
//! state, outline and page turns, the chip lane, both panels and the selection strip, each through a handler that holds
//! the hub weakly, then asks the scroll, prediction and vault modules for their state, and it sends the notices'
//! answers. The keyboard's wake, power and vault changes included, is heard only while a face is attached, after the
//! face's own handlers.

use super::{Hub, Wake, sound};
use crate::ui_module::MANIFEST;
use kx_module_api::ports::{
    AssistChanged, AssistNote, FillLive, LatchesUsed, PointerMoved, PowerChanged, ScrollAimed,
    ScrollAsk, ScrollChanged, ScrollPage,
};
use kx_module_api::{Bus, Event, EventError, Notice, NoticeAnswered, Subscription};
use std::sync::{Arc, MutexGuard, PoisonError};

/// The kernel's bus once `listen` has it, and the keyboard's wake handlers while a face is attached.
#[derive(Default)]
pub(super) struct Heard {
    bus: Option<Bus>,
    /// The same bus limited to the `ui` module's capabilities, which sends the notices' answers.
    gated: Option<Bus>,
    keys: Vec<Subscription>,
}

/// Sends `event` through `handle`, one the hub keeps, with no lock held; nothing goes without one, and a refusal logs
/// `what`.
pub(super) fn send<E: Event>(handle: Option<Bus>, event: &E, what: &str) {
    if let Some(Err(e)) = handle.map(|b| b.publish(event)) {
        tracing::warn!(error = %e, "{what}");
    }
}

impl Hub {
    /// Hears every notice, mouse-assist and scroll event, the chip lane, both panels and the selection strip on `bus`
    /// until the subscriptions drop; a pad page turns the open panel, else our open list. It keeps `bus` for the
    /// keyboard's wake and a copy limited to the `ui` module's grants for the answers, and asks the scroll, prediction
    /// and vault modules for their state, so any start order works; the app calls it before the kernel loads.
    ///
    /// # Errors
    /// `EventError` when the bus refuses a subscription or an ask.
    pub fn listen(self: &Arc<Self>, bus: &Bus) -> Result<Vec<Subscription>, EventError> {
        {
            let mut kept = self.heard();
            kept.bus = Some(bus.clone());
            kept.gated = Some(bus.restricted(MANIFEST.capabilities));
        }
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
        heard.extend(self.hear_fill(bus)?);
        heard.extend(self.hear_sel(bus)?);
        bus.publish(&ScrollAsk)?;
        Ok(heard)
    }

    /// Wakes the keyboard on each mouse mode change, used latch, scroll change, power change and vault state through
    /// new handlers. The face that `attach` takes started first, so these run after its own and the window reads its
    /// new state; the last ones go.
    pub(super) fn hear_keys(self: &Arc<Self>) {
        let Some(bus) = self.heard().bus.clone() else {
            return;
        };
        let keys = [
            self.hear(&bus, |hub, _: &AssistChanged| hub.wake(Wake::Keys)),
            self.hear(&bus, |hub, _: &LatchesUsed| hub.wake(Wake::Keys)),
            self.hear(&bus, |hub, _: &ScrollChanged| hub.wake(Wake::Keys)),
            self.hear(&bus, |hub, _: &PowerChanged| hub.wake(Wake::Keys)),
            self.hear(&bus, |hub, _: &FillLive| hub.wake(Wake::Keys)),
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

    /// Sends `answer` through the handle limited to the `ui` module's capabilities; nothing goes before `listen`.
    pub(super) fn send_answer(&self, answer: &NoticeAnswered) {
        let gated = self.heard().gated.clone();
        send(gated, answer, "a notice's answer was not sent");
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

#[cfg(test)]
mod tests {
    use super::super::words::tests::hub;
    use kx_module_api::ports::PasteAsk;
    use kx_module_api::{Bus, Capability, Event, EventError, NoticeAnswered};
    use kx_test_support::host::FakeCore;
    use std::sync::Arc;

    #[test]
    fn the_kept_handle_sends_answers_and_can_never_paste() {
        let hub = hub();
        let bus = Bus::new(Arc::new(FakeCore::default()));
        let _heard = hub.listen(&bus).expect("the bus is heard");
        let kept = hub.heard().gated.clone().expect("the handle is kept");
        assert_eq!(kept.check::<NoticeAnswered>(), Ok(()));
        let refused = EventError::NotGranted(PasteAsk::NAME, Capability::WriteClipboard);
        assert_eq!(
            kept.check::<PasteAsk>(),
            Err(refused),
            "only the ui's grants"
        );
    }
}
