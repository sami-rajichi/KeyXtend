//! The clipboard in the face's state (spec §7): whether the history chord is wanted, as the bus tells, and the paste
//! the clipboard module asks for. Its panel is one of ours, in `panels.rs`.

use super::Inner;

impl Inner {
    /// Takes whether the clipboard module wants the platform's history chord.
    pub(in crate::face) fn heard_history(&mut self, on: bool) {
        self.typing.want_history(on);
    }

    /// Drops a held key, lets every latch go and takes the aim, which the next relabel redraws, then sends one plain
    /// paste; true when all of it went. `None` while the module is stopped or a fill waits, which touches nothing.
    pub(in crate::face) fn paste(&mut self) -> Option<bool> {
        if !self.running || self.filling() {
            return None;
        }
        self.repeat.leave();
        self.lose_aim();
        let table = self.ports.layouts.current();
        let cells = self.layer.cells();
        let went = self.typing.paste(&self.ports.io(&table, &cells));
        self.unshown = true;
        Some(went.is_ok())
    }
}
