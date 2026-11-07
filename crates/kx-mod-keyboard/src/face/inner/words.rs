//! Word prediction in the face's state (ADR-0033): the pieces typing told, the typing mode, picked words one part at a
//! time, and whether the module runs.

use super::Inner;
use crate::legend::LegendMods;
use crate::panel::Panel;
use crate::run::{Part, Pause, Word};
use kx_module_api::Redacted;
use kx_module_api::ports::{PanelLine, TypeWord, TypingMode};
use std::sync::Arc;

/// Where a picked word goes (ADR-0040 decision 8).
pub(in crate::face) enum WordTo {
    /// Into the app, part by part.
    App(Word),
    /// Into this panel's search line, as its keys.
    Line(Panel),
    /// Nowhere: its target no longer matches the aim, or typing refused it.
    Nowhere,
}

impl Inner {
    /// Notes whether the module runs; a picked word, a paste and a fill go only while it does, and a line is in effect.
    pub(in crate::face) fn set_running(&mut self, on: bool) {
        self.running = on;
        self.note_line();
    }

    /// Where `word` goes, with `clicks` outside clicks heard so far: one for a line goes only to a search in effect, one
    /// for the app only while no line is aimed at.
    pub(in crate::face) fn word_to(&mut self, word: &TypeWord, clicks: u64) -> WordTo {
        self.settle_aim(clicks);
        let to = if word.line {
            self.line_word(word.erase, &word.text).map(WordTo::Line)
        } else if matches!(self.aimed_line(), PanelLine::None) {
            self.start_word(word.erase, &word.text).map(WordTo::App)
        } else {
            None
        };
        to.unwrap_or(WordTo::Nowhere)
    }

    /// The panel that takes a word of `erase` Backspaces, then `text`, for its search line, by the rules of a word for
    /// the app; a burst ends and the latches let go as after a letter.
    fn line_word(&mut self, erase: u16, text: &Redacted<String>) -> Option<Panel> {
        let search = matches!(self.aimed_line(), PanelLine::Search(_));
        let panel = self.open.filter(|_| search && !self.filling())?;
        let table = self.ports.layouts.current();
        let cells = self.layer.cells();
        let io = self.ports.io(&table, &cells);
        if !self.typing.takes(&io, erase, text.expose()) {
            return None;
        }
        self.ended();
        self.typing.spent();
        self.unshown = true;
        Some(panel)
    }

    /// A picked word of `erase` Backspaces, then `text`, while the module runs, no fill waits and typing takes it.
    fn start_word(&self, erase: u16, text: &Redacted<String>) -> Option<Word> {
        if !self.running || self.filling() {
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
