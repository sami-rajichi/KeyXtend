//! A picked word (ADR-0033 decision 2): the keyboard stays the only typist, so prediction's word erases and types
//! through the same route, latches and `text_cap` as a key, and is told like keys. It goes out one part at a time,
//! with `erase_gap_ms` after each Backspace, as some apps drop keys sent in a fast burst.

use super::checks::reached;
use super::ports::Pause;
use super::{Io, Typing};
use crate::legend::LegendMods;
use crate::plan::text_route;
use kx_module_api::ports::{Piece, TypeWord};
use kx_module_api::{Redacted, is_hidden};
use std::time::Duration;

/// A picked word on its way out: the Backspaces left, then its text.
#[derive(Debug)]
pub struct Word {
    erase: u16,
    text: Option<Redacted<String>>,
    /// Some of it reached the app.
    any: bool,
}

/// What a picked word's part asks for next.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Part {
    /// A Backspace went out: wait this long, then send the next part.
    Gap(Duration),
    /// The word is done; true when any of it reached the app.
    Done(bool),
}

/// Sends a picked word through `part`, one part per call, waiting each nonzero gap through `pause` between them, so a
/// caller holds its lock for one part only; true when any of it reached the app.
pub fn drive(mut part: impl FnMut() -> Part, pause: &dyn Pause) -> bool {
    loop {
        match part() {
            Part::Gap(gap) if gap.is_zero() => {}
            Part::Gap(gap) => pause.wait(gap),
            Part::Done(any) => return any,
        }
    }
}

impl Typing {
    /// Erases `erase` characters with Backspace, then types `text` by the route a character key takes now, waiting
    /// `erase_gap_ms` through `pause` after each Backspace that went out. Refused whole as `word` says; true when any
    /// of it reached the app.
    pub fn type_word(
        &mut self,
        io: &Io<'_>,
        erase: u16,
        text: &Redacted<String>,
        pause: &dyn Pause,
    ) -> bool {
        let Some(mut word) = self.word(io, erase, text) else {
            return false;
        };
        drive(|| self.part(io, &mut word), pause)
    }

    /// The picked word of `erase` Backspaces, then `text`, when it may go: refused while a shortcut latch is on or a
    /// dead key waits, for a hidden mark (`is_hidden`), past `text_cap`, or when it asks for nothing.
    #[must_use]
    pub fn word(&self, io: &Io<'_>, erase: u16, text: &Redacted<String>) -> Option<Word> {
        self.takes(io, erase, text.expose()).then(|| Word {
            erase,
            text: Some(text.clone()),
            any: false,
        })
    }

    /// Sends `word`'s next part: one Backspace while any is left, then the gap; else its text, unless a shortcut latch
    /// or a dead key came meanwhile. Each part is told once it reached the app, and a Backspace not taken whole ends
    /// the word.
    pub fn part(&mut self, io: &Io<'_>, word: &mut Word) -> Part {
        if word.erase > 0 {
            return self.erase_one(io, word);
        }
        let text = word.text.take().filter(|t| !t.expose().is_empty());
        if let Some(text) = text.filter(|_| self.free(io)) {
            word.any |= self.type_text(io, &text);
        }
        self.end(word)
    }

    /// Ends `word` with nothing more sent; once any of it reached the app, the latches are used.
    pub fn end(&mut self, word: &mut Word) -> Part {
        word.erase = 0;
        word.text = None;
        if word.any {
            self.used();
        }
        Part::Done(word.any)
    }

    /// True when a word may go: it is `free`, the text holds no hidden mark, both parts fit `text_cap` in characters
    /// (`TypeWord::fits`), it asks for something, and the board has a key to erase with.
    fn takes(&self, io: &Io<'_>, erase: u16, text: &str) -> bool {
        let plain = !text.chars().any(is_hidden);
        let fits = TypeWord::fits(erase, text, self.cap);
        let some = erase > 0 || !text.is_empty();
        let can = erase == 0 || self.erase.is_some();
        self.free(io) && plain && fits && some && can
    }

    /// True when no shortcut latch is on (Ctrl, Alt, Win, or `AltGr` as Right Alt on a layout with no `AltGr`
    /// characters) and no dead key waits.
    fn free(&self, io: &Io<'_>) -> bool {
        !LegendMods::shortcut(&self.latches, &io.table.keys) && !self.dead
    }

    /// Taps one Backspace of `word`, told once it reached the app; then the gap, or the end when not taken whole.
    fn erase_one(&mut self, io: &Io<'_>, word: &mut Word) -> Part {
        let Some(key) = self.erase else {
            return self.end(word);
        };
        word.erase -= 1;
        let went = io.injector.tap(key);
        if reached(went) {
            self.pieces.push(Piece::Erase);
            word.any = true;
        }
        if went.is_err() {
            return self.end(word);
        }
        Part::Gap(self.gap)
    }

    /// Types `text` by the route a character key takes now, told once any of it reached the app; true then.
    fn type_text(&mut self, io: &Io<'_>, text: &Redacted<String>) -> bool {
        let by = text_route(&self.apps, || io.injector.target_app());
        let went = reached(io.injector.text(text, by));
        if went {
            self.pieces.push(Piece::Text(text.clone()));
        }
        went
    }
}
