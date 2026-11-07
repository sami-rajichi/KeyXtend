//! Whether the open panel's line is aimed at (ADR-0040): while it is not, every key reaches the app. Outside clicks are
//! only counted, as their handler takes no lock, so a read compares the count first; a change shows at the next relabel.

use super::{Inner, STATE};
use crate::panel::Panel;
use kx_module_api::ports::{FaceReply, PanelLine};

/// The aim on the open panel's line.
#[derive(Default)]
pub(super) struct Aim {
    /// The line takes the keys.
    on: bool,
    /// The outside clicks heard when it aimed.
    clicks: u64,
    /// The window in front when it aimed, once the system could name one.
    front: Option<u64>,
    /// While a form's line shows, whether the aim was on before it.
    form: Option<bool>,
    /// The line the panel's module said shows, with a search's text.
    line: PanelLine,
}

impl Inner {
    /// Aims at the open panel's line, with `clicks` outside clicks heard so far.
    fn aim_at(&mut self, clicks: u64) {
        self.unshown |= !self.aim.on;
        self.aim.on = true;
        self.aim.clicks = clicks;
        self.aim.front = self.ports.injector.front_window();
        self.note_line();
    }

    /// A new panel's line: aimed, with no form to go back from, and none of the old panel's text.
    pub(super) fn aim_fresh(&mut self, clicks: u64) {
        self.aim.form = None;
        self.aim.line = PanelLine::None;
        self.aim_at(clicks);
    }

    /// The panel closed, so nothing is aimed at.
    pub(super) fn aim_end(&mut self) {
        self.aim = Aim::default();
        self.note_line();
    }

    /// A fill or a paste went from the panel: the app has the letters.
    pub(super) fn lose_aim(&mut self) {
        self.unshown |= self.aim.on;
        self.aim.on = false;
        self.note_line();
    }

    /// The line in effect for prediction: the panel's while aimed at, and only a running module's aim counts.
    pub(super) fn aimed_line(&self) -> &PanelLine {
        if self.aim.on && self.running {
            &self.aim.line
        } else {
            &PanelLine::None
        }
    }

    /// Checks the aim against `clicks` outside clicks and the window in front: one heard since, or another window, ends
    /// it. A form's letters may be a secret, so only a click moves them. A window the system cannot name changes
    /// nothing, and the first it names is where the aim stays.
    pub(in crate::face) fn settle_aim(&mut self, clicks: u64) {
        if !self.aim.on {
            return;
        }
        if self.aim.clicks != clicks {
            self.lose_aim();
            return;
        }
        if matches!(self.aim.line, PanelLine::Private) {
            return;
        }
        let Some(now) = self.ports.injector.front_window() else {
            return;
        };
        match self.aim.front {
            Some(was) if was != now => self.lose_aim(),
            _ => self.aim.front = Some(now),
        }
    }

    /// The line is aimed at; only an open panel's ever is.
    pub(in crate::face) fn aimed(&self) -> bool {
        self.aim.on
    }

    /// A press on the open panel, with `clicks` outside clicks heard so far; no panel, no aim.
    pub(in crate::face) fn aim_panel(&mut self, clicks: u64) -> FaceReply {
        if self.open.is_none() {
            return FaceReply::default();
        }
        self.aim_at(clicks);
        STATE
    }

    /// `panel`'s module says which line shows: a form's aims and saves the aim, and leaving it puts the aim back off if
    /// it was off; one taken meanwhile stays off. A word for a panel not open changes nothing, and prediction hears a
    /// line only while it is aimed at.
    pub(in crate::face) fn heard_line(&mut self, panel: Panel, line: &PanelLine, clicks: u64) {
        if self.open != Some(panel) {
            return;
        }
        self.settle_aim(clicks);
        self.aim.line = line.clone();
        self.follow_form(line, clicks);
        self.note_line();
    }

    /// A form's line aims and saves the aim, and leaving it puts the aim back off if it was off.
    fn follow_form(&mut self, line: &PanelLine, clicks: u64) {
        match (self.aim.form, matches!(line, PanelLine::Private)) {
            (None, true) => {
                self.aim.form = Some(self.aim.on);
                self.aim_at(clicks);
            }
            (Some(was), false) => {
                self.aim.form = None;
                if !was {
                    self.lose_aim();
                }
            }
            _ => {}
        }
    }

    /// True while the mouse module's presses are wanted: a panel is open and the module runs.
    pub(in crate::face) fn pressing(&self) -> bool {
        self.open.is_some() && self.running
    }
}
