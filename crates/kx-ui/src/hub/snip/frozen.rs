//! The frozen screens never trap the user (ADR-0059): they lie under every other window of ours while they show, and
//! the hand's Esc is heard then, which Snip takes as our keyboard's Esc.

use crate::hub::Hub;
use crate::i18n::keys::{SNIP_WINDOWS, WINDOW_SNIP_PANEL};
use kx_module_api::ports::{SnipClick, SnipPage};
use kx_platform::ui::{UiError, WatchGuard};
use std::sync::Weak;

/// The hand's Esc watch, on only while frozen screens lie low.
#[derive(Default)]
pub(super) enum Listen {
    /// No frozen screen shows.
    #[default]
    Off,
    /// The watch hears Esc until its guard drops.
    On {
        /// Held only for its drop, which stops the watch.
        _guard: WatchGuard,
    },
    /// The system would not listen for these frozen screens; asked again for the next ones.
    Refused,
}

impl Hub {
    /// Snip's window titles in the keyboard's language, its panel's too, which guard runs spare: their windows leave the
    /// top while a system dialog shows, and the guard must not lift them back over it.
    pub(in crate::hub) fn snip_titles(&self) -> Vec<String> {
        let texts = self.i18n();
        let keys = SNIP_WINDOWS.into_iter().chain([WINDOW_SNIP_PANEL]);
        keys.map(|k| texts.text(k)).collect()
    }

    /// The Qt thread's work as Snip's windows change or a frozen screen moves: QML gives the frozen screens' `title`,
    /// kept for the windows of ours that show later, and they go under our other windows.
    pub fn snip_order(&self, title: &str) {
        title.clone_into(&mut self.snip().screen);
        self.snip_lay();
    }

    /// The Qt thread's work as one of our windows shows: while frozen screens show and no system dialog sits in front,
    /// they go under every other window of ours, by the title QML gave last, so the bar and the keyboard stay above.
    pub fn snip_lay(&self) {
        let title = self.snip().screen.clone();
        if title.is_empty() || !self.snip_page().as_deref().is_some_and(lies_low) {
            return;
        }
        match self.ports.guard.lay_low(&title) {
            // Frozen screens Qt has not made yet go under as they show.
            Ok(()) | Err(UiError::NoWindow) => {}
            Err(e) => {
                if !std::mem::replace(&mut self.snip().warned, true) {
                    tracing::warn!(error = %e, "the frozen screens could not go under our windows");
                }
            }
        }
    }

    /// The Qt thread's work at each view: the hand's Esc is heard while frozen screens lie low, and the watch stops
    /// once they go, outside the lock, as its thread may be telling a press; a refused order warns again only for the
    /// next frozen screens.
    pub(super) fn snip_listen(&self) {
        let page = self.snip_page();
        let low = page.as_deref().is_some_and(lies_low);
        let frozen = page.is_some_and(|p| !p.frozen.is_empty());
        let (gone, me) = {
            let mut s = self.snip();
            s.warned &= frozen;
            match (matches!(s.esc, Listen::Off), low) {
                (true, true) => (None, Some(Weak::clone(&s.me))),
                (false, false) => (Some(std::mem::take(&mut s.esc)), None),
                _ => (None, None),
            }
        };
        drop(gone);
        if let Some(me) = me {
            let esc = self.snip_watch(me);
            self.snip().esc = esc;
        }
    }

    /// Starts the watch, which holds the hub weakly; a refusal warns once for these frozen screens.
    fn snip_watch(&self, me: Weak<Self>) -> Listen {
        let on = Box::new(move || {
            if let Some(hub) = me.upgrade() {
                hub.snip_esc();
            }
        });
        match self.ports.esc.watch(on) {
            Ok(guard) => Listen::On { _guard: guard },
            Err(e) => {
                tracing::warn!(error = %e, "the Esc key cannot end a snip");
                Listen::Refused
            }
        }
    }

    /// A press of the hand's Esc, on the watch's thread: Snip hears it only while the newest view still lies low.
    fn snip_esc(&self) {
        if self.snip_page().as_deref().is_some_and(lies_low) {
            self.snip_now(SnipClick::Esc);
        }
    }
}

/// True while `page` shows frozen screens that must lie under our windows: no system dialog sits in front, which
/// Snip's windows leave the top for.
fn lies_low(page: &SnipPage) -> bool {
    !page.frozen.is_empty() && !page.dialog
}
