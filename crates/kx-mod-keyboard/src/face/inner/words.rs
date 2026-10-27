//! Word prediction in the face's state (ADR-0033): the pieces typing told, the typing mode, picked words one part at a
//! time, and whether the module runs.

use super::Inner;
use crate::legend::LegendMods;
use crate::run::{Part, Pause, Word};
use kx_module_api::Redacted;
use kx_module_api::ports::{Piece, TypingMode};
use std::sync::Arc;

impl Inner {
    /// Notes whether the module runs; a picked word types only while it does.
    pub(in crate::face) fn set_running(&mut self, on: bool) {
        self.running = on;
    }

    /// A picked word of `erase` Backspaces, then `text`, while the module runs and typing takes it.
    pub(in crate::face) fn start_word(&self, erase: u16, text: &Redacted<String>) -> Option<Word> {
        if !self.running {
            return None;
        }
        let table = self.ports.layouts.current();
        let cells = self.layer.cells();
        self.typing
            .word(&self.ports.io(&table, &cells), erase, text)
    }

    /// Sends `word`'s next part while the module runs and nothing `stop`s it, else ends it; the next relabel redraws the
    /// latches it used.
    pub(in crate::face) fn word_part(&mut self, word: &mut Word, stop: bool) -> Part {
        let part = if self.running && !stop {
            let table = self.ports.layouts.current();
            let cells = self.layer.cells();
            self.typing.part(&self.ports.io(&table, &cells), word)
        } else {
            self.typing.end(word)
        };
        if part == Part::Done(true) {
            self.unshown = true;
        }
        part
    }

    /// The pause after each Backspace of a picked word.
    pub(in crate::face) fn pause(&self) -> Arc<dyn Pause> {
        Arc::clone(&self.ports.pause)
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
