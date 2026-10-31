//! `Ui`'s ties to the app: the timings and paging QML reads, how the user left the keyboard, Quit, and the assets.

use super::qobject;
use crate::bridges::qs;
use crate::hub::{Hub, Mode, NULL};
use cxx_qt_lib::QString;

impl qobject::Ui {
    pub(super) fn settings_json(&self) -> QString {
        qs(&self.with(NULL.to_owned(), Hub::settings_json))
    }

    pub(super) fn mode_json(&self) -> QString {
        qs(&self.with(NULL.to_owned(), Hub::mode_json))
    }

    pub(super) fn keep_mode(&self, away: bool, minimised: bool) {
        self.with((), |h| h.keep_mode(Mode::of(away, minimised)));
    }

    pub(super) fn quit(&self) -> bool {
        self.with(false, Hub::quit)
    }

    #[allow(clippy::unused_self, reason = "cxx-qt invokables take the object")]
    pub(super) fn assets(&self) -> QString {
        qs(crate::ASSETS)
    }
}
