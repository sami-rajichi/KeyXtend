//! The search line and its caret: Left and Right move it while the line draws text, a click places it, and letters,
//! chips, Backspace and Delete act at it; the keyboard hears the text before it (ADR-0040).

use kx_module_api::Redacted;
use kx_module_api::caret::{self, Side};
use kx_module_api::ports::PanelLine;

use super::{Do, Mode, Panel};
use crate::entry::flat;

impl Panel {
    /// The line the page shows, which the keyboard aims by (ADR-0040): on the list with no question, the search up to
    /// its caret.
    #[must_use]
    pub fn line(&self) -> PanelLine {
        match self.mode {
            Mode::List if !self.asking() => {
                let before = caret::before(self.query.expose(), self.at());
                PanelLine::Search(Redacted::new(before.to_owned()))
            }
            _ => PanelLine::None,
        }
    }

    /// The search's caret, in the UTF-16 units the line draws before it.
    #[must_use]
    pub fn caret(&self) -> u32 {
        caret::units(self.query.expose(), self.at())
    }

    /// Left or Right on the list: an empty line switches filters, and one with text moves the caret, stopping at its
    /// ends.
    pub(super) fn arrow(&mut self, side: Side) -> Do {
        let line = self.query.expose();
        if caret::empty(line) {
            return self.next_filter(side == Side::Right);
        }
        self.go_to(caret::arrow(line, self.at(), side))
    }

    /// A click on the line puts the caret at `at` units of what it draws, on a grapheme's edge.
    pub(super) fn place(&mut self, at: u32) -> Do {
        self.go_to(caret::of_units(self.query.expose(), at))
    }

    /// Erases up to `erase` characters before the caret and adds `text` at it, one change, with the caret after it.
    pub(super) fn write(&mut self, erase: u16, text: &str) -> Do {
        let line = self.query.expose().clone();
        let (head, tail) = line.split_at(self.at());
        let mut head = head.to_owned();
        for _ in 0..erase {
            head.pop();
        }
        head.push_str(text);
        self.join(&head, tail)
    }

    /// Removes the drawn grapheme after the caret; nothing when none is left.
    pub(super) fn delete(&mut self) -> Do {
        let line = self.query.expose().clone();
        let at = self.at();
        let tail = line.get(caret::on(&line, at)..).unwrap_or_default();
        self.join(caret::before(&line, at), tail)
    }

    /// Empties the line.
    pub(super) fn clear(&mut self) -> Do {
        self.search(String::new(), 0)
    }

    /// The caret as a byte offset, on a char boundary of the search.
    fn at(&self) -> usize {
        caret::clamped(self.query.expose(), *self.caret.expose())
    }

    /// Moves the caret to `to`, which changes neither the results nor the highlight.
    fn go_to(&mut self, to: usize) -> Do {
        if to == self.at() {
            return Do::Stay;
        }
        self.caret = Redacted::new(to);
        Do::Show
    }

    /// Makes the line `head` then `tail`, flattened and cut to `query_chars` characters; the caret stays after `head`,
    /// as far as it reaches.
    fn join(&mut self, head: &str, tail: &str) -> Do {
        let chars = self.cfg.query_chars;
        let whole: String = flat(&format!("{head}{tail}")).take(chars).collect();
        let at = flat(head).take(chars).map(char::len_utf8).sum();
        self.search(whole, at)
    }

    /// Sets the search to `query` and its caret to `at`; a new text highlights the top row, and the same text and caret
    /// change nothing.
    fn search(&mut self, query: String, at: usize) -> Do {
        if query == *self.query.expose() {
            return self.go_to(at);
        }
        self.query = Redacted::new(query);
        self.caret = Redacted::new(at);
        self.top();
        Do::Show
    }
}
