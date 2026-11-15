//! The `Voice` object: the dictation bar and the Voice panel as JSON, the sound level, their clicks, the tray's
//! Dictation row and the bar's and panel's spots. It takes the hub's voice wakes as it starts and tells QML on the Qt thread, so make
//! one, at the app's root; a new one takes them over.

use super::{hub, keyboard_box, qs};
use crate::hub::{Hub, NULL, VoiceWake, json};
use cxx_qt::{CxxQtType, Threading};
use cxx_qt_lib::QString;
use std::pin::Pin;
use std::sync::Arc;

/// The cxx-qt bridge that makes `Voice` a QML type.
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
        type Voice = super::VoiceRust;

        /// The bar and the panel as JSON, every word in the keyboard's language, or null before any view.
        #[qinvokable]
        fn view(&self) -> QString;

        /// The sound level while the bar listens, 0 to 1.
        #[qinvokable]
        fn level(&self) -> f64;

        /// A click on view `round`: `kind` and `id` name it, and `arg` its row, page way or answer, as a button gives.
        #[qinvokable]
        fn act(&self, kind: &QString, id: &QString, arg: i32, round: i64);

        /// The tray's Dictation row: a hidden keyboard shows first, then the panel opens.
        #[qinvokable]
        fn open(&self);

        /// The bar started (`start`) or stopped listening: the start or stop sound plays, unless sounds are off.
        #[qinvokable]
        fn chime(&self, start: bool);

        /// Where a `w` by `h` bar goes by the keyboard box `over` (JSON, physical pixels, null while hidden), as
        /// JSON `{x, y}` in physical pixels, or null.
        #[qinvokable]
        fn bar_at(&self, over: &QString, w: f64, h: f64) -> QString;

        /// Where a `w` by `h` Voice panel goes by the keyboard box `over` and the bar box `bar` (JSON, physical pixels,
        /// null while hidden), as JSON `{x, y, fit}` in physical pixels, or null.
        #[qinvokable]
        fn panel_at(&self, over: &QString, bar: &QString, w: f64, h: f64) -> QString;

        /// A new view, or the keyboard's language changed: read `view`.
        #[qsignal]
        fn view_changed(self: Pin<&mut Self>);

        /// A new sound level: read `level`.
        #[qsignal]
        fn level_changed(self: Pin<&mut Self>);
    }

    impl cxx_qt::Threading for Voice {}
    impl cxx_qt::Initialize for Voice {}
}

/// Rust side of `Voice`.
pub struct VoiceRust {
    hub: Option<Arc<Hub>>,
}

impl Default for VoiceRust {
    fn default() -> Self {
        Self { hub: hub() }
    }
}

impl cxx_qt::Initialize for qobject::Voice {
    fn initialize(self: Pin<&mut Self>) {
        let Some(hub) = self.rust().hub.clone() else {
            return;
        };
        let qt = self.qt_thread();
        hub.set_voice_waker(Box::new(move |what| {
            // A queue refused after the object is gone has nobody left to tell.
            let _ = match what {
                VoiceWake::View => qt.queue(qobject::Voice::view_changed),
                VoiceWake::Level => qt.queue(qobject::Voice::level_changed),
            };
        }));
    }
}

impl qobject::Voice {
    fn view(&self) -> QString {
        qs(&self.with(NULL.to_owned(), Hub::voice_json))
    }

    fn level(&self) -> f64 {
        self.with(0.0, Hub::voice_level)
    }

    fn act(&self, kind: &QString, id: &QString, arg: i32, round: i64) {
        let (kind, id) = (kind.to_string(), id.to_string());
        self.with((), |h| h.voice_act(&kind, &id, arg, round));
    }

    fn open(&self) {
        self.with((), Hub::voice_open);
    }

    fn chime(&self, start: bool) {
        self.with((), |h| h.voice_chime(start));
    }

    fn bar_at(&self, over: &QString, w: f64, h: f64) -> QString {
        let Ok(over) = keyboard_box(&over.to_string()) else {
            tracing::warn!("QML sent a keyboard box that does not read");
            return qs(NULL);
        };
        let spot = self.with(None, |hub| hub.voice_at(over, w, h));
        qs(&json(&spot))
    }

    fn panel_at(&self, over: &QString, bar: &QString, w: f64, h: f64) -> QString {
        let boxes = keyboard_box(&over.to_string())
            .and_then(|kb| Ok((kb, keyboard_box(&bar.to_string())?)));
        let Ok((kb, bar)) = boxes else {
            tracing::warn!("QML sent a keyboard or bar box that does not read");
            return qs(NULL);
        };
        let spot = self.with(None, |hub| hub.voice_panel_at(kb, bar, w, h));
        qs(&json(&spot))
    }

    /// `f` over the hub, or `none` before the app installed one.
    fn with<T>(&self, none: T, f: impl FnOnce(&Hub) -> T) -> T {
        self.rust().hub.as_deref().map_or(none, f)
    }
}
