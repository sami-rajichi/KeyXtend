//! A held key on the face (spec §4.6): a press starts its hold, the hold timer types its repeats with the latches
//! kept down, and a let-go, a leave or a repeat that cannot go on ends it.

use super::{Inner, STATE};
use crate::run::{TapError, Tapped};
use kx_module_api::ports::FaceReply;

/// One beat of the hold timer that found a repeat due.
pub(in crate::face) struct Beat {
    /// What typing the repeat did.
    pub(in crate::face) tapped: Result<Tapped, TapError>,
    /// The burst ended, and its latches let go.
    pub(in crate::face) ended: bool,
}

impl Inner {
    /// The pointer went down on key `id`; it repeats only with repeat on, no list open, and a key that may repeat now.
    pub(in crate::face) fn press(&mut self, id: &str) -> FaceReply {
        let ended = self.ended();
        let table = self.ports.layouts.current();
        let may = self.settings.repeat && self.list.is_none();
        let repeats = may && self.typing.repeats(&self.ports.io(&table), id);
        self.repeat
            .press(id.to_owned(), repeats, self.ports.clock.now());
        ended
    }

    /// The hold timer: when a repeat is due, types the held key with the latches kept down, and ends the burst once
    /// the key cannot go on.
    pub(in crate::face) fn tick(&mut self) -> Option<Beat> {
        let id = self.repeat.tick(self.ports.clock.now())?;
        self.typing.hold();
        let table = self.ports.layouts.current();
        let io = self.ports.io(&table);
        let tapped = self.typing.tap(&io, &id, false);
        let on = matches!(tapped, Ok(Tapped::Done)) && self.typing.repeats(&io, &id);
        let ended = !on && self.ended().state;
        Some(Beat { tapped, ended })
    }

    /// The pointer let go of key `id` on it: `None` for a click, which the face taps now, else the burst's end.
    pub(in crate::face) fn let_go(&mut self, id: &str) -> Option<FaceReply> {
        match self.repeat.let_go(id) {
            Some(_) => None,
            None => Some(self.ended()),
        }
    }

    /// The pointer left the held key, or the keyboard hid: nothing more types.
    pub(in crate::face) fn leave(&mut self) -> FaceReply {
        self.ended()
    }

    /// Drops the held key and ends its burst; the state redraws when the burst's latches let go.
    pub(super) fn ended(&mut self) -> FaceReply {
        self.repeat.leave();
        if self.typing.release() {
            STATE
        } else {
            FaceReply::default()
        }
    }
}
