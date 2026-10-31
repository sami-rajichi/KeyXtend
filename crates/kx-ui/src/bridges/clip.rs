//! The clipboard panel's picture provider on the app's engine (`cpp/clips.h`): `image://clip/<round>/<id>` draws a
//! row's thumbnail and `image://clip/<round>/preview` the preview image, copied from the hub's kept view of that round
//! only. Debug builds can read what it serves, for tests.

use super::installed;
use crate::hub::PICTURES;
use cxx_qt_lib::QQmlApplicationEngine;
use std::pin::Pin;

#[cxx_qt::bridge]
mod ffi {
    // SAFETY: `clips.h` declares exactly these functions with these types; Qt calls the provider on its GUI or image
    // thread, the engine owns the provider it is given, and `fetch` is a plain Rust function, safe on any thread.
    unsafe extern "C++" {
        include!("kx-ui/cpp/clips.h");
        include!("cxx-qt-lib/qqmlapplicationengine.h");
        /// Qt's QML engine for an app.
        type QQmlApplicationEngine = cxx_qt_lib::QQmlApplicationEngine;

        /// Registers the provider on `engine` as `name`; `fetch` copies a picture's bytes and sets its size.
        #[namespace = "kxui"]
        fn serve_clips(
            engine: Pin<&mut QQmlApplicationEngine>,
            name: &str,
            fetch: fn(&str, &mut u32, &mut u32) -> Vec<u8>,
        );

        /// What a provider over `fetch` serves for `id`, as RGBA8 bytes row by row, with its width.
        #[cfg(debug_assertions)]
        #[namespace = "kxui"]
        fn clip_served(
            id: &str,
            fetch: fn(&str, &mut u32, &mut u32) -> Vec<u8>,
            width: &mut u32,
        ) -> Vec<u8>;
    }
}

/// Copies picture `id` of the installed hub's kept view and sets its size; nothing, at size 0, when there is none.
fn fetch(id: &str, width: &mut u32, height: &mut u32) -> Vec<u8> {
    let Some(pixels) = installed().and_then(|hub| hub.clip_picture(id)) else {
        return Vec::new();
    };
    *width = pixels.width();
    *height = pixels.height();
    pixels.rgba().expose().to_vec()
}

/// Registers the clipboard panel's picture provider on `engine`; call it on the Qt GUI thread before the engine loads
/// QML.
pub fn serve_clips(engine: Pin<&mut QQmlApplicationEngine>) {
    ffi::serve_clips(engine, PICTURES, fetch);
}

/// What the provider serves for `id`, `<round>/<row id>` or `<round>/preview`, as RGBA8 bytes row by row, with its
/// width; one clear pixel when the kept view has no such picture. Debug builds only, for tests.
#[cfg(debug_assertions)]
#[must_use]
pub fn clip_pixels(id: &str) -> (u32, Vec<u8>) {
    let mut width = 0;
    let rgba = ffi::clip_served(id, fetch, &mut width);
    (width, rgba)
}
