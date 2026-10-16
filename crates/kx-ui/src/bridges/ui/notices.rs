//! `Ui`'s notices: the banner takes them and tells them again, and answers each with Try again or OK.

use super::qobject;
use crate::bridges::qs;
use crate::hub::{NULL, json};
use cxx_qt_lib::QString;

impl qobject::Ui {
    pub(super) fn take_notices(&self) -> QString {
        qs(&self.with(NULL.to_owned(), |h| json(&h.take_notices())))
    }

    pub(super) fn shown_notices(&self) -> QString {
        qs(&self.with(NULL.to_owned(), |h| json(&h.shown_notices())))
    }

    pub(super) fn retry(&self, id: i64) -> bool {
        let id = u64::try_from(id).unwrap_or_default();
        self.with(false, |h| h.retry(id))
    }

    pub(super) fn dismiss(&self, id: i64) {
        let id = u64::try_from(id).unwrap_or_default();
        self.with((), |h| h.dismiss(id));
    }
}
