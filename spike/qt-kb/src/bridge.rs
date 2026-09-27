//! The `Keyboard` QObject: gives QML the key boxes, sizes, status line and actions of spike-core.

use std::ffi::c_void;
use std::pin::Pin;
use std::sync::OnceLock;

use cxx_qt::CxxQtType;
use cxx_qt_lib::QString;
use spike_core::config::SpikeConfig;
use spike_core::layout::{self, Follow};
use spike_core::status::{self, Guard};
use spike_core::{inject, place, uiaccess, window};
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
        #[qproperty(f32, face_width, READ, CONSTANT)]
        #[qproperty(f32, face_height, READ, CONSTANT)]
        #[qproperty(f32, gap_px, READ, CONSTANT)]
        #[qproperty(f32, font_px, READ, CONSTANT)]
        #[qproperty(i32, guard_delay_ms, READ, CONSTANT)]
        #[qproperty(i32, relabel_ms, READ, CONSTANT)]
        #[qproperty(QString, title, READ, CONSTANT)]
        #[qproperty(QString, guard_pending, READ, CONSTANT)]
        type Keyboard = super::KeyboardRust;

        /// Rows as JSON, labelled for the app in front; each key carries its box.
        #[qinvokable]
        fn rows_json(self: Pin<&mut Self>) -> QString;

        /// Sends one key; returns a short note.
        #[qinvokable]
        fn tap(&self, code: i32) -> QString;

        /// Tries the guard once; empty means try again after the delay, else focus went back.
        #[qinvokable]
        fn guard(self: Pin<&mut Self>) -> QString;

        /// New rows JSON if the layout in front changed, else empty.
        #[qinvokable]
        fn relabel(self: Pin<&mut Self>) -> QString;

        /// The whole status line, ending with `note`.
        #[qinvokable]
        fn line(&self, note: &QString) -> QString;
    }
}

/// Face name used to pick the window title and name the error file.
pub const FACE: &str = "qt";

/// What `main` hands over before QML exists.
struct Start {
    cfg: SpikeConfig,
    /// The window in front at start-up, as a raw handle value.
    prev: usize,
}

static START: OnceLock<Start> = OnceLock::new();

/// Stores the settings and the window in front at start-up; call before loading QML.
pub fn init(cfg: SpikeConfig, prev: HWND) {
    let _ = START.set(Start {
        cfg,
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

/// Rust side of `Keyboard`.
pub struct KeyboardRust {
    face_width: f32,
    face_height: f32,
    gap_px: f32,
    font_px: f32,
    guard_delay_ms: i32,
    relabel_ms: i32,
    title: QString,
    guard_pending: QString,
    cfg: SpikeConfig,
    prev: HWND,
    /// Fixed part of the status line.
    base: String,
    /// Layout the labels show, so they change only when the app in front changes it.
    follow: Follow,
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
        let (face_width, face_height) = place::face_size(kb);
        let keys = kb.rows.iter().map(Vec::len).sum();
        Self {
            face_width,
            face_height,
            gap_px: kb.gap_px,
            font_px: kb.font_px,
            guard_delay_ms: ms(kb.guard_delay_ms),
            relabel_ms: ms(kb.relabel_ms),
            title: QString::from(&start.cfg.title(FACE)),
            guard_pending: QString::from(status::GUARD_PENDING),
            cfg: start.cfg.clone(),
            prev: HWND(start.prev as *mut c_void),
            base: status::base(uiaccess::active(), keys),
            follow: Follow::default(),
            tries: 0,
        }
    }
}

impl qobject::Keyboard {
    fn rows_json(mut self: Pin<&mut Self>) -> QString {
        let hkl = layout::foreground_layout();
        self.as_mut().rust_mut().follow = Follow::new(hkl);
        QString::from(&layout::rows_json(&self.rust().cfg.keyboard, hkl))
    }

    fn tap(&self, code: i32) -> QString {
        let sent = u32::try_from(code)
            .map_err(|e| e.to_string())
            .and_then(inject::tap);
        QString::from(&match sent {
            Ok(()) => format!("tap {code:#x}"),
            Err(err) => format!("tap {code:#x}: {err}"),
        })
    }

    fn guard(mut self: Pin<&mut Self>) -> QString {
        let attempt = self.rust().tries + 1;
        self.as_mut().rust_mut().tries = attempt;
        let max = self.rust().cfg.keyboard.guard_tries;
        match status::guard(window::guard_own_windows(), attempt, max) {
            Guard::Retry => QString::default(),
            Guard::Done(note) | Guard::GiveUp(note) => {
                window::give_back(self.rust().prev);
                QString::from(&note)
            }
        }
    }

    fn relabel(mut self: Pin<&mut Self>) -> QString {
        match self.as_mut().rust_mut().follow.changed() {
            Some(hkl) => QString::from(&layout::rows_json(&self.rust().cfg.keyboard, hkl)),
            None => QString::default(),
        }
    }

    fn line(&self, note: &QString) -> QString {
        QString::from(&status::with(&self.rust().base, &note.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ms_saturates_at_the_qml_int_limit() {
        assert_eq!(ms(300), 300);
        assert_eq!(ms(u64::MAX), i32::MAX);
    }
}
