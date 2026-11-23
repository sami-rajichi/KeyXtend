//! The Voice panel's line for the keyboard (ADR-0049): which line the page shows, told after its view once per change,
//! again when asked and closed at the stop; the keys the keyboard sends while it is aimed at; and the close another
//! panel brings.

use kx_module_api::ports::{PanelKey, PanelLine, VoiceLine};

use super::Actor;

impl Actor {
    /// Tells the keyboard the panel's `line`, read from its view's facts, when it changed since last told.
    pub(super) fn tell_line(&mut self, line: Option<PanelLine>) {
        if line != self.line {
            self.line.clone_from(&line);
            self.publish(&VoiceLine(line));
        }
    }

    /// Tells the keyboard the panel's line again, for a late starter.
    pub(super) fn retell_line(&self) {
        self.publish(&VoiceLine(self.line.clone()));
    }

    /// Tells the keyboard the panel is gone, at the stop.
    pub(super) fn drop_line(&mut self) {
        if self.line.take().is_some() {
            self.publish(&VoiceLine(None));
        }
    }

    /// A key the keyboard sent to the page's line, worked as the panel says.
    pub(super) fn line_key(&mut self, key: &PanelKey) {
        let facts = self.held.facts(self.ports.calendar.now(), &self.flow);
        let dos = self.panel.line_key(key, &facts);
        self.dos(dos);
    }

    /// Another panel opened: this one closes as its Close does; a late close finds it shut and does nothing.
    pub(super) fn close_panel(&mut self) {
        if self.panel.page().is_some() {
            let dos = self.panel.close();
            self.dos(dos);
        }
    }
}
