//! Whether the line of the panel that holds the place is aimed at (ADR-0040, ADR-0057): while it is not, every key
//! reaches the app. Outside clicks are only counted, as their handler takes no lock, so a read compares the count first;
//! a change shows at the next relabel.

use super::{Inner, STATE};
use crate::panel::{Panel, PanelUse};
use crate::spec::Nav;
use kx_module_api::ports::{FaceReply, PanelLine};

/// The aim on the line of the panel that holds the place, and the open panel's line.
#[derive(Default)]
pub(super) struct Aim {
    /// The line takes the keys.
    on: bool,
    /// The outside clicks heard when it aimed.
    clicks: u64,
    /// The window in front when it aimed, once the system could name one.
    front: Option<u64>,
    /// While the open panel's form line shows, whether the aim was on before it.
    form: Option<bool>,
    /// The line the open panel's module said shows, with a search's text; Snip's is in `snip.rs`.
    line: PanelLine,
}

impl Inner {
    /// Aims at the line of the panel that holds the place, with `clicks` outside clicks heard so far.
    pub(super) fn aim_at(&mut self, clicks: u64) {
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

    /// The open panel closed: its line goes, and so does the aim unless Snip's windows hold the place.
    pub(super) fn aim_end(&mut self) {
        if self.snip_held().is_some() {
            self.aim.line = PanelLine::None;
            self.aim.form = None;
        } else {
            self.aim = Aim::default();
            self.note_line();
        }
    }

    /// The line of the panel that holds the place: Snip's while their windows hold it, else the open panel's.
    fn place_line(&self) -> &PanelLine {
        self.snip_held().unwrap_or(&self.aim.line)
    }

    /// A fill or a paste went from the panel: the app has the letters.
    pub(super) fn lose_aim(&mut self) {
        self.unshown |= self.aim.on;
        self.aim.on = false;
        self.note_line();
    }

    /// The line in effect for prediction: the place's while aimed at, and only a running module's aim counts.
    pub(super) fn aimed_line(&self) -> &PanelLine {
        if self.aim.on && self.running {
            self.place_line()
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
        if matches!(self.place_line(), PanelLine::Private) {
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

    /// The open `panel` takes `key` while aimed at: the Voice panel only on a line, and never its Esc, which leaves the
    /// text and reaches the app (ADR-0049).
    pub(super) fn takes(&self, panel: Panel, key: PanelUse) -> bool {
        let line = matches!(self.aim.line, PanelLine::Private);
        panel != Panel::Voice || (line && key != PanelUse::Nav(Nav::Esc))
    }

    /// The place's line is aimed at; only a shown panel's ever is.
    pub(in crate::face) fn aimed(&self) -> bool {
        self.aim.on
    }

    /// The panel whose line takes the keys now, which the state names so only its line draws a caret.
    pub(super) fn aimed_at(&self) -> Option<Panel> {
        self.place().filter(|_| self.aim.on)
    }

    /// A press on the open panel, with `clicks` outside clicks heard so far: it takes the keys back from Snip's
    /// windows, which stay. No panel, no aim.
    pub(in crate::face) fn aim_panel(&mut self, clicks: u64) -> FaceReply {
        if self.open.is_none() {
            return FaceReply::default();
        }
        self.snip_leave();
        self.aim_at(clicks);
        STATE
    }

    /// `panel`'s module says which line shows: a form's aims and saves the aim, and leaving it puts the aim back off if
    /// it was off; one taken meanwhile stays off. A word for a panel not open changes nothing, one behind Snip's windows
    /// only notes its line and drops the saved aim when a form ends, and prediction hears a line only while aimed at.
    pub(in crate::face) fn heard_line(&mut self, panel: Panel, line: &PanelLine, clicks: u64) {
        if self.open != Some(panel) {
            return;
        }
        if self.snip_held().is_some() {
            if !matches!(line, PanelLine::Private) {
                self.aim.form = None;
            }
            self.aim.line = line.clone();
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

    /// True while the mouse module's presses are wanted: a panel or Snip's line shows and the module runs.
    pub(in crate::face) fn pressing(&self) -> bool {
        self.place().is_some() && self.running
    }
}
