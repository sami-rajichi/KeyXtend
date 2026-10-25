//! `Ui`'s chip lane: the lane as JSON for the top bar, and its clicks sent on to the prediction module.

use super::qobject;
use crate::bridges::qs;
use crate::hub::{Hub, NULL};
use cxx_qt_lib::QString;

impl qobject::Ui {
    pub(super) fn words(&self) -> QString {
        qs(&self.with(NULL.to_owned(), Hub::words_json))
    }

    pub(super) fn pick_chip(&self, round: i64, index: i32) {
        self.with((), |h| h.pick_chip(round, index));
    }

    pub(super) fn words_ask(&self, kind: &QString, arg: i32) {
        let kind = kind.to_string();
        self.with((), |h| h.words_ask(&kind, arg));
    }
}
