//! `Ui`'s clipboard panel: the panel as JSON, its clicks sent on to the clipboard module, and Open folder.

use super::qobject;
use crate::bridges::qs;
use crate::hub::{Hub, NULL};
use cxx_qt_lib::QString;

impl qobject::Ui {
    pub(super) fn clip(&self) -> QString {
        qs(&self.with(NULL.to_owned(), Hub::clip_json))
    }

    pub(super) fn clip_act(&self, kind: &QString, arg: i32) {
        let kind = kind.to_string();
        self.with((), |h| h.clip_act(&kind, arg));
    }

    pub(super) fn open_folder(&self) {
        self.with(false, Hub::open_folder);
    }
}
