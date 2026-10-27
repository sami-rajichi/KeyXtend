//! The engine's notes as bus events, and the state the keyboard shows; Right-click is saved when the user changes it.

use super::Actor;
use crate::Note;
use crate::settings::RCLICK_KEY;
use kx_module_api::Event;
use kx_module_api::ports::{
    AssistChanged, AssistNote, GrabState, LatchesUsed, PointerMoved, PointerPressed,
};

impl Actor {
    /// Turns an engine note into its bus event.
    pub(super) fn note(&mut self, note: Note) {
        let s = &self.settings;
        let event = match note {
            Note::State { rclick, grab } => return self.state(rclick, grab),
            Note::Moved { .. } => return self.moved(),
            Note::LatchesUsed => return self.publish(&LatchesUsed),
            Note::Pressed { ours } => return self.publish(&PointerPressed { ours }),
            Note::HoldStart { at } => AssistNote::HoldStart {
                at,
                ring_after_ms: s.ring_after_ms,
                hold_ms: s.hold_ms,
            },
            Note::HoldEnd => AssistNote::HoldEnd,
            Note::Fired { at } => AssistNote::Fired { at },
            Note::Picked { at } => AssistNote::Picked { at },
            Note::Dropped { at } => AssistNote::Dropped { at },
            Note::Cancelled => AssistNote::Cancelled,
        };
        self.publish(&event);
    }

    /// Tells the new state, and saves Right-click when the user changed it; Grab's pause is not a change.
    fn state(&mut self, rclick: bool, grab: GrabState) {
        (self.rclick, self.grab) = (rclick, grab);
        self.tell_state();
        if grab == GrabState::Off && rclick != self.saved {
            self.saved = rclick;
            self.save(rclick);
        }
    }

    /// Saves Right-click. A file left unsaved is the kernel's to tell, once per run; a refused write is logged by
    /// its kind only, as its text may quote a value.
    fn save(&self, rclick: bool) {
        if let Err(error) = self.store.set(RCLICK_KEY, rclick.into())
            && !error.holds()
        {
            tracing::warn!(kind = error.kind(), "mouse setting not written");
        }
    }

    /// Publishes the latest pointer point, which may be newer than the note's, and lets the next move through.
    fn moved(&self) {
        let at = self.moves.take();
        self.publish(&PointerMoved { at });
    }

    /// Publishes the live state.
    pub(super) fn tell_state(&self) {
        let (rclick, grab) = (self.rclick, self.grab);
        self.publish(&AssistChanged {
            live: true,
            rclick,
            grab,
        });
    }

    /// Publishes `event`; the start checked `InjectInput` through its subscriptions and `KeyboardUi` directly, so
    /// none is refused.
    pub(super) fn publish<E: Event>(&self, event: &E) {
        let _ = self.bus.publish(event);
    }
}
