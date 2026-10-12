//! The keyboard's own frames on the Qt GUI thread: a press frame drawn at once (`press_frame = "now"`), and the frame
//! log (`frame_log`), which logs one line per press and per frame, with times only.

use crate::bridges::harden::drawing_pinned;
use crate::frames::{Frame, FrameTimes, Log, Mark, PressTimes};
use cxx::UniquePtr;
use cxx_qt_lib::QString;
use std::sync::{Mutex, PoisonError};

#[cxx_qt::bridge]
mod ffi {
    // SAFETY: `frames.h` and `framelog.h` declare exactly these items with these types; QString is cxx-qt-lib's own
    // binding. QML and the app call them on the Qt GUI thread, outside a frame, and the watch calls back on that
    // thread, so its plain Rust functions run where Qt draws.
    unsafe extern "C++" {
        include!("kx-ui/cpp/frames.h");
        include!("kx-ui/cpp/framelog.h");
        include!("cxx-qt-lib/qstring.h");
        /// Qt string.
        type QString = cxx_qt_lib::QString;

        /// Draws our shown Quick window titled `title` now, as its next update would; false when none shows.
        #[namespace = "kxui"]
        fn draw_now(title: &QString) -> bool;

        /// Times each press and frame of our Quick windows while it lives.
        #[namespace = "kxui"]
        type FrameWatch;

        /// Starts the watch, which calls `pressed` and `framed` as presses and frames come.
        #[namespace = "kxui"]
        fn frame_watch_new(
            pressed: fn(window: usize, msg: i64, qt: i64),
            framed: fn(
                window: usize,
                update: i64,
                begin: i64,
                sync: i64,
                rendered: i64,
                swap: i64,
                done: i64,
            ),
        ) -> UniquePtr<FrameWatch>;

        /// Now in µs, on the clock of the watch.
        #[namespace = "kxui"]
        fn now_us() -> i64;
    }
}

/// The frame log while one runs.
static LOG: Mutex<Option<Log>> = Mutex::new(None);

/// Draws our shown Quick window titled `title` now, as its next update would; false when none shows.
pub(crate) fn draw_now(title: &QString) -> bool {
    ffi::draw_now(title)
}

/// `draw_now` by a Rust title; debug builds only, for tests.
#[cfg(debug_assertions)]
#[must_use]
pub fn draw_window(title: &str) -> bool {
    draw_now(&QString::from(title))
}

/// The frame log: while it lives, each press and each frame of our windows logs one line of times (`[ui] frame_log`).
pub struct FrameLog {
    _watch: UniquePtr<ffi::FrameWatch>,
}

impl FrameLog {
    /// Starts the log once Qt's app object exists, on its GUI thread; `None` unless drawing is pinned there, as the watch
    /// keeps its times on that thread only.
    #[must_use]
    pub fn start() -> Option<Self> {
        if !drawing_pinned() {
            return None;
        }
        *LOG.lock().unwrap_or_else(PoisonError::into_inner) = Some(Log::new());
        Some(Self {
            _watch: ffi::frame_watch_new(pressed, framed),
        })
    }
}

impl Drop for FrameLog {
    fn drop(&mut self) {
        *LOG.lock().unwrap_or_else(PoisonError::into_inner) = None;
    }
}

/// Marks `mark` now in the open press, while the log runs.
pub(crate) fn mark(mark: Mark) {
    if let Some(line) = with(|log| log.mark(mark, ffi::now_us())).flatten() {
        press_line(&line);
    }
}

/// `f` over the running log, or `None` while none runs.
fn with<T>(f: impl FnOnce(&mut Log) -> T) -> Option<T> {
    LOG.lock()
        .unwrap_or_else(PoisonError::into_inner)
        .as_mut()
        .map(f)
}

/// The watch heard a press: the last press's line comes out if it had not.
fn pressed(window: usize, msg: i64, qt: i64) {
    if let Some(line) = with(|log| log.pressed(window, msg, qt)).flatten() {
        press_line(&line);
    }
}

/// The watch heard a frame end: its line, and its press's once whole.
fn framed(window: usize, update: i64, begin: i64, sync: i64, rendered: i64, swap: i64, done: i64) {
    let f = Frame {
        update,
        begin,
        sync,
        rendered,
        swap,
        done,
    };
    let Some((frame, press)) = with(|log| log.framed(window, f)) else {
        return;
    };
    frame_line(&frame);
    if let Some(line) = press {
        press_line(&line);
    }
}

/// Logs frame `f`'s line.
fn frame_line(f: &FrameTimes) {
    tracing::info!(
        at_us = f.at_us,
        polish_us = f.polish_us,
        wait_us = f.wait_us,
        render_us = f.render_us,
        present_us = f.present_us,
        busy_us = f.busy_us,
        "frame timed"
    );
}

/// Logs press `p`'s line.
fn press_line(p: &PressTimes) {
    tracing::info!(
        at_us = p.at_us,
        qt_us = p.qt_us,
        start_us = p.start_us,
        sent_us = p.sent_us,
        end_us = p.end_us,
        update_us = p.update_us,
        begin_us = p.begin_us,
        sync_us = p.sync_us,
        rendered_us = p.rendered_us,
        swap_us = p.swap_us,
        done_us = p.done_us,
        "press timed"
    );
}
