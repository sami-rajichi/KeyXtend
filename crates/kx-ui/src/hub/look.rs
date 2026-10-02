//! The look side of the hub: the look view QML draws, following the system look, the theme button, the light or
//! dark switch and the backdrop.

use super::{Hub, NULL, Wake, json};
use crate::plate::{Plate, plate_after};
use crate::settings::{Backdrop, UiSettings};
use kx_module_api::WriteError;
use kx_platform::ui::{SysLook, UiError, WatchGuard};
use kx_theme::{Mode, ModeChoice, look_view, mode_for};
use std::sync::Arc;
use toml::Value;

/// The `[ui]` key of the theme.
const THEME_KEY: &str = "theme";
/// The `[ui]` key of the mode.
const MODE_KEY: &str = "mode";

/// A board row in the theme's single precision.
#[allow(
    clippy::cast_possible_truncation,
    reason = "a keyboard row is tens of pixels, which f32 holds exactly enough"
)]
fn narrow(row_px: f64) -> f32 {
    row_px as f32
}

impl Hub {
    /// The look to draw now, as JSON; `null` when the theme is missing. The accent is checked on the face's board
    /// row, so it waits for the face.
    #[must_use]
    pub fn look_json(&self) -> String {
        let (choice, sys, face) = {
            let live = self.lock();
            (live.settings.choice(), live.sys, live.face.clone())
        };
        let row = face.map(|f| narrow(f.row_px()));
        let view = look_view(&self.themes, &choice, &sys, row);
        view.map_or_else(|| NULL.to_owned(), |v| json(&v))
    }

    /// Takes a new system look, as the watch reports it from any thread.
    pub fn set_sys(&self, sys: SysLook) {
        self.lock().sys = sys;
        self.wake(Wake::Look);
    }

    /// Follows the system look until the guard drops; the watch holds the hub weakly.
    ///
    /// # Errors
    /// `UiError` when the system will not report changes; the look stays as read at start.
    pub fn watch(self: &Arc<Self>) -> Result<WatchGuard, UiError> {
        let hub = Arc::downgrade(self);
        self.ports.look.watch(Box::new(move |sys| {
            if let Some(hub) = hub.upgrade() {
                hub.set_sys(sys);
            }
        }))
    }

    /// Shows the next theme, in the order the theme button cycles, and saves it; false when it did not change.
    pub fn step_theme(&self) -> bool {
        let now = self.lock().settings.theme.clone();
        let list = &self.themes.list;
        let at = list.iter().position(|t| t.look.id == now);
        let next = at.map_or(0, |i| (i + 1) % list.len().max(1));
        let Some(id) = list.get(next).map(|t| t.look.id.clone()) else {
            return false;
        };
        self.save(THEME_KEY, Value::String(id.clone()), |s| s.theme = id)
    }

    /// Switches to light or dark, the other one from what shows now, and saves it; false when it did not change.
    pub fn flip_mode(&self) -> bool {
        let (choice, sys) = {
            let live = self.lock();
            (live.settings.mode, live.sys)
        };
        let mode = match mode_for(choice, &sys) {
            Mode::Dark => ModeChoice::Light,
            Mode::Light => ModeChoice::Dark,
        };
        let Ok(value) = Value::try_from(mode) else {
            return false;
        };
        self.save(MODE_KEY, value, |s| s.mode = mode)
    }

    /// Puts the chosen backdrop behind window `title` in the mode shown; true when the plate may be see-through.
    /// Call it on the UI thread, once the window shows.
    pub fn backdrop(&self, title: &str) -> bool {
        let (settings, sys) = {
            let live = self.lock();
            (live.settings.clone(), live.sys)
        };
        let kind = settings.backdrop.kind();
        if settings.backdrop == Backdrop::None {
            return false;
        }
        let dark = mode_for(settings.mode, &sys) == Mode::Dark;
        let tried = self.ports.backdrop.apply(title, kind, dark);
        if let Err(e) = tried {
            tracing::warn!(error = %e, "backdrop refused; the plate stays solid");
        }
        plate_after(kind, tried) == Plate::SeeThrough
    }

    /// Saves `[ui]` `key` through the module's own handle, then `apply`s it; a save that failed still holds this run.
    fn save(&self, key: &str, value: Value, apply: impl FnOnce(&mut UiSettings)) -> bool {
        let Some(store) = self.lock().store.clone() else {
            return false;
        };
        match store.set(key, value) {
            Ok(()) | Err(WriteError::Unsaved) => {}
            Err(e) => {
                tracing::warn!(key, kind = e.kind(), "a [ui] value was refused");
                return false;
            }
        }
        apply(&mut self.lock().settings);
        self.wake(Wake::Look);
        true
    }
}
