//! The `Ui` object: the look, the texts, the notices and the app's commands. It follows the system look and hears
//! the hub's wakes on the Qt thread, so make one per app, in the root file.

use super::{hub, qs};
use crate::hub::{Hub, NULL, Wake, json};
use cxx_qt::{CxxQtType, Threading};
use cxx_qt_lib::QString;
use kx_platform::ui::WatchGuard;
use std::pin::Pin;
use std::sync::Arc;

/// The cxx-qt bridge that makes `Ui` a QML type.
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
        type Ui = super::UiRust;

        /// The timings, paging and text direction, as JSON.
        #[qinvokable]
        fn settings_json(&self) -> QString;

        /// The look to draw now, as JSON.
        #[qinvokable]
        fn look_json(&self) -> QString;

        /// The text of translation key `key`.
        #[qinvokable]
        fn text(&self, key: &QString) -> QString;

        /// The name of `id` in group `group`, such as a theme or a top-bar button.
        #[qinvokable]
        fn name_of(&self, group: &QString, id: &QString) -> QString;

        /// The space bar's text from its parts, a JSON list.
        #[qinvokable]
        fn space(&self, parts: &QString) -> QString;

        /// The notices waiting for the banner, as a JSON list of `{id, text, retry}`.
        #[qinvokable]
        fn take_notices(&self) -> QString;

        /// Try again on notice `id`; true when the app was asked.
        #[qinvokable]
        fn retry(&self, id: i64) -> bool;

        /// OK on notice `id`.
        #[qinvokable]
        fn dismiss(&self, id: i64);

        /// Shows the next theme and saves it; true when it changed.
        #[qinvokable]
        fn step_theme(&self) -> bool;

        /// Shows the other of light and dark and saves it; true when it changed.
        #[qinvokable]
        fn flip_mode(&self) -> bool;

        /// Puts the chosen backdrop behind window `title`; true when the plate may be see-through.
        #[qinvokable]
        fn backdrop(&self, title: &QString) -> bool;

        /// Asks the app to quit, which stops the modules and then sends `quit_asked`; false when asked already or gone.
        #[qinvokable]
        fn quit(&self) -> bool;

        /// The settings, texts or face changed: read everything again.
        #[qsignal]
        fn settings_changed(self: Pin<&mut Self>);

        /// The look changed.
        #[qsignal]
        fn look_changed(self: Pin<&mut Self>);

        /// Notices wait for the banner.
        #[qsignal]
        fn notices_waiting(self: Pin<&mut Self>);

        /// A second copy asks to show the keyboard.
        #[qsignal]
        fn show_asked(self: Pin<&mut Self>);

        /// The app stopped the modules: end the UI.
        #[qsignal]
        fn quit_asked(self: Pin<&mut Self>);
    }

    impl cxx_qt::Threading for Ui {}
    impl cxx_qt::Initialize for Ui {}
}

/// Rust side of `Ui`.
pub struct UiRust {
    hub: Option<Arc<Hub>>,
    /// Keeps the system look watch running while the object lives.
    watch: Option<WatchGuard>,
}

impl Default for UiRust {
    fn default() -> Self {
        Self {
            hub: hub(),
            watch: None,
        }
    }
}

impl cxx_qt::Initialize for qobject::Ui {
    fn initialize(mut self: Pin<&mut Self>) {
        let Some(hub) = self.rust().hub.clone() else {
            return;
        };
        let qt = self.qt_thread();
        hub.set_waker(Box::new(move |wake| {
            // A queue refused after the object is gone has nobody left to tell.
            let _ = qt.queue(move |ui| woken(ui, wake));
        }));
        match hub.watch() {
            Ok(watch) => self.as_mut().rust_mut().watch = Some(watch),
            Err(e) => tracing::warn!(error = %e, "the system look will not be followed"),
        }
    }
}

/// Tells QML what changed, on the Qt thread.
fn woken(ui: Pin<&mut qobject::Ui>, wake: Wake) {
    match wake {
        Wake::Settings => ui.settings_changed(),
        Wake::Look => ui.look_changed(),
        Wake::Notices => ui.notices_waiting(),
        Wake::Show => ui.show_asked(),
        Wake::End => ui.quit_asked(),
    }
}

impl qobject::Ui {
    /// `f` over the hub, or `none` before the app installed one.
    fn with<T>(&self, none: T, f: impl FnOnce(&Hub) -> T) -> T {
        self.rust().hub.as_deref().map_or(none, f)
    }

    fn settings_json(&self) -> QString {
        qs(&self.with(NULL.to_owned(), Hub::settings_json))
    }

    fn look_json(&self) -> QString {
        qs(&self.with(NULL.to_owned(), Hub::look_json))
    }

    fn text(&self, key: &QString) -> QString {
        let key = key.to_string();
        qs(&self.with(key.clone(), |h| h.i18n().text(&key)))
    }

    fn name_of(&self, group: &QString, id: &QString) -> QString {
        let (group, id) = (group.to_string(), id.to_string());
        qs(&self.with(id.clone(), |h| h.i18n().name_of(&group, &id)))
    }

    fn space(&self, parts: &QString) -> QString {
        let Ok(parts) = serde_json::from_str::<Vec<String>>(&parts.to_string()) else {
            tracing::warn!("QML sent space bar parts that do not read");
            return QString::default();
        };
        qs(&self.with(String::new(), |h| h.i18n().space(&parts)))
    }

    fn take_notices(&self) -> QString {
        qs(&self.with(NULL.to_owned(), |h| json(&h.take_notices())))
    }

    fn retry(&self, id: i64) -> bool {
        let id = u64::try_from(id).unwrap_or_default();
        self.with(false, |h| h.retry(id))
    }

    fn dismiss(&self, id: i64) {
        let id = u64::try_from(id).unwrap_or_default();
        self.with((), |h| h.dismiss(id));
    }

    fn step_theme(&self) -> bool {
        self.with(false, Hub::step_theme)
    }

    fn flip_mode(&self) -> bool {
        self.with(false, Hub::flip_mode)
    }

    fn backdrop(&self, title: &QString) -> bool {
        let title = title.to_string();
        self.with(false, |h| h.backdrop(&title))
    }

    fn quit(&self) -> bool {
        self.with(false, Hub::quit)
    }
}
