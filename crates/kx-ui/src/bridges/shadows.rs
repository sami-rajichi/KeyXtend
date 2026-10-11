//! The soft shadows' adopter on the app's engine: Qt 6.11 never frees a shadow's default material, so each soft shadow
//! hands it to itself through the adopter (`cpp/shadows.h`, `qml/Soft.qml`). Debug builds count the adoptions.

use cxx_qt_lib::QQmlApplicationEngine;
use std::pin::Pin;

#[cxx_qt::bridge]
mod ffi {
    // SAFETY: `shadows.h` declares exactly these functions with these types; they run on the Qt GUI thread, and the
    // engine owns the adopter it is given.
    unsafe extern "C++" {
        include!("kx-ui/cpp/shadows.h");
        include!("cxx-qt-lib/qqmlapplicationengine.h");
        /// Qt's QML engine for an app.
        type QQmlApplicationEngine = cxx_qt_lib::QQmlApplicationEngine;

        /// Puts the adopter on `engine`'s root context.
        #[namespace = "kxui"]
        fn serve_shadows(engine: Pin<&mut QQmlApplicationEngine>);

        /// How many materials the adopters gave their shadows so far.
        #[cfg(debug_assertions)]
        #[namespace = "kxui"]
        fn adopted_materials() -> usize;
    }
}

/// Puts the soft shadows' adopter on `engine`; call it on the Qt GUI thread before the engine loads QML.
pub fn serve_shadows(engine: Pin<&mut QQmlApplicationEngine>) {
    ffi::serve_shadows(engine);
}

/// How many materials the adopters gave their shadows so far; debug builds only, for tests.
#[cfg(debug_assertions)]
#[must_use]
pub fn adopted() -> usize {
    ffi::adopted_materials()
}
