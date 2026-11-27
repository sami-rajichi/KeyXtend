//! Snip's capture windows out of every capture: each is shielded once Qt made it, and one that cannot be stays hidden
//! while a countdown or recording runs, and the module hears `Bare`.

use super::acts::CAPTURE;
use crate::hub::Hub;
use kx_module_api::ports::{SnipClick, SnipPage, SnipStage};

impl Hub {
    /// Keeps Snip's capture window `window`, one of `CAPTURE`, titled `title`, out of every capture; call it on the UI
    /// thread once Qt made the window and before it shows. False for another window or a refused shield, which keeps
    /// that window hidden while a countdown or recording runs; a new refusal during a snip tells the module `Bare`.
    pub fn snip_shield(&self, window: &str, title: &str) -> bool {
        let Some(&id) = CAPTURE.iter().find(|w| **w == window) else {
            tracing::warn!(
                "QML asked to shield a window that is not one of Snip's capture windows"
            );
            return false;
        };
        let held = self.ports.guard.shield(title, true);
        if let Err(e) = &held {
            tracing::warn!(error = %e, window = id, "a Snip window could not be kept out of captures");
        }
        let changed = {
            let mut s = self.snip();
            let before = !s.bare.contains(&id);
            s.bare.retain(|w| *w != id);
            if held.is_err() {
                s.bare.push(id);
            }
            before != held.is_ok()
        };
        if changed {
            self.snip_view_woken();
            if held.is_err() && self.snip_live() {
                self.snip_now(SnipClick::Bare);
            }
        }
        held.is_ok()
    }
}

/// The capture windows QML keeps hidden on `page`: those `bare` of their shield, while a countdown or recording runs.
pub(super) fn hidden(page: &SnipPage, bare: &[&'static str]) -> Vec<&'static str> {
    let live = matches!(page.stage, SnipStage::Count(_) | SnipStage::Record(_));
    CAPTURE
        .into_iter()
        .filter(|w| live && bare.contains(w))
        .collect()
}
