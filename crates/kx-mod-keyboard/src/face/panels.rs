//! Our panels on the face (spec §3.3, §3.4, §7, §8.1, ADR-0040): while one is open and aimed at its keys go to it instead
//! of the app, and one opening closes the others. As in `bus.rs`, nothing is sent while the face's lock is held.

use super::{Face, STATE};
use crate::panel::Panel;
use crate::run::ToPanel;
use kx_module_api::Bus;
use kx_module_api::ports::{
    ClipAct, ClipKey, FaceReply, FillClose, FillKey, PanelKey, PanelLine, VoiceClose, VoiceLineKey,
};
use std::sync::atomic::Ordering::SeqCst;

impl Face {
    /// While one of our panels is open and aimed at, key `id` goes to it if its key list takes the key: its module hears
    /// it, and the state redraws, as a letter lets latches go. `None` when the key types as usual: no panel is open or
    /// aimed at, the open language list drives it, or Ctrl, Alt or Win acts on the key.
    pub(super) fn to_panel(&self, id: &str) -> Option<FaceReply> {
        let (took, bus) = {
            let mut inner = self.lock();
            if inner.listing() && self.navs.contains_key(id) {
                return None;
            }
            (inner.took(id, &self.uses, self.outside())?, inner.bus())
        };
        match took {
            (_, ToPanel::Pass) => return None,
            (panel, ToPanel::Key(key)) => send_key(&bus, panel, key),
            (_, ToPanel::Nothing) => {}
        }
        Some(STATE)
    }

    /// Sends `key` to `panel`'s module, outside the lock.
    pub(super) fn key_to(&self, panel: Panel, key: PanelKey) {
        let bus = self.lock().bus();
        send_key(&bus, panel, key);
    }

    /// A panel's module said it opened or closed. A newly open one asks the others to close, whatever the keyboard last
    /// heard of them, with a close that never opens them, so a late word cannot leave two open; a repeated word, as
    /// each Voice line change, closes nothing. The mouse module hears whether presses are wanted.
    pub(super) fn heard_panel(&self, panel: Panel, open: bool) {
        let (new, bus) = {
            let mut inner = self.lock();
            let new = if open {
                inner.panel_opened(panel, self.outside())
            } else {
                inner.panel_shut(panel);
                false
            };
            (new, inner.bus())
        };
        if new {
            close_others(&bus, panel);
        }
        self.pour();
        self.tell(false);
    }

    /// The voice module said which line its panel shows, `None` once closed: the panel opens or closes as the others
    /// do on their words, and its line aims as theirs (ADR-0049).
    pub(super) fn heard_voice_line(&self, line: Option<&PanelLine>) {
        self.heard_panel(Panel::Voice, line.is_some());
        if let Some(line) = line {
            self.heard_line(Panel::Voice, line);
        }
    }

    /// `panel`'s module said which line shows; prediction hears it while it is aimed at.
    pub(super) fn heard_line(&self, panel: Panel, line: &PanelLine) {
        self.lock().heard_line(panel, line, self.outside());
        self.pour();
    }

    /// A press on the open panel.
    pub(super) fn aim_panel(&self) -> FaceReply {
        self.lock().aim_panel(self.outside())
    }

    /// How many clicks outside our windows were heard so far.
    pub(super) fn outside(&self) -> u64 {
        self.clicks.load(SeqCst)
    }
}

/// Sends `key` to `panel`'s module. The module holds `ReadClipboard`, `Vault` and `Voice`, which these keys need, or
/// its start would have failed.
fn send_key(bus: &Bus, panel: Panel, key: PanelKey) {
    let _ = match panel {
        Panel::Clip => bus.publish(&ClipKey(key)),
        Panel::Fill => bus.publish(&FillKey(key)),
        Panel::Voice => bus.publish(&VoiceLineKey(key)),
    };
}

/// Asks every panel but `open` to close. The module holds `ReadClipboard` and `Voice`, which the clipboard's and the
/// Voice panel's closes need; Quick-fill's needs none.
fn close_others(bus: &Bus, open: Panel) {
    if open != Panel::Fill {
        let _ = bus.publish(&FillClose);
    }
    if open != Panel::Clip {
        let _ = bus.publish(&ClipAct::Close);
    }
    if open != Panel::Voice {
        let _ = bus.publish(&VoiceClose);
    }
}
