//! Caps Lock as the UI's thread saw it, since Windows reads it per thread: the last read, the view shown, and our own
//! click's flip until the system has it.

use super::Inner;
use kx_module_api::Mono;
use std::time::Duration;

/// Caps Lock as the UI's thread saw it.
#[derive(Default)]
pub(super) struct CapsLook {
    /// What the last state view showed, so a press outside the keyboard redraws it.
    pub(super) shown: Option<bool>,
    /// The last read, which the typing mode tells from any thread.
    pub(super) seen: bool,
    /// When our Caps click flipped `seen`; the UI thread's refresh replaces the flip, and any read a relabel period
    /// later does too, as when the window hid before its tick.
    flipped: Option<Mono>,
}

impl Inner {
    /// True when Caps Lock changed since the last state view showed it, as after a press on a real keyboard. The UI
    /// thread's refresh calls it, at the relabel tick or after a stale tap, and its read replaces our flip.
    pub(in crate::face) fn caps_moved(&mut self) -> bool {
        self.caps.flipped = None;
        let now = self.look_caps();
        self.caps.shown.is_some_and(|shown| shown != now)
    }

    /// Whether Caps Lock shows lit, as the layout source says on the UI's thread, kept for the typing mode. Within one
    /// `relabel_ms` of our flip the kept view stands, as the system may not have it yet.
    pub(super) fn look_caps(&mut self) -> bool {
        let period = Duration::from_millis(self.settings.relabel_ms);
        let now = self.ports.clock.now();
        if self
            .caps
            .flipped
            .is_none_or(|at| now >= at.saturating_add(period))
        {
            self.caps.flipped = None;
            self.caps.seen = self.ports.source.caps_on();
        }
        self.caps.seen
    }

    /// After our Caps Lock tap went out, the kept view flips at once. The next tick's read, or a state read one
    /// `relabel_ms` after the tap, replaces it, whichever comes first.
    pub(super) fn follow_caps(&mut self) {
        if self.typing.took_caps() {
            self.caps.seen = !self.caps.seen;
            self.caps.flipped = Some(self.ports.clock.now());
        }
    }
}
