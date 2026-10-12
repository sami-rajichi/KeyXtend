//! The keyboard's own frames: a press frame drawn at once (`press_frame = "now"`), on the Qt GUI thread.

use cxx_qt_lib::QString;

#[cxx_qt::bridge]
mod ffi {
    // SAFETY: `frames.h` declares exactly this function with these types; QString is cxx-qt-lib's own binding. QML
    // calls it on the Qt GUI thread, outside a frame, so the frame it draws runs where Qt's basic loop draws them all.
    unsafe extern "C++" {
        include!("kx-ui/cpp/frames.h");
        include!("cxx-qt-lib/qstring.h");
        /// Qt string.
        type QString = cxx_qt_lib::QString;

        /// Draws our shown Quick window titled `title` now, as its next update would; false when none shows.
        #[namespace = "kxui"]
        fn draw_now(title: &QString) -> bool;
    }
}

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
