//! A held key on the face (spec §4.6): the hold timer's repeats and the let-go; the press that starts a hold goes with
//! the clicks in `face.rs`.

use super::Face;
use crate::run::Tapped;
use kx_module_api::ports::{FaceError, FaceReply};

impl Face {
    /// The hold timer: a repeat that went out redraws only when its burst ended, as the latches stay latched until
    /// then; any other due repeat answers as a tap does. Each reply says whether the key is still held.
    pub(super) fn tick(&self) -> Result<FaceReply, FaceError> {
        let (went, holding) = {
            let mut inner = self.lock();
            let went = inner.tick();
            (went, inner.holding())
        };
        let Some(went) = went else {
            return Ok(FaceReply {
                holding,
                ..FaceReply::default()
            });
        };
        if matches!(went.tapped, Ok(Tapped::Done)) {
            return Ok(FaceReply {
                state: went.ended,
                holding,
                ..FaceReply::default()
            });
        }
        Ok(FaceReply {
            holding,
            ..self.reply(went)?
        })
    }

    /// The pointer let go of key `id`, wherever it is: it typed at the press, so only its burst ends, and only when
    /// `id` is the key held.
    pub(super) fn let_go(&self, id: &str) -> Result<FaceReply, FaceError> {
        self.known(id)?;
        Ok(self.lock().let_go(id))
    }
}
