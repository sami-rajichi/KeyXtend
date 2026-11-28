//! The `Snip` object: Snip's windows as JSON, their clicks, presses and pen paths, the theme's fit, the editor's
//! flattened picture and place, the tray's Snip row, each screen's work area, the capture windows' shields and the
//! frozen screens' place under our other windows. It takes the hub's Snip wakes as it starts and tells QML on the Qt
//! thread, so make one, at the app's root.

use super::frames::draw_now;
use super::snips::{made, shot_of, shown};
use super::{hub, qs};
use crate::hub::{Hub, NULL, SnipCounted, SnipOpened, SnipWake, json};
use cxx_qt::{CxxQtType, Threading};
use cxx_qt_lib::{QImage, QString};
use kx_module_api::Zeroizing;
use kx_module_api::ports::Rect;
use std::pin::Pin;
use std::sync::Arc;

/// The cxx-qt bridge that makes `Snip` a QML type.
#[cxx_qt::bridge]
pub mod qobject {
    // SAFETY: the headers and types are cxx-qt-lib's own QString and QImage bindings, so both sides match.
    unsafe extern "C++" {
        include!("cxx-qt-lib/qstring.h");
        include!("cxx-qt-lib/qimage.h");
        /// Qt string.
        type QString = cxx_qt_lib::QString;
        /// Qt's picture in memory.
        type QImage = cxx_qt_lib::QImage;
    }

    #[auto_cxx_name]
    extern "RustQt" {
        #[qobject]
        #[qml_element]
        type Snip = super::SnipRust;

        /// Snip's windows as JSON, or null before any view.
        #[qinvokable]
        fn view(&self) -> QString;

        /// A key on view `round`: `kind` and `id` name it, and `arg` a step's sign or a caret.
        #[qinvokable]
        fn act(&self, kind: &QString, id: &QString, arg: i32, round: i64);

        /// A press at (`x`, `y`) logical pixels on frozen screen `screen`, the view's place in `frozen`.
        #[qinvokable]
        fn spot(&self, screen: i32, x: f64, y: f64, round: i64);

        /// A press at (`x`, `y`) window pixels on the editor's picture, shown at `scale` window pixels a picture pixel.
        #[qinvokable]
        fn press(&self, x: f64, y: f64, scale: f64, round: i64);

        /// A pen's path since its first click, JSON `[[x, y], …]` in window pixels, on the picture shown at `scale`.
        #[qinvokable]
        fn stroke(&self, points: &QString, scale: f64, round: i64);

        /// The theme's sizes as JSON, whenever an editor opens; false when they do not read.
        #[qinvokable]
        fn fit(&self, sizes: &QString) -> bool;

        /// The editor's picture flattened at its own pixels, for `keep`, `copy` or `save`.
        #[qinvokable]
        fn flat(&self, keep: &QString, picture: &QImage, round: i64);

        /// The tray's Snip row: a hidden keyboard shows first, then the Snip panel opens.
        #[qinvokable]
        fn open(&self);

        /// A screen came, went or changed.
        #[qinvokable]
        fn screens(&self);

        /// Makes capture window `window`, titled `title`, and keeps it out of every capture; call it before it first
        /// shows. False when refused, so it stays hidden while a countdown or recording runs.
        #[qinvokable]
        fn shield(&self, window: &QString, title: &QString) -> bool;

        /// Makes our windows titled `title` that Qt has not made yet, each after the window that owns it, hidden; call
        /// it before they first show, so each show finds them made.
        #[qinvokable]
        fn make(&self, title: &QString);

        /// Stage `stage`'s window opened: the ms from the click before its view (negative for none), then from its
        /// view to its show, its first frame and the end of its pop. Logged at debug, numbers only.
        #[qinvokable]
        fn opened(
            &self,
            stage: &QString,
            click_ms: i32,
            shown_ms: i32,
            drawn_ms: i32,
            ready_ms: i32,
        );

        /// A countdown second showed, `left_s` 0 once it hid: its `draws` since it showed and whether Cancel is
        /// `held`, with the frozen screens Qt still shows by their title `screens`. Logged at debug, numbers only.
        #[qinvokable]
        fn counted(&self, left_s: i32, draws: i32, held: bool, screens: &QString);

        /// Draws our shown window titled `title` now, as its next update would; false while none shows or it draws.
        #[qinvokable]
        fn draw(&self, title: &QString) -> bool;

        /// Snip's windows changed or a frozen screen moved: the frozen screens, titled `title`, go under our other
        /// windows.
        #[qinvokable]
        fn order(&self, title: &QString);

        /// Where the editor opens, JSON `{place, back}` of `{x, y, w, h}` in physical pixels, `back` being the box a
        /// maximised one comes back to (null when it opens unmaximised); null with no screen.
        #[qinvokable]
        fn editor_at(&self) -> QString;

        /// The editor moved or resized to `x`, `y`, `w` by `h` physical pixels: where it stays, as JSON, or null.
        #[qinvokable]
        fn editor_fit(&self, x: i32, y: i32, w: i32, h: i32) -> QString;

        /// The editor ended a move, a resize or a restore there: where it stays, which the module keeps for next time.
        #[qinvokable]
        fn editor_placed(&self, x: i32, y: i32, w: i32, h: i32) -> QString;

        /// The editor maximised from there: its screen's work area, which the module keeps over that box for next time.
        #[qinvokable]
        fn editor_full(&self, x: i32, y: i32, w: i32, h: i32) -> QString;

        /// A new view or a refused shield: read `view`.
        #[qsignal]
        fn view_changed(self: Pin<&mut Self>);
    }

    impl cxx_qt::Threading for Snip {}
    impl cxx_qt::Initialize for Snip {}
}

/// Rust side of `Snip`.
pub struct SnipRust {
    hub: Option<Arc<Hub>>,
}

impl Default for SnipRust {
    fn default() -> Self {
        Self { hub: hub() }
    }
}

impl cxx_qt::Initialize for qobject::Snip {
    fn initialize(self: Pin<&mut Self>) {
        let Some(hub) = self.rust().hub.clone() else {
            return;
        };
        let qt = self.qt_thread();
        hub.set_snip_waker(Box::new(move |SnipWake::View| {
            // A queue refused after the object is gone has nobody left to tell.
            let _ = qt.queue(|snip: Pin<&mut qobject::Snip>| {
                snip.with((), Hub::snip_woken);
                snip.view_changed();
            });
        }));
    }
}

impl qobject::Snip {
    fn view(&self) -> QString {
        qs(&self.with(Zeroizing::new(NULL.to_owned()), Hub::snip_json))
    }

    fn act(&self, kind: &QString, id: &QString, arg: i32, round: i64) {
        let (kind, id) = (kind.to_string(), id.to_string());
        self.with((), |h| h.snip_act(&kind, &id, arg, round));
    }

    fn spot(&self, screen: i32, x: f64, y: f64, round: i64) {
        self.with((), |h| h.snip_spot(screen, x, y, round));
    }

    fn press(&self, x: f64, y: f64, scale: f64, round: i64) {
        self.with((), |h| h.snip_press(x, y, scale, round));
    }

    fn stroke(&self, points: &QString, scale: f64, round: i64) {
        let Ok(points) = serde_json::from_str::<Vec<[f64; 2]>>(&points.to_string()) else {
            tracing::warn!("QML sent a pen path that does not read");
            return;
        };
        self.with((), |h| h.snip_stroke(&points, scale, round));
    }

    fn fit(&self, sizes: &QString) -> bool {
        let sizes = sizes.to_string();
        self.with(false, |h| h.snip_fit(&sizes))
    }

    fn flat(&self, keep: &QString, picture: &QImage, round: i64) {
        let Some(shot) = shot_of(picture) else {
            tracing::warn!("QML sent an empty flattened picture");
            return;
        };
        let keep = keep.to_string();
        self.with((), |h| h.snip_flat(&keep, shot, round));
    }

    fn open(&self) {
        self.with((), Hub::snip_open);
    }

    fn screens(&self) {
        self.with((), Hub::snip_screens);
    }

    fn shield(&self, window: &QString, title: &QString) -> bool {
        let (window, title) = (window.to_string(), title.to_string());
        made(&title);
        self.with(false, |h| h.snip_shield(&window, &title))
    }

    #[allow(clippy::unused_self, reason = "cxx-qt invokables take the object")]
    fn make(&self, title: &QString) {
        made(&title.to_string());
    }

    fn opened(&self, stage: &QString, click_ms: i32, shown_ms: i32, drawn_ms: i32, ready_ms: i32) {
        let stage = stage.to_string();
        let times = SnipOpened {
            click_ms,
            shown_ms,
            drawn_ms,
            ready_ms,
        };
        self.with((), |h| h.snip_opened(&stage, times));
    }

    fn counted(&self, left_s: i32, draws: i32, held: bool, screens: &QString) {
        let frozen = shown(&screens.to_string());
        let second = SnipCounted {
            left_s,
            draws,
            held,
            frozen,
        };
        self.with((), |h| h.snip_counted(second));
    }

    #[allow(clippy::unused_self, reason = "cxx-qt invokables take the object")]
    fn draw(&self, title: &QString) -> bool {
        draw_now(title)
    }

    fn order(&self, title: &QString) {
        let title = title.to_string();
        self.with((), |h| h.snip_order(&title));
    }

    fn editor_at(&self) -> QString {
        qs(&json(&self.with(None, Hub::editor_at)))
    }

    fn editor_fit(&self, x: i32, y: i32, w: i32, h: i32) -> QString {
        let rect = Rect { x, y, w, h };
        qs(&json(&self.with(None, |hub| hub.editor_fit(rect))))
    }

    fn editor_placed(&self, x: i32, y: i32, w: i32, h: i32) -> QString {
        let rect = Rect { x, y, w, h };
        qs(&json(&self.with(None, |hub| hub.editor_placed(rect))))
    }

    fn editor_full(&self, x: i32, y: i32, w: i32, h: i32) -> QString {
        let rect = Rect { x, y, w, h };
        qs(&json(&self.with(None, |hub| hub.editor_full(rect))))
    }

    /// `f` over the hub, or `none` before the app installed one.
    fn with<T>(&self, none: T, f: impl FnOnce(&Hub) -> T) -> T {
        self.rust().hub.as_deref().map_or(none, f)
    }
}
