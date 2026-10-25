//! A picked word (ADR-0033 decision 2): the keyboard stays the only typist, so prediction's word erases and types
//! through the same route, latches and `text_cap` as a key, and is told like keys.

use super::checks::reached;
use super::{Io, Typing};
use crate::legend::LegendMods;
use crate::plan::text_route;
use kx_module_api::Redacted;
use kx_module_api::ports::Piece;

impl Typing {
    /// Erases `erase` characters with Backspace, then types `text` by the route a character key takes now; once any
    /// of it reached the app, the latches are used. Refused whole while a shortcut latch is on or a dead key waits, for
    /// a control character or past `text_cap`; true when any of it reached the app.
    pub fn type_word(&mut self, io: &Io<'_>, erase: u16, text: &Redacted<String>) -> bool {
        if !self.takes(io, erase, text.expose()) {
            return false;
        }
        let before = self.pieces.len();
        if self.erase_all(io, erase) && !text.expose().is_empty() {
            self.type_text(io, text);
        }
        let any = self.pieces.len() > before;
        if any {
            self.used();
        }
        any
    }

    /// True when a word may go: no shortcut latch is on (Ctrl, Alt, Win, or `AltGr` as Right Alt on a layout with no
    /// `AltGr` characters), no dead key waits, the text holds no control character, both parts fit `text_cap` in
    /// characters, it asks for something, and the board has a key to erase with.
    fn takes(&self, io: &Io<'_>, erase: u16, text: &str) -> bool {
        let free = !LegendMods::shortcut(&self.latches, &io.table.keys) && !self.dead;
        let plain = !text.chars().any(char::is_control);
        let fits = erase <= self.cap && text.chars().count() <= usize::from(self.cap);
        let some = erase > 0 || !text.is_empty();
        let can = erase == 0 || self.erase.is_some();
        free && plain && fits && some && can
    }

    /// Taps Backspace `erase` times, each told once it reached the app; false at the first tap not taken whole.
    fn erase_all(&mut self, io: &Io<'_>, erase: u16) -> bool {
        let Some(key) = self.erase else {
            return erase == 0;
        };
        for _ in 0..erase {
            let went = io.injector.tap(key);
            if reached(went) {
                self.pieces.push(Piece::Erase);
            }
            if went.is_err() {
                return false;
            }
        }
        true
    }

    /// Types `text` by the route a character key takes now, told once any of it reached the app.
    fn type_text(&mut self, io: &Io<'_>, text: &Redacted<String>) {
        let by = text_route(&self.apps, || io.injector.target_app());
        if reached(io.injector.text(text, by)) {
            self.pieces.push(Piece::Text(text.clone()));
        }
    }
}
