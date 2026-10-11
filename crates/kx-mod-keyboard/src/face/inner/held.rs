//! A held key on the face (spec §4.6): a press types at once and, for a key that may repeat, starts its hold; the
//! hold timer types its repeats with the latches kept down, and a let-go, a leave or a repeat that cannot go on ends
//! it.

use super::{Inner, STATE};
use crate::run::{TapError, Tapped};
use kx_module_api::ports::FaceReply;

/// What a click, a press or a due repeat did.
#[derive(Clone, Copy)]
pub(in crate::face) struct Went {
    /// What typing the key did.
    pub(in crate::face) tapped: Result<Tapped, TapError>,
    /// A burst ended, and its latches let go.
    pub(in crate::face) ended: bool,
}

impl Inner {
    /// The pointer went down on key `id`: it types now, and holds when repeat is on, no list is open and the key may
    /// repeat now, keeping the latches down until its burst ends; a press that typed nothing never holds.
    pub(in crate::face) fn press(&mut self, id: &str) -> Went {
        let ended = self.ended().state;
        let table = self.ports.layouts.current();
        let io = self.ports.io(&table);
        let may = self.settings.repeat && self.list.is_none();
        let repeats = may && self.typing.repeats(&io, id);
        if repeats {
            self.typing.hold();
        }
        let tapped = self.typing.tap(&io, id, false);
        if repeats && matches!(tapped, Ok(Tapped::Done)) {
            self.repeat.press(id.to_owned(), self.ports.clock.now());
            return Went { tapped, ended };
        }
        let own = repeats && self.ended().state;
        Went {
            tapped,
            ended: ended || own,
        }
    }

    /// The hold timer: when a repeat is due, types the held key with the latches kept down, and ends the burst once
    /// the key cannot go on.
    pub(in crate::face) fn tick(&mut self) -> Option<Went> {
        let id = self.repeat.tick(self.ports.clock.now())?;
        self.typing.hold();
        let table = self.ports.layouts.current();
        let io = self.ports.io(&table);
        let tapped = self.typing.tap(&io, &id, false);
        let on = matches!(tapped, Ok(Tapped::Done)) && self.typing.repeats(&io, &id);
        let ended = !on && self.ended().state;
        Some(Went { tapped, ended })
    }

    /// The pointer let go, Qt took the press away, or the keyboard hid: nothing more types.
    pub(in crate::face) fn leave(&mut self) -> FaceReply {
        self.ended()
    }

    /// True while a key is held, so the UI keeps its beat.
    pub(in crate::face) fn holding(&self) -> bool {
        self.repeat.held()
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
