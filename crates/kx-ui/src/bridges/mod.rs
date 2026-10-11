//! The cxx-qt objects QML makes (ADR-0013): `Ui` for the app's look, texts, notices and commands, and `Keyboard`
//! for the keyboard window. They reach the app's hub through `install`, since QML makes them without arguments.
#![allow(
    unsafe_code,
    reason = "each cxx-qt bridge declares its C++ side in an unsafe extern block (ADR-0013, tidy F3)"
)]
#![allow(
    clippy::unnecessary_box_returns,
    reason = "cxx-qt's generated glue makes each object's Rust side in a Box"
)]

pub mod device;
pub mod fonts;
pub mod harden;
mod keyboard;
pub mod shadows;
pub mod tint;
mod ui;

use crate::hub::Hub;
use cxx_qt_lib::QString;
use std::sync::{Arc, OnceLock};

/// The resource folder of the bundled files, as a path Qt opens, from the build.
const FILES: &str = env!("KX_UI_FILES");

/// The hub the bridge objects share, set once by the app before QML loads (ADR-0018).
static HUB: OnceLock<Arc<Hub>> = OnceLock::new();

/// Hands `hub` to the bridge objects QML makes; call it once, before loading QML. False when one was set already.
pub fn install(hub: Arc<Hub>) -> bool {
    HUB.set(hub).is_ok()
}

/// The installed hub, if the app installed one.
pub(crate) fn installed() -> Option<Arc<Hub>> {
    HUB.get().cloned()
}

/// The installed hub for a new bridge object; one made before `install` warns and does nothing.
fn hub() -> Option<Arc<Hub>> {
    let hub = HUB.get().cloned();
    if hub.is_none() {
        tracing::warn!("a QML object was made before the hub was installed, so it does nothing");
    }
    hub
}

/// `text` as a Qt string.
fn qs(text: &str) -> QString {
    QString::from(text)
}
