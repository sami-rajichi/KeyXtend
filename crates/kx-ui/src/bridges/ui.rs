//! The `Ui` object: the look, the keyboard's and the tray's texts, the notices, the tray icon, where pop-ups go and the
//! app's commands. It follows the system look and hears the hub's wakes on the Qt thread, so make one per app, in the
//! root file.

mod spots;
mod texts;
mod tray;

use super::{hub, qs};
use crate::hub::{Hub, NULL, Wake, json};
use cxx_qt::{CxxQtType, Threading};
use cxx_qt_lib::QString;
use kx_platform::ui::{TrayEvent, WatchGuard};
use std::pin::Pin;
use std::sync::Arc;

/// The cxx-qt bridge that makes `Ui` a QML type.
#[cxx_qt::bridge]
pub mod qobject {
    // SAFETY: the headers and types are cxx-qt-lib's own QString and QImage bindings, so both sides match.
    unsafe extern "C++" {
        include!("cxx-qt-lib/qstring.h");
        /// Qt string.
        type QString = cxx_qt_lib::QString;
        include!("cxx-qt-lib/qimage.h");
        /// Qt picture.
        type QImage = cxx_qt_lib::QImage;
    }

    #[auto_cxx_name]
    extern "RustQt" {
        #[qobject]
        #[qml_element]
        type Ui = super::UiRust;

        /// The timings and paging, as JSON.
        #[qinvokable]
        fn settings_json(&self) -> QString;

        /// The keyboard's language, its size buttons' texts and the tray's language, as JSON.
        #[qinvokable]
        fn lang_json(&self) -> QString;

        /// The look to draw now, as JSON.
        #[qinvokable]
        fn look_json(&self) -> QString;

        /// The text of translation key `key`, in the keyboard's language.
        #[qinvokable]
        fn text(&self, key: &QString) -> QString;

        /// The text of translation key `key`, in the system's language, for the tray menu.
        #[qinvokable]
        fn tray_text(&self, key: &QString) -> QString;

        /// The name of `id` in group `group`, such as a theme or a top-bar button.
        #[qinvokable]
        fn name_of(&self, group: &QString, id: &QString) -> QString;

        /// The space bar's text from its parts, a JSON list.
        #[qinvokable]
        fn space(&self, parts: &QString) -> QString;

        /// How many more notices wait behind the one the banner shows; empty for none.
        #[qinvokable]
        fn more_text(&self, n: i32) -> QString;

        /// A pager's number: page `at` of `count`, both from 1.
        #[qinvokable]
        fn page_text(&self, at: i32, count: i32) -> QString;

        /// The tray picture's side in pixels.
        #[qinvokable]
        fn tray_px(&self) -> i32;

        /// Shows `picture`, scaled to `tray_px`, as the tray icon, or changes it; true when shown.
        #[qinvokable]
        fn tray_show(&self, picture: &QImage) -> bool;

        /// Removes the tray icon; call it before the app ends.
        #[qinvokable]
        fn tray_hide(&self);

        /// Where a `w` by `h` pop-up goes beside the keyboard box `over` (JSON, physical pixels, or null), its left
        /// edge at `left` unless `centred`; a JSON point in physical pixels, or null.
        #[qinvokable]
        fn pop_at(&self, over: &QString, left: f64, centred: bool, w: f64, h: f64) -> QString;

        /// Where a `w` by `h` tray menu goes for a click at (`x`, `y`); a JSON point in physical pixels, or null.
        #[qinvokable]
        fn menu_at(&self, x: i32, y: i32, w: f64, h: f64) -> QString;

        /// The notices waiting for the banner, as a JSON list of `{id, text, retry}`.
        #[qinvokable]
        fn take_notices(&self) -> QString;

        /// The notices the banner shows, in the keyboard's language now, as `take_notices` gives them.
        #[qinvokable]
        fn shown_notices(&self) -> QString;

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

        /// The compiled-in icon folder and type and the font files, as JSON `{icons, ext, fonts}`.
        #[qinvokable]
        fn assets(&self) -> QString;

        /// Asks the app to quit, which stops the modules and then sends `quit_asked`; a gone app ends the UI at once.
        /// False when Quit was asked already, the app is gone, or no hub is installed.
        #[qinvokable]
        fn quit(&self) -> bool;

        /// The settings or face changed: read everything again.
        #[qsignal]
        fn settings_changed(self: Pin<&mut Self>);

        /// The keyboard's language changed: read only its texts again.
        #[qsignal]
        fn texts_changed(self: Pin<&mut Self>);

        /// The look changed.
        #[qsignal]
        fn look_changed(self: Pin<&mut Self>);

        /// Other bundled fonts are loaded: pick the families again.
        #[qsignal]
        fn fonts_changed(self: Pin<&mut Self>);

        /// Notices wait for the banner.
        #[qsignal]
        fn notices_waiting(self: Pin<&mut Self>);

        /// A second copy asks to show the keyboard.
        #[qsignal]
        fn show_asked(self: Pin<&mut Self>);

        /// The app stopped the modules: end the UI.
        #[qsignal]
        fn quit_asked(self: Pin<&mut Self>);

        /// The tray icon was clicked; its menu opens from (`x`, `y`), in physical pixels.
        #[qsignal]
        fn tray_clicked(self: Pin<&mut Self>, x: i32, y: i32);

        /// Another window came to the front, so the tray menu closes.
        #[qsignal]
        fn tray_away(self: Pin<&mut Self>);
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

/// Tells QML what changed, on the Qt thread; a new theme's fonts load first, so QML picks its families from them.
fn woken(mut ui: Pin<&mut qobject::Ui>, wake: Wake) {
    if matches!(wake, Wake::Settings | Wake::Look) && ui.with(false, Hub::sync_fonts) {
        ui.as_mut().fonts_changed();
    }
    match wake {
        Wake::Settings => ui.settings_changed(),
        Wake::Texts => ui.texts_changed(),
        Wake::Look => ui.look_changed(),
        Wake::Notices => ui.notices_waiting(),
        Wake::Show => ui.show_asked(),
        Wake::End => ui.quit_asked(),
        Wake::Tray(TrayEvent::Click(at)) => ui.tray_clicked(at.x, at.y),
        Wake::Tray(TrayEvent::Away) => ui.tray_away(),
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

    fn take_notices(&self) -> QString {
        qs(&self.with(NULL.to_owned(), |h| json(&h.take_notices())))
    }

    fn shown_notices(&self) -> QString {
        qs(&self.with(NULL.to_owned(), |h| json(&h.shown_notices())))
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

    #[allow(clippy::unused_self, reason = "cxx-qt invokables take the object")]
    fn assets(&self) -> QString {
        qs(crate::ASSETS)
    }
}
