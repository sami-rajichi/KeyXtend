//! `Ui`'s pointer overlay and scroll pad: their states as JSON, and the guard's calls for the pad's target and
//! outline.

use super::qobject;
use crate::bridges::qs;
use crate::hub::{Hub, NULL};
use cxx_qt_lib::QString;

impl qobject::Ui {
    pub(super) fn assist_json(&self) -> QString {
        qs(&self.with(NULL.to_owned(), Hub::assist_json))
    }

    pub(super) fn scroll_json(&self) -> QString {
        qs(&self.with(NULL.to_owned(), Hub::scroll_json))
    }

    pub(super) fn watch_entries(&self, title: &QString) -> bool {
        let title = title.to_string();
        self.with(false, |h| h.watch_entries(&title))
    }

    pub(super) fn unwatch_entries(&self, title: &QString) {
        let title = title.to_string();
        self.with((), |h| h.unwatch_entries(&title));
    }

    pub(super) fn lay_under(&self, title: &QString, under: &QString) -> bool {
        let (title, under) = (title.to_string(), under.to_string());
        self.with(false, |h| h.lay_under(&title, &under))
    }
}
