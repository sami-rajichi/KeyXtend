//! Our windows out of every capture while the keyboard is aside (P12, edge 16): the keyboard, its bubble and every other
//! window of ours but Snip's, all or none. One Qt shows later is shielded as it shows, and a refused shield hides ours.

use super::super::{Hub, Wake};
use crate::i18n::keys::{TRAY_WINDOWS, WINDOW_BUBBLE, WINDOW_KEYBOARD, WINDOWS};
use kx_platform::ui::UiError;

/// The aside's own windows, made before their shield, so one not found counts as refused.
const MADE: [&str; 2] = [WINDOW_KEYBOARD, WINDOW_BUBBLE];

impl Hub {
    /// The Qt thread's work at `Wake::Aside`: while aside, `make` makes the keyboard's and bubble's windows and every
    /// window of ours is kept out of captures, a refusal hiding ours; else all are let back in. True when it shielded.
    #[must_use]
    pub fn shield_aside(&self, make: &dyn Fn(&str)) -> bool {
        let on = self.lock().aside.is_some();
        let mut held = true;
        for (key, title) in self.ours() {
            let own = MADE.contains(&key);
            if on && own {
                make(&title);
            }
            match self.ports.guard.shield(&title, on) {
                Ok(()) => {}
                // A window never made has nothing to let back in, and one Qt shows later is shielded as it shows.
                Err(UiError::NoWindow) if !on || !own => {}
                Err(e) => {
                    held = false;
                    tracing::warn!(error = %e, on, "a window's capture shield did not change");
                }
            }
        }
        if on && !held {
            self.aside_bare(false);
        }
        on
    }

    /// The Qt thread's work as Qt is about to show our window titled `title`, made but not yet on screen: while the
    /// keyboard is aside, it is kept out of every capture first, and a refused shield hides ours. Others are left alone.
    pub fn aside_window(&self, title: &str) {
        let on = self.lock().aside.is_some();
        if !on || !self.ours().iter().any(|(_, t)| t == title) {
            return;
        }
        if let Err(e) = self.ports.guard.shield(title, true) {
            tracing::warn!(error = %e, "a window shown during a capture could not be kept out of it");
            self.aside_bare(true);
        }
    }

    /// Every title of ours the aside shields, by key, each in the language its window speaks.
    fn ours(&self) -> Vec<(&'static str, String)> {
        let (t, tray) = (self.i18n(), self.tray_i18n());
        let kept = WINDOWS.into_iter().map(|k| (k, t.text(k)));
        kept.chain(TRAY_WINDOWS.into_iter().map(|k| (k, tray.text(k))))
            .collect()
    }

    /// Marks the aside bare, so QML hides ours; `wake` tells QML, which its own wake needs not.
    fn aside_bare(&self, wake: bool) {
        let marked = self.lock().aside.as_mut().map(|a| a.bare = true).is_some();
        if marked && wake {
            self.wake(Wake::Aside);
        }
    }
}
