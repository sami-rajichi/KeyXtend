//! The icon provider on the app's engine: Qt draws each bundled icon from the compiled-in resources in its tint, at the
//! size it asks, so no icon needs a shader effect. Debug builds can read what it serves, for tests.

use super::FILES;
use cxx_qt_lib::QQmlApplicationEngine;
use kx_theme::assets::{ICON_EXT, ICON_PROVIDER, ICONS_AT};
use std::pin::Pin;

#[cxx_qt::bridge]
mod ffi {
    // SAFETY: `icons.h` declares exactly these functions with these types; they run on the Qt GUI thread, the engine
    // owns the provider it is given, and `heard` is a plain Rust function, safe to call from any thread.
    unsafe extern "C++" {
        include!("kx-ui/cpp/icons.h");
        include!("cxx-qt-lib/qqmlapplicationengine.h");
        /// Qt's QML engine for an app.
        type QQmlApplicationEngine = cxx_qt_lib::QQmlApplicationEngine;

        /// Registers the provider on `engine` as `name`, over the icons of type `ext` in folder `root`; `heard` hears
        /// each refused id.
        #[namespace = "kxui"]
        fn serve_icons(
            engine: Pin<&mut QQmlApplicationEngine>,
            name: &str,
            root: &str,
            ext: &str,
            heard: fn(&str),
        );

        /// What a provider over `root` and `ext` serves for `id` at `side` pixels square, as premultiplied ARGB.
        #[cfg(debug_assertions)]
        #[namespace = "kxui"]
        fn icon_pixels(root: &str, ext: &str, id: &str, side: i32, heard: fn(&str)) -> Vec<u32>;

        /// File `name` in `root` read at `side` pixels square without a tint, as premultiplied ARGB.
        #[cfg(debug_assertions)]
        #[namespace = "kxui"]
        fn icon_mask(root: &str, name: &str, side: i32) -> Vec<u32>;

        /// How many icon textures the provider made so far.
        #[cfg(debug_assertions)]
        #[namespace = "kxui"]
        fn icon_textures_made() -> u64;

        /// How many icon textures are alive now.
        #[cfg(debug_assertions)]
        #[namespace = "kxui"]
        fn icon_textures_live() -> u64;
    }
}

/// The bundled icons' folder in the resources.
fn root() -> String {
    format!("{FILES}/{ICONS_AT}")
}

/// Logs one refused icon; its id holds only the app's own icon name and tint.
fn refused(id: &str) {
    tracing::warn!(
        icon = id,
        "an icon was refused or did not read, so it shows nothing"
    );
}

/// Registers the icon provider on `engine`, which then draws `<icon_url>/<rrggbb>/<name>.<ext>`; call it on the Qt
/// GUI thread before the engine loads QML.
pub fn serve_icons(engine: Pin<&mut QQmlApplicationEngine>) {
    ffi::serve_icons(engine, ICON_PROVIDER, &root(), ICON_EXT, refused);
}

/// What the provider serves for `id`, `<rrggbb>/<name>.<ext>`, at `side` pixels square, as premultiplied ARGB words
/// row by row; debug builds only, for tests.
#[cfg(debug_assertions)]
#[must_use]
pub fn icon_pixels(id: &str, side: i32) -> Vec<u32> {
    ffi::icon_pixels(&root(), ICON_EXT, id, side, refused)
}

/// Bundled icon file `name` read at `side` pixels square without a tint, as premultiplied ARGB words row by row; debug
/// builds only, for tests.
#[cfg(debug_assertions)]
#[must_use]
pub fn icon_mask(name: &str, side: i32) -> Vec<u32> {
    ffi::icon_mask(&root(), name, side)
}

/// How many icon textures the provider made so far, and how many are alive now; debug builds only, for tests.
#[cfg(debug_assertions)]
#[must_use]
pub fn icon_textures() -> (u64, u64) {
    (ffi::icon_textures_made(), ffi::icon_textures_live())
}
