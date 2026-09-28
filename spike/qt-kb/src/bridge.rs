//! The `Keyboard` QObject: the window title, the focus guard and the status line; also holds what `main` loaded.

use std::ffi::c_void;
use std::pin::Pin;
use std::sync::OnceLock;

use cxx_qt::CxxQtType;
use cxx_qt_lib::QString;
use spike_core::config::SpikeConfig;
use spike_core::facecfg::Corner;
use spike_core::hold::Pt;
use spike_core::place::Place;
use spike_core::status::{self, Guard};
use spike_core::theme::Themes;
use spike_core::{kbgeom, screen, uiaccess, window};
use windows::Win32::Foundation::HWND;

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
        #[qproperty(f32, gap_px, READ, CONSTANT)]
        #[qproperty(f32, font_px, READ, CONSTANT)]
        #[qproperty(i32, guard_delay_ms, READ, CONSTANT)]
        #[qproperty(i32, relabel_ms, READ, CONSTANT)]
        #[qproperty(QString, title, READ, CONSTANT)]
        #[qproperty(QString, guard_pending, READ, CONSTANT)]
        type Keyboard = super::KeyboardRust;

        /// Tries the guard once; empty means try again after the delay, else focus went back.
        #[qinvokable]
        fn guard(self: Pin<&mut Self>) -> QString;

        /// The whole status line, ending with `note`.
        #[qinvokable]
        fn line(&self, note: &QString) -> QString;

        /// Cuts the window down to `boxes` (a JSON list of places, in Qt units) at scale `dpr`; returns a note or empty.
        #[qinvokable]
        fn shape(&self, boxes: &QString, dpr: f64) -> QString;

        /// Where a `w` by `h` window starts, as JSON `{x, y}` in physical pixels, or `{note}`.
        #[qinvokable]
        fn start_at(&self, w: f64, h: f64) -> QString;

        /// Where the bubble goes on the keyboard's screen, as JSON `{x, y}` in physical pixels, or `{note}`.
        #[qinvokable]
        fn bubble_at(&self) -> QString;
    }
}

/// Face name used to pick the window title and name the error file.
pub const FACE: &str = "qt";
/// Note when the window could not be cut because it is not on screen yet.
const NOT_SHOWN: &str = "the keyboard is not on screen yet, so it was not cut";

/// What `main` hands over before QML exists.
struct Start {
    cfg: SpikeConfig,
    themes: Themes,
    /// The window in front at start-up, as a raw handle value.
    prev: usize,
}

static START: OnceLock<Start> = OnceLock::new();

/// Stores the settings, the themes and the window in front at start-up; call before loading QML.
pub fn init(cfg: SpikeConfig, themes: Themes, prev: HWND) {
    let _ = START.set(Start {
        cfg,
        themes,
        prev: prev.0 as usize,
    });
}

/// What `init` stored; `main` always calls it before QML makes any object.
fn start() -> &'static Start {
    START.get().expect("bridge::init runs before QML loads")
}

/// The settings `init` stored; QML objects read them when they are made.
pub fn config() -> &'static SpikeConfig {
    &start().cfg
}

/// The themes `init` stored.
pub fn themes() -> &'static Themes {
    &start().themes
}

/// Rust side of `Keyboard`.
pub struct KeyboardRust {
    gap_px: f32,
    font_px: f32,
    guard_delay_ms: i32,
    relabel_ms: i32,
    title: QString,
    guard_pending: QString,
    prev: HWND,
    /// Guard tries allowed.
    max_tries: u32,
    /// Fixed part of the status line.
    base: String,
    /// Guard tries so far.
    tries: u32,
}

/// Milliseconds as a QML `int`.
pub(crate) fn ms(value: u64) -> i32 {
    i32::try_from(value).unwrap_or(i32::MAX)
}

impl Default for KeyboardRust {
    fn default() -> Self {
        let start = start();
        let kb = &start.cfg.keyboard;
        let keys = kbgeom::board(&start.cfg.layout, 0.0, 1.0).keys.len();
        Self {
            gap_px: kb.gap_px,
            font_px: kb.font_px,
            guard_delay_ms: ms(kb.guard_delay_ms),
            relabel_ms: ms(kb.relabel_ms),
            title: QString::from(&start.cfg.title(FACE)),
            guard_pending: QString::from(status::GUARD_PENDING),
            prev: HWND(start.prev as *mut c_void),
            max_tries: kb.guard_tries,
            base: status::base(uiaccess::active(), keys),
            tries: 0,
        }
    }
}

impl qobject::Keyboard {
    fn guard(mut self: Pin<&mut Self>) -> QString {
        let attempt = self.rust().tries + 1;
        self.as_mut().rust_mut().tries = attempt;
        match status::guard(window::guard_own_windows(), attempt, self.rust().max_tries) {
            Guard::Retry => QString::default(),
            Guard::Done(note) | Guard::GiveUp(note) => {
                window::give_back(self.rust().prev);
                QString::from(&note)
            }
        }
    }

    fn line(&self, note: &QString) -> QString {
        QString::from(&status::with(&self.rust().base, &note.to_string()))
    }

    fn shape(&self, boxes: &QString, dpr: f64) -> QString {
        let rects = serde_json::from_str::<Vec<Place>>(&boxes.to_string())
            .map(|all| {
                all.iter()
                    .map(|p| p.outward(dpr as f32))
                    .collect::<Vec<_>>()
            })
            .map_err(|e| e.to_string());
        let done = rects.and_then(|r| window::shape(&self.rust().title.to_string(), &r));
        QString::from(match done {
            Ok(true) => String::new(),
            Ok(false) => NOT_SHOWN.into(),
            Err(e) => e,
        })
    }

    fn start_at(&self, w: f64, h: f64) -> QString {
        spot(screen::start_spot(w as f32, h as f32))
    }

    fn bubble_at(&self) -> QString {
        let [d, _, inset] = themes().shape.extra.bubble_px;
        let left = config().bar.bubble_corner == Corner::Left;
        let title = self.rust().title.to_string();
        spot(screen::bubble_for(&title, left, [d, inset]))
    }
}

/// A spot as JSON for QML: `{x, y}`, or `{note}` when Windows could not say.
fn spot(r: Result<Pt, String>) -> QString {
    let v = match r {
        Ok(p) => serde_json::json!({ "x": p.x, "y": p.y }),
        Err(e) => serde_json::json!({ "note": e }),
    };
    QString::from(&v.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_spot_reaches_qml_as_a_point_or_a_note() {
        let at = spot(Ok(Pt { x: 3, y: -4 })).to_string();
        assert_eq!(at, r#"{"x":3,"y":-4}"#);
        assert_eq!(spot(Err("gone".into())).to_string(), r#"{"note":"gone"}"#);
    }

    #[test]
    fn ms_saturates_at_the_qml_int_limit() {
        assert_eq!(ms(300), 300);
        assert_eq!(ms(u64::MAX), i32::MAX);
    }
}
