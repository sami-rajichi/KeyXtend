//! The clipboard panel in the face's state (spec §7): whether it is open and whether the history chord is wanted, as
//! the bus tells, the keys it takes, and the paste the clipboard module asks for.

use super::Inner;
use crate::panel::ClipUse;
use crate::run::ToPanel;
use std::collections::HashMap;

impl Inner {
    /// Takes the clipboard module's panel state; an open panel closes the language list, so one panel shows at a
    /// time. The next relabel redraws when either changed.
    pub(in crate::face) fn heard_clip(&mut self, open: bool) {
        let closed = open && self.list.take().is_some();
        if self.clip != open || closed {
            self.clip = open;
            self.unshown = true;
        }
    }

    /// Takes whether the clipboard module wants the platform's history chord.
    pub(in crate::face) fn heard_history(&mut self, on: bool) {
        self.typing.want_history(on);
    }

    /// Key `id` from the clipboard panel's key list `keys`: a move, or what a character key gives the search; `Pass`
    /// while the panel is shut or for a key it does not take.
    pub(in crate::face) fn clip_key(
        &mut self,
        id: &str,
        keys: &HashMap<String, ClipUse>,
    ) -> ToPanel {
        match keys.get(id).filter(|_| self.clip) {
            None => ToPanel::Pass,
            Some(&ClipUse::Nav(nav)) => ToPanel::Key(nav.into()),
            Some(&ClipUse::Char(sc)) => {
                let table = self.ports.layouts.current();
                let cells = self.layer.cells();
                self.typing.search(&self.ports.io(&table, &cells), sc)
            }
        }
    }

    /// Drops a held key and lets every latch go, then sends one plain paste; true when all of it went. The next
    /// relabel redraws the latches.
    pub(in crate::face) fn paste(&mut self) -> bool {
        self.repeat.leave();
        let table = self.ports.layouts.current();
        let cells = self.layer.cells();
        let went = self.typing.paste(&self.ports.io(&table, &cells));
        self.unshown = true;
        went.is_ok()
    }
}
