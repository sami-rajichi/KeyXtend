//! `Ui`'s Rust side: the hub it reads, the system look's watch and the look the windows drew last.

use super::qobject;
use crate::bridges::hub;
use crate::hub::Hub;
use cxx_qt::CxxQtType;
use kx_platform::ui::WatchGuard;
use std::sync::Arc;

/// Rust side of `Ui`.
pub struct UiRust {
    pub(super) hub: Option<Arc<Hub>>,
    /// Keeps the system look watch running while the object lives.
    pub(super) watch: Option<WatchGuard>,
    /// The look the windows drew last, as QML reads it, so only a new one drops Qt's cached graphics.
    pub(super) drawn: String,
}

impl Default for UiRust {
    fn default() -> Self {
        let hub = hub();
        let drawn = hub.as_deref().map(Hub::look_json).unwrap_or_default();
        Self {
            hub,
            watch: None,
            drawn,
        }
    }
}

impl qobject::Ui {
    /// `f` over the hub, or `none` before the app installed one.
    pub(super) fn with<T>(&self, none: T, f: impl FnOnce(&Hub) -> T) -> T {
        self.rust().hub.as_deref().map_or(none, f)
    }
}
