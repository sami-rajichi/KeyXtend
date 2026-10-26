//! Word prediction in the face's state (ADR-0033): the pieces typing told, the typing mode, picked words, and whether
//! the module runs.

use super::Inner;
use crate::legend::LegendMods;
use kx_module_api::Redacted;
use kx_module_api::ports::{Piece, TypingMode};

impl Inner {
    /// Notes whether the module runs; a picked word types only while it does.
    pub(in crate::face) fn set_up(&mut self, on: bool) {
        self.up = on;
    }

    /// Types a picked word while the module runs: `erase` Backspaces, then `text`; the next relabel redraws the
    /// latches it used.
    pub(in crate::face) fn type_word(&mut self, erase: u16, text: &Redacted<String>) {
        if !self.up {
            return;
        }
        let table = self.ports.layouts.current();
        let cells = self.layer.cells();
        let io = self.ports.io(&table, &cells);
        if self.typing.type_word(&io, erase, text) {
            self.unshown = true;
        }
    }

    /// The pieces that reached the app since the last take, oldest first.
    pub(in crate::face) fn take_pieces(&mut self) -> Vec<Piece> {
        self.typing.take_pieces()
    }

    /// The latches, dead-key wait and text cap that change what a key or picked word types, as word prediction hears
    /// them, with Caps Lock as the UI's thread last read it, so any thread may build it.
    pub(in crate::face) fn mode(&self) -> TypingMode {
        let table = self.ports.layouts.current();
        let m = LegendMods::of(self.typing.latches(), self.caps.seen, &table.keys);
        TypingMode {
            shift: m.shift,
            caps: m.caps,
            shortcut: m.shortcut,
            dead: self.typing.waiting(),
            cap: Some(self.typing.cap()),
        }
    }
}
