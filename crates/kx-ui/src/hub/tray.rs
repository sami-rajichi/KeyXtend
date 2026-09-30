//! The tray side of the hub: the icon's picture and tooltip, and its clicks, which reach the UI as wakes.

use super::{Hub, Wake};
use crate::i18n::keys::WINDOW_KEYBOARD;
use kx_platform::ui::{OnTray, TrayIcon, TrayImage, UiError};
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};

/// The tray icon's slot: none yet, the icon shown, or hidden for good as the app quits.
#[derive(Default)]
pub(super) enum TraySlot {
    #[default]
    Empty,
    Shown(Box<dyn TrayIcon>),
    Closed,
}

impl Hub {
    /// The tray picture's side in pixels, at the system scale.
    #[must_use]
    pub fn tray_px(&self) -> u32 {
        self.ports.tray.icon_px()
    }

    /// Shows `image` as the tray icon with the keyboard window's title as its tooltip, or changes it; call it on the UI
    /// thread. False when the system refused or the icon was hidden for good; its events come back as `Wake::Tray`.
    pub fn tray_show(self: &Arc<Self>, image: &TrayImage) -> bool {
        let tip = self.i18n().text(WINDOW_KEYBOARD);
        // Taken in its own statement, so the slot's lock is free again while the port works.
        let slot = std::mem::take(&mut *held(&self.tray));
        match slot {
            TraySlot::Closed => {
                *held(&self.tray) = TraySlot::Closed;
                false
            }
            TraySlot::Shown(icon) => {
                let set = icon.set(image, &tip);
                self.keep(icon) && set.map_err(warn).is_ok()
            }
            TraySlot::Empty => match self.ports.tray.show(image, &tip, self.on_tray()) {
                Ok(icon) => self.keep(icon),
                Err(e) => {
                    warn(e);
                    false
                }
            },
        }
    }

    /// Removes the tray icon for good, so no dead icon stays in the taskbar; call it before the app ends.
    pub fn tray_hide(&self) {
        let slot = std::mem::replace(&mut *held(&self.tray), TraySlot::Closed);
        drop(slot);
    }

    /// Keeps `icon` in its slot; a hide that came meanwhile wins, and the icon goes.
    fn keep(&self, icon: Box<dyn TrayIcon>) -> bool {
        let mut slot = held(&self.tray);
        if matches!(*slot, TraySlot::Closed) {
            drop(slot);
            drop(icon);
            return false;
        }
        *slot = TraySlot::Shown(icon);
        true
    }

    /// Hands each tray event to the UI as a wake; it holds the hub weakly, so the icon never keeps it alive.
    fn on_tray(self: &Arc<Self>) -> OnTray {
        let hub = Arc::downgrade(self);
        Box::new(move |event| {
            if let Some(hub) = hub.upgrade() {
                hub.wake(Wake::Tray(event));
            }
        })
    }
}

/// Logs a refused icon without the picture.
fn warn(e: UiError) {
    tracing::warn!(error = %e, "the tray icon was refused");
}

/// Locks the icon slot, taking it even when a panicking thread poisoned it.
fn held(slot: &Mutex<TraySlot>) -> MutexGuard<'_, TraySlot> {
    slot.lock().unwrap_or_else(PoisonError::into_inner)
}
