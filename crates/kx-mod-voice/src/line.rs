//! A typed line of the key page (ADR-0049): letters go in at its caret, which a click places and Left and Right move;
//! it reads left to right in any language, and the key's hidden line steps one dot per character. It reserves its room
//! at once and is wiped when dropped.

use std::fmt;

use kx_module_api::caret;
use kx_module_api::{Redacted, Secret, Zeroizing, is_hidden};

/// A line of letters with no hidden mark, taking them until it holds more than `max` bytes.
pub struct Line {
    text: Zeroizing<String>,
    /// The caret, in bytes, on a char boundary; it tells a secret's length, so `Debug` never shows it.
    at: usize,
    max: usize,
    /// Drawn as dots, one per character.
    hidden: bool,
}

impl Line {
    /// An empty line of at most `max` bytes and one letter more, so a text past its limit is refused at Save with its
    /// own reason; `hidden` for the key's dots.
    #[must_use]
    pub fn new(max: usize, hidden: bool) -> Self {
        let room = max.saturating_add(char::MAX_LEN_UTF8);
        Self {
            text: Zeroizing::new(String::with_capacity(room)),
            at: 0,
            max,
            hidden,
        }
    }

    /// A shown line holding `text` as far as it takes, such as a kept address, with the caret at its end.
    #[must_use]
    pub fn with(text: &str, max: usize) -> Self {
        let mut line = Self::new(max, false);
        line.write(text);
        line
    }

    /// Types `text` at the caret, letter by letter: a hidden mark never goes in, nor any letter once the line holds
    /// more than its most, so it never reallocates. True when one went in.
    pub fn write(&mut self, text: &str) -> bool {
        let mut took = false;
        for c in text.chars() {
            if is_hidden(c) || self.text.len() > self.max {
                continue;
            }
            self.text.insert(self.at, c);
            self.at += c.len_utf8();
            took = true;
        }
        took
    }

    /// Backspace: removes the character before the caret; false when none is there.
    pub fn erase(&mut self) -> bool {
        let to = caret::dot_back(&self.text, self.at);
        self.cut(to, self.at)
    }

    /// Delete: removes the letter after the caret, or one dot on a hidden line; false when none is there.
    pub fn delete(&mut self) -> bool {
        let to = if self.hidden {
            caret::dot_on(&self.text, self.at)
        } else {
            caret::on(&self.text, self.at)
        };
        self.cut(self.at, to)
    }

    /// Moves the caret one letter, or one dot on a hidden line, on when `on`, else back; false when it stays.
    pub fn step(&mut self, on: bool) -> bool {
        let to = match (self.hidden, on) {
            (true, true) => caret::dot_on(&self.text, self.at),
            (true, false) => caret::dot_back(&self.text, self.at),
            (false, true) => caret::on(&self.text, self.at),
            (false, false) => caret::back(&self.text, self.at),
        };
        self.go(to)
    }

    /// Puts the caret `at` into what the line draws: UTF-16 units, or dots on a hidden line; false when it stays.
    pub fn place(&mut self, at: u32) -> bool {
        let to = if self.hidden {
            caret::of_dots(&self.text, at)
        } else {
            caret::of_units(&self.text, at)
        };
        self.go(to)
    }

    /// The caret in what the line draws: UTF-16 units, or dots on a hidden line.
    #[must_use]
    pub fn caret(&self) -> u32 {
        if self.hidden {
            caret::dots(&self.text, self.at)
        } else {
            caret::units(&self.text, self.at)
        }
    }

    /// How many characters it holds, which a hidden line draws as as many dots.
    #[must_use]
    pub fn count(&self) -> u32 {
        caret::dots(&self.text, self.text.len())
    }

    /// True while it holds nothing.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.text.is_empty()
    }

    /// The text typed.
    #[must_use]
    pub fn text(&self) -> &str {
        &self.text
    }

    /// The text typed, moved out unread as a secret.
    #[must_use]
    pub fn into_secret(self) -> Secret {
        Redacted::new(self.text)
    }

    /// Removes the bytes from `from` to `to`, leaving the caret at `from`; false when none.
    fn cut(&mut self, from: usize, to: usize) -> bool {
        if from >= to {
            return false;
        }
        self.text.replace_range(from..to, "");
        self.at = from;
        true
    }

    /// Moves the caret to `to`; false when it stays.
    fn go(&mut self, to: usize) -> bool {
        let moved = to != self.at;
        self.at = to;
        moved
    }

    /// The bytes it holds room for.
    #[cfg(test)]
    fn room(&self) -> usize {
        self.text.capacity()
    }
}

/// A copy with the same room, so it too never reallocates.
impl Clone for Line {
    fn clone(&self) -> Self {
        let mut copy = Self::new(self.max, self.hidden);
        copy.text.push_str(&self.text);
        copy.at = self.at;
        copy
    }
}

/// Never shows the text, the caret or the most: each may tell a secret's length.
impl fmt::Debug for Line {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Line")
            .field("text", &Redacted::new(()))
            .field("hidden", &self.hidden)
            .finish_non_exhaustive()
    }
}

#[cfg(test)]
mod tests;
