//! The `Keyboard` object: the keyboard window's board, state, clicks and held keys through the face, its guard
//! and shape, the pointer and the bubble's spot. Every call runs on the Qt thread, which runs the relabel and hold
//! timers too.

use super::{hub, qs};
use crate::geom::whole;
use crate::guard::Run;
use crate::hub::{Hub, NULL, Redraw, Resize, json};
use cxx_qt::CxxQtType;
use cxx_qt_lib::QString;
use kx_module_api::ports::{Corner, FaceAction, KeyId, Point, Rect};
use std::sync::Arc;

mod frame;
mod held;
mod window;

/// The cxx-qt bridge that makes `Keyboard` a QML type.
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
        type Keyboard = super::KeyboardRust;

        /// The board to draw, as the face's JSON; `null` while the keyboard is not running.
        #[qinvokable]
        fn board_json(&self) -> QString;

        /// The live state, as the face's JSON.
        #[qinvokable]
        fn state_json(&self) -> QString;

        /// A click on key `id`; `back` is the language key's back part. Every action answers `{board, state}`.
        #[qinvokable]
        fn tap(&self, id: &QString, back: bool) -> QString;

        /// Key `id` goes down: it types now, and one that repeats starts the hold.
        #[qinvokable]
        fn press(&self, id: &QString) -> QString;

        /// The hold timer's beat: the held key may type again.
        #[qinvokable]
        fn tick(&self) -> QString;

        /// Key `id` comes up, wherever the pointer is: its hold ends.
        #[qinvokable]
        fn let_go(&self, id: &QString) -> QString;

        /// Qt took the press away, or the window hid: the hold ends with nothing more typed.
        #[qinvokable]
        fn leave(&self) -> QString;

        /// One size step smaller.
        #[qinvokable]
        fn smaller(&self) -> QString;

        /// One size step bigger.
        #[qinvokable]
        fn bigger(&self) -> QString;

        /// Size preset `index`.
        #[qinvokable]
        fn preset(&self, index: i32) -> QString;

        /// A resize starts where the pointer is.
        #[qinvokable]
        fn resize_begin(&self) -> QString;

        /// The resize follows the pointer.
        #[qinvokable]
        fn resize_at(&self) -> QString;

        /// The resize ends.
        #[qinvokable]
        fn resize_end(&self) -> QString;

        /// The relabel timer asks whether the app in front switched layout.
        #[qinvokable]
        fn relabel(&self) -> QString;

        /// Language `index` of the open list was picked.
        #[qinvokable]
        fn lang_pick(&self, index: i32) -> QString;

        /// Key `id` goes to the open list.
        #[qinvokable]
        fn panel_key(&self, id: &QString) -> QString;

        /// The window sits at `x`, `y`, `w` by `h` physical pixels; its resize corner hangs `overhang` Qt units past the
        /// plate at size 1.
        #[qinvokable]
        fn placed(&self, x: f64, y: f64, w: f64, h: f64, overhang: f64) -> QString;

        /// The pointer in physical pixels as JSON `{x, y}`, or `null`.
        #[qinvokable]
        fn cursor(&self) -> QString;

        /// The bubble's spot in bottom `corner` of the screen holding physical point `x`, `y`: `{x, y}` or `null`.
        #[qinvokable]
        fn bubble_at(&self, x: f64, y: f64, corner: &QString) -> QString;

        /// Cuts window `title` to `boxes`, a JSON list of `{x, y, w, h}` at scale `dpr`; false until it shows.
        #[qinvokable]
        fn shape(&self, title: &QString, boxes: &QString, dpr: f64) -> bool;

        /// Lifts our shown windows titled `title` alone in front of our others, as a tip does; false until one shows.
        #[qinvokable]
        fn lift_alone(&self, title: &QString) -> bool;

        /// Draws window `title` at once, so a press shows before its key acts; false while it does not show.
        #[qinvokable]
        fn draw_now(&self, title: &QString) -> bool;

        /// The frame log's mark: a key's action starts.
        #[qinvokable]
        fn press_start(&self);

        /// The frame log's mark: a key's action and its redraw ended.
        #[qinvokable]
        fn press_end(&self);

        /// One guard try, with our window titled `top` (empty for none) landing in front; true to try again after
        /// `guard_delay_ms`.
        #[qinvokable]
        fn guard(self: Pin<&mut Self>, top: &QString) -> bool;

        /// Debug builds only: the churn plan as JSON while `KX_UI_CHURN` is set, else empty.
        #[cfg(debug_assertions)]
        #[qinvokable]
        fn churn_plan(&self) -> QString;

        /// Debug builds only: Caps Lock shows the other way round, with no key sent.
        #[cfg(debug_assertions)]
        #[qinvokable]
        fn flip_caps(&self) -> QString;

        /// Debug builds only: the churn ran to its end.
        #[cfg(debug_assertions)]
        #[qinvokable]
        fn churned(&self);
    }
}

/// Rust side of `Keyboard`.
pub struct KeyboardRust {
    hub: Option<Arc<Hub>>,
    /// The guard run's tries so far.
    guard: Run,
}

impl Default for KeyboardRust {
    fn default() -> Self {
        Self {
            hub: hub(),
            guard: Run::default(),
        }
    }
}

/// `index` from QML as a list place; a negative one is past every list.
fn place(index: i32) -> usize {
    usize::try_from(index).unwrap_or(usize::MAX)
}

impl qobject::Keyboard {
    /// `f` over the hub as JSON, or `none` before the app installed one.
    fn with(&self, none: &str, f: impl FnOnce(&Hub) -> String) -> QString {
        qs(&self
            .rust()
            .hub
            .as_deref()
            .map_or_else(|| none.to_owned(), f))
    }

    /// Does `action` and answers what to redraw.
    fn run(&self, action: FaceAction) -> QString {
        self.redraw(|h| h.act(action))
    }

    /// Answers what to redraw after `f`.
    fn redraw(&self, f: impl FnOnce(&Hub) -> Redraw) -> QString {
        self.with(&Redraw::default().to_json(), |h| f(h).to_json())
    }

    fn board_json(&self) -> QString {
        self.with(NULL, Hub::board_json)
    }

    fn state_json(&self) -> QString {
        self.with(NULL, Hub::state_json)
    }

    fn tap(&self, id: &QString, back: bool) -> QString {
        let id = KeyId::new(id.to_string());
        self.sent(FaceAction::Tap { id, back })
    }

    fn smaller(&self) -> QString {
        self.run(FaceAction::Smaller)
    }

    fn bigger(&self) -> QString {
        self.run(FaceAction::Bigger)
    }

    fn preset(&self, index: i32) -> QString {
        self.run(FaceAction::Preset {
            index: place(index),
        })
    }

    fn resize_begin(&self) -> QString {
        self.redraw(|h| h.resize(Resize::Begin))
    }

    fn resize_at(&self) -> QString {
        self.redraw(|h| h.resize(Resize::At))
    }

    fn resize_end(&self) -> QString {
        self.redraw(|h| h.resize(Resize::End))
    }

    fn relabel(&self) -> QString {
        self.run(FaceAction::Relabel)
    }

    fn lang_pick(&self, index: i32) -> QString {
        self.run(FaceAction::LangPick {
            index: place(index),
        })
    }

    fn panel_key(&self, id: &QString) -> QString {
        let id = id.to_string();
        self.redraw(|h| h.panel_key(&id))
    }

    fn placed(&self, x: f64, y: f64, w: f64, h: f64, overhang: f64) -> QString {
        let rect = Rect {
            x: whole(x),
            y: whole(y),
            w: whole(w),
            h: whole(h),
        };
        self.redraw(|hub| hub.placed(rect, overhang))
    }

    fn cursor(&self) -> QString {
        self.with(NULL, |h| json(&h.cursor()))
    }

    fn bubble_at(&self, x: f64, y: f64, corner: &QString) -> QString {
        let at = Point {
            x: whole(x),
            y: whole(y),
        };
        let corner = serde_json::Value::String(corner.to_string());
        let Ok(corner) = serde_json::from_value::<Corner>(corner) else {
            tracing::warn!("QML named no bottom corner");
            return qs(NULL);
        };
        self.with(NULL, |h| json(&h.bubble_at(at, corner)))
    }

    #[cfg(debug_assertions)]
    fn churn_plan(&self) -> QString {
        let value = std::env::var_os(crate::churn::CHURN_VAR);
        let themes = self.rust().hub.as_deref().map_or(0, Hub::theme_count);
        qs(&crate::churn::plan(value.as_deref(), themes))
    }

    #[cfg(debug_assertions)]
    fn flip_caps(&self) -> QString {
        self.run(FaceAction::FlipCaps)
    }

    #[cfg(debug_assertions)]
    #[allow(clippy::unused_self, reason = "cxx-qt invokables take the object")]
    fn churned(&self) {
        crate::churn::ended();
    }
}
