//! Where the keyboard's letters go: a key to the open list, and a press on the open panel or on a Snip window that
//! shows a line, which takes the letters (ADR-0040, ADR-0057); each answers `{board, state}`.

use super::qobject;
use crate::hub::Hub;
use cxx_qt_lib::QString;

impl qobject::Keyboard {
    pub(super) fn panel_key(&self, id: &QString) -> QString {
        let id = id.to_string();
        self.redraw(|h| h.panel_key(&id))
    }

    pub(super) fn aim_panel(&self) -> QString {
        self.redraw(Hub::aim_panel)
    }

    pub(super) fn aim_snip(&self) -> QString {
        self.redraw(Hub::aim_snip)
    }
}
