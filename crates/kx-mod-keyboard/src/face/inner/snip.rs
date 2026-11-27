//! Snip's windows in the face's state (ADR-0057): the line they show, the Snip key's light, and a click on the key
//! whose line has not come yet. They show beside the open panel, and only the user's press on them or on the key gives
//! them the keys, so a line that shows on its own never takes keys the user aims elsewhere.

use super::{Inner, STATE};
use crate::panel::Panel;
use kx_module_api::ports::{FaceReply, PanelLine, SnipLit};

/// What the snip module last told, a click on its key, and whether the windows hold the place.
#[derive(Default)]
pub(super) struct Snip {
    /// The line Snip's windows show; `None` while none takes keys.
    line: Option<PanelLine>,
    /// The Snip key's light.
    lit: SnipLit,
    /// The outside clicks heard at a click on the Snip key while no line showed, so the next line takes the keys.
    asked: Option<u64>,
    /// The windows hold the place, so the keys go to them while aimed at; never while no line shows.
    place: bool,
}

impl Inner {
    /// The snip module tells the key's light; the ui's wake reads a change at once, else the next relabel does. A
    /// capture that ended drops a click whose line never came, such as one that ended a countdown.
    pub(in crate::face) fn heard_snip_lit(&mut self, lit: SnipLit) {
        if lit == SnipLit::Off {
            self.snip.asked = None;
        }
        if self.snip.lit != lit {
            self.snip.lit = lit;
            self.unshown = true;
        }
    }

    /// The Snip key's light, for the state.
    pub(super) fn snip_lit(&self) -> SnipLit {
        self.snip.lit
    }

    /// A click on the Snip key, with `clicks` outside clicks heard so far: a line that shows takes the keys now, else
    /// the next one told does. No panel closes.
    pub(in crate::face) fn snip_key(&mut self, clicks: u64) {
        if self.snip.line.is_some() {
            self.snip_take(clicks);
        } else {
            self.snip.asked = Some(clicks);
        }
    }

    /// A press on Snip's windows, with `clicks` outside clicks heard so far: their line takes the keys and the open
    /// panel stays; with no line, nothing.
    pub(in crate::face) fn aim_snip(&mut self, clicks: u64) -> FaceReply {
        if self.snip.line.is_none() {
            return FaceReply::default();
        }
        self.snip_take(clicks);
        STATE
    }

    /// The snip module tells its line: after a click on the Snip key it takes the keys, else it takes a free place
    /// unaimed or waits behind the open panel. `None` gives the place back.
    pub(in crate::face) fn heard_snip(&mut self, line: Option<&PanelLine>) {
        let asked = self.snip.asked.take();
        self.snip.line = line.cloned();
        if line.is_none() {
            return self.snip_gone();
        }
        if let Some(at) = asked {
            self.snip_take(at);
        } else {
            self.snip_back();
            self.note_line();
        }
    }

    /// Snip's windows take the place and aim, with `clicks` outside clicks heard at the user's click.
    fn snip_take(&mut self, clicks: u64) {
        if !std::mem::replace(&mut self.snip.place, true) {
            self.unshown = true;
        }
        self.aim_at(clicks);
    }

    /// Snip's windows take a free place, unaimed, while their line shows and no panel is open.
    pub(super) fn snip_back(&mut self) {
        if self.open.is_none() && self.snip.line.is_some() {
            self.snip.place = true;
        }
    }

    /// The open panel takes the place back from Snip's windows, which stay on screen.
    pub(super) fn snip_leave(&mut self) {
        if std::mem::take(&mut self.snip.place) {
            self.unshown = true;
        }
    }

    /// Snip's windows went: the open panel, if any, takes the place back unaimed, so the keys reach the app.
    fn snip_gone(&mut self) {
        if std::mem::take(&mut self.snip.place) {
            self.lose_aim();
        }
    }

    /// Snip's line while their windows hold the place.
    pub(super) fn snip_held(&self) -> Option<&PanelLine> {
        self.snip.line.as_ref().filter(|_| self.snip.place)
    }

    /// The panel that takes the keys while aimed at: Snip's windows while they hold the place, else the open panel.
    pub(super) fn place(&self) -> Option<Panel> {
        if self.snip.place {
            Some(Panel::Snip)
        } else {
            self.open
        }
    }

    /// Forgets what the snip module told, as it may have changed unheard while the keyboard was down.
    pub(in crate::face) fn forget_snip(&mut self) {
        self.heard_snip_lit(SnipLit::Off);
        self.heard_snip(None);
    }
}
