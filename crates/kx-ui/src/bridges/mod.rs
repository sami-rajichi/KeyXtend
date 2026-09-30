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

mod keyboard;
mod ui;

use crate::hub::Hub;
use cxx_qt_lib::QString;
use std::sync::{Arc, OnceLock};

/// The hub the bridge objects share, set once by the app before QML loads.
static HUB: OnceLock<Arc<Hub>> = OnceLock::new();

/// Hands `hub` to the bridge objects QML makes; call it once, before loading QML. False when one was set already.
pub fn install(hub: Arc<Hub>) -> bool {
    HUB.set(hub).is_ok()
}

/// The installed hub; a bridge made before `install` gets `None` and does nothing.
fn hub() -> Option<Arc<Hub>> {
    HUB.get().cloned()
}

/// `text` as a Qt string.
fn qs(text: &str) -> QString {
    QString::from(text)
}
