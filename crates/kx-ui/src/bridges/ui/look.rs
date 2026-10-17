//! `Ui`'s look: the look to draw, the theme and light or dark the user steps through, and the backdrop behind a window.

use super::qobject;
use crate::bridges::qs;
use crate::hub::{Hub, NULL};
use cxx_qt_lib::QString;

impl qobject::Ui {
    pub(super) fn look_json(&self) -> QString {
        qs(&self.with(NULL.to_owned(), Hub::look_json))
    }

    pub(super) fn step_theme(&self) -> bool {
        self.with(false, Hub::step_theme)
    }

    pub(super) fn flip_mode(&self) -> bool {
        self.with(false, Hub::flip_mode)
    }

    pub(super) fn backdrop(&self, title: &QString) -> bool {
        let title = title.to_string();
        self.with(false, |h| h.backdrop(&title))
    }
}
