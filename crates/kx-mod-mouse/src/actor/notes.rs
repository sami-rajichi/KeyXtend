//! The engine's notes as bus events, and the state the keyboard shows; Right-click is saved when the user changes it.

use super::Actor;
use crate::Note;
use crate::settings::RCLICK_KEY;
use kx_module_api::ports::{AssistChanged, AssistNote, GrabState, LatchesUsed, PointerMoved};
use kx_module_api::{Event, Notice, keys};

impl Actor {
    /// Turns an engine note into its bus event.
    pub(super) fn note(&mut self, note: Note) {
        let s = &self.settings;
        let event = match note {
            Note::State { rclick, grab } => return self.state(rclick, grab),
            Note::Moved { .. } => return self.moved(),
            Note::LatchesUsed => return self.publish(&LatchesUsed),
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
            if self.store.set(RCLICK_KEY, rclick.into()).is_err() {
                self.publish(&Notice {
                    key: keys::SETTINGS_UNSAVED,
                    module: None,
                    args: Vec::new(),
                });
            }
        }
    }

    /// Publishes the latest pointer point, and lets the next move through.
    pub(super) fn moved(&self) {
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

    /// Publishes `event`; the start's subscriptions already checked the grants these events need.
    pub(super) fn publish<E: Event>(&self, event: &E) {
        let _ = self.bus.publish(event);
    }
}
