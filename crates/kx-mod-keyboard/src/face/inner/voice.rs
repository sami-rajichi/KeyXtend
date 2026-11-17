//! Voice typing in the face's state (ADR-0042 decision 7): the Dictation key's light, and a dictated text one piece
//! per lock. Like an edit, a text keeps every latch, and its answer waits behind its own break.

use super::Inner;
use crate::run::{Part, Text};
use kx_module_api::ports::{TextAsk, TextDone, VoiceLit};

impl Inner {
    /// The voice module says whether the microphone is open; a change redraws the key at the next relabel.
    pub(in crate::face) fn heard_voice(&mut self, lit: VoiceLit) {
        if self.voice != lit {
            self.voice = lit;
            self.unshown = true;
        }
    }

    /// Starts text `ask`: a held key ends as its let-go does and the aim goes, as for an edit. `None` once its answer
    /// is queued: no while the module is stopped, a fill waits or typing refuses it, yes when it asks for nothing.
    pub(in crate::face) fn start_text(&mut self, ask: &TextAsk) -> Option<Text> {
        let text = if !self.running || self.filling() {
            None
        } else {
            self.typing.text(ask)
        };
        let ok = text.is_some();
        let Some(text) = text.filter(Text::left) else {
            let round = ask.round;
            self.text_done(TextDone {
                round,
                count: 0,
                ok,
            });
            return None;
        };
        self.unshown |= self.ended().state;
        self.lose_aim();
        Some(text)
    }

    /// Sends `text`'s next piece while the module runs and nothing `stop`s it, else ends it unfinished.
    pub(in crate::face) fn text_part(&mut self, text: &mut Text, stop: bool) -> Part {
        if !self.running || stop {
            return text.stop();
        }
        let table = self.ports.layouts.current();
        let cells = self.layer.cells();
        self.typing.text_part(&self.ports.io(&table, &cells), text)
    }
}
