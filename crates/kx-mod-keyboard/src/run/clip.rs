//! Typing's part in our panels (spec §7, ADR-0034): what a character key gives the search of the clipboard or
//! Quick-fill panel, the platform's history chord, and the plain paste the clipboard module asks for.

use super::{Io, TapError, Typing};
use crate::legend::{LegendMods, slot};
use crate::spec::{Nav, Role};
use kx_module_api::Redacted;
use kx_module_api::ports::{PASTE_ID, PanelKey, Scan, ShortcutError, ShortcutId};

/// What the open clipboard or Quick-fill panel does with a click.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ToPanel {
    /// Not the panel's: the key types as usual.
    Pass,
    /// The panel takes the click and nothing goes out: a dead or empty key.
    Nothing,
    /// The panel takes this key.
    Key(PanelKey),
}

impl Typing {
    /// Takes whether the clipboard module wants the platform's history chord, as the bus last told.
    pub fn want_history(&mut self, on: bool) {
        self.history = on;
    }

    /// Character key `sc` while either panel is open: the text it would type now in the layout shown, with
    /// Shift, `AltGr` and Caps Lock, after which the latches let go as after typing. A dead key gives nothing and keeps
    /// the wait for the accent the app may hold; with Ctrl, Alt or Win latched the key is not the panel's (`Pass`).
    pub fn search(&mut self, io: &Io<'_>, sc: Scan) -> ToPanel {
        if self.chorded(io) {
            return ToPanel::Pass;
        }
        let keys = &io.table.keys;
        let m = LegendMods::of(&self.latches, io.source.caps_on(), keys);
        let Some((k, s)) = keys.get(&sc).and_then(|k| slot(k, m).map(|s| (k, s))) else {
            return ToPanel::Nothing;
        };
        if k.is_dead(s) {
            self.spent();
            return ToPanel::Nothing;
        }
        let text = k.get(s);
        if text.is_empty() {
            return ToPanel::Nothing;
        }
        self.spent();
        ToPanel::Key(PanelKey::Text(Redacted::new(text.to_owned())))
    }

    /// Move `nav` while either panel is open: the panel's, but with Ctrl, Alt or Win latched it is the app's (`Pass`).
    /// Space types there, so the latches let go after it as after a letter.
    pub fn moved(&mut self, io: &Io<'_>, nav: Nav) -> ToPanel {
        if self.chorded(io) {
            return ToPanel::Pass;
        }
        if nav == Nav::Tick {
            self.spent();
        }
        ToPanel::Key(nav.into())
    }

    /// True while Ctrl, Alt or Win is latched, so a key is the app's shortcut and no panel's.
    fn chorded(&self, io: &Io<'_>) -> bool {
        LegendMods::shortcut(&self.latches, &io.table.keys)
    }

    /// True when key `id` with the latched keys is the platform's history chord while the clipboard module wants it;
    /// then the latches let go as after a key, and nothing reaches the app. A chord holds a key, so a key alone never
    /// asks the platform.
    pub(super) fn chord(&mut self, io: &Io<'_>, id: &str) -> bool {
        let Some(&(role, Some(sc))) = self.keys.get(id).filter(|_| self.history) else {
            return false;
        };
        let held = self.latches.held(&[]);
        let sends = matches!(role, Role::Char | Role::Send);
        let hit = sends && !held.is_empty() && io.shortcuts.is_history(&held, sc);
        if hit {
            self.spent();
        }
        hit
    }

    /// Lets every latch go whatever the mode and ends a burst, then sends the paste shortcut once with nothing held,
    /// as the clipboard module asked (ADR-0034 decision 2).
    ///
    /// # Errors
    /// `Shortcut` when the paste did not all reach the app.
    pub fn paste(&mut self, io: &Io<'_>) -> Result<(), TapError> {
        self.burst = None;
        self.latches.clear();
        let id = ShortcutId::new(PASTE_ID).ok_or(ShortcutError::Unknown)?;
        self.shortcut(io, &id)
    }

    /// The panel or the history chord took a key as typing: the latches let go as their mode says, or when the burst
    /// ends; nothing reached the app.
    pub fn spent(&mut self) {
        let now = self.latches.clone();
        self.used_of(&now);
    }
}
