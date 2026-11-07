//! The `QuickFill` object: the Quick-fill panel as JSON and its clicks, sent on to the vault. It takes the hub's panel
//! wakes as it starts and tells QML on the Qt thread, so make one, at the app's root; a new one takes them over.

use super::{hub, qs};
use crate::hub::{Hub, NULL};
use cxx_qt::{CxxQtType, Threading};
use cxx_qt_lib::QString;
use std::pin::Pin;
use std::sync::Arc;

/// The cxx-qt bridge that makes `QuickFill` a QML type.
#[cxx_qt::bridge]
pub mod qobject {
    // SAFETY: the header and type are cxx-qt-lib's own QString binding, so both sides match.
    unsafe extern "C++" {
        include!("cxx-qt-lib/qstring.h");
        /// Qt string.
        type QString = cxx_qt_lib::QString;
    }

    #[auto_cxx_name]
    extern "RustQt" {
        #[qobject]
        #[qml_element]
        type QuickFill = super::QuickFillRust;

        /// The panel as JSON, every text worded, or null while closed; reading it lets the next change wake.
        #[qinvokable]
        fn view(&self) -> QString;

        /// A click on the panel drawn from view `round`: `kind` names it, `id` its button, filter or kind, and `arg`
        /// its row or field place, page way or answer.
        #[qinvokable]
        fn act(&self, kind: &QString, id: &QString, arg: i32, round: i64);

        /// The panel shows a new view, closes, or speaks a new language: read `view`.
        #[qsignal]
        fn view_changed(self: Pin<&mut Self>);
    }

    impl cxx_qt::Threading for QuickFill {}
    impl cxx_qt::Initialize for QuickFill {}
}

/// Rust side of `QuickFill`.
pub struct QuickFillRust {
    hub: Option<Arc<Hub>>,
}

impl Default for QuickFillRust {
    fn default() -> Self {
        Self { hub: hub() }
    }
}

impl cxx_qt::Initialize for qobject::QuickFill {
    fn initialize(self: Pin<&mut Self>) {
        let Some(hub) = self.rust().hub.clone() else {
            return;
        };
        let qt = self.qt_thread();
        hub.set_fill_waker(Box::new(move || {
            // A queue refused after the object is gone has nobody left to tell.
            let _ = qt.queue(qobject::QuickFill::view_changed);
        }));
    }
}

impl qobject::QuickFill {
    fn view(&self) -> QString {
        qs(&self.with(NULL.to_owned(), Hub::fill_json))
    }

    fn act(&self, kind: &QString, id: &QString, arg: i32, round: i64) {
        let (kind, id) = (kind.to_string(), id.to_string());
        self.with((), |h| h.fill_act(&kind, &id, arg, round));
    }

    /// `f` over the hub, or `none` before the app installed one.
    fn with<T>(&self, none: T, f: impl FnOnce(&Hub) -> T) -> T {
        self.rust().hub.as_deref().map_or(none, f)
    }
}
