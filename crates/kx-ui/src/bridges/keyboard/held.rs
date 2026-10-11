//! The keyboard window's held key (spec §4.6): its press, the hold timer's ticks, its let-go and a leave, each
//! answering `{board, state}`.

use super::qobject;
use cxx_qt_lib::QString;
use kx_module_api::ports::{FaceAction, KeyId};

impl qobject::Keyboard {
    pub(super) fn press(&self, id: &QString) -> QString {
        let id = KeyId::new(id.to_string());
        self.run(FaceAction::Press { id })
    }

    pub(super) fn tick(&self) -> QString {
        self.run(FaceAction::Tick)
    }

    pub(super) fn let_go(&self, id: &QString) -> QString {
        let id = KeyId::new(id.to_string());
        self.run(FaceAction::LetGo { id })
    }

    pub(super) fn leave(&self) -> QString {
        self.run(FaceAction::Leave)
    }
}
