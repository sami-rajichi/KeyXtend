//! The token: every character typed since the last space or reset, how much of it came from a read, and the open
//! word at its end.

use std::borrow::Cow;

use crate::fold;
use crate::script;
use crate::split::{normalise, spans};

/// The packs' apostrophe; a broken fold table, which its tests rule out, leaves NUL, which no word holds.
pub(super) fn apostrophe() -> char {
    fold::apostrophe().unwrap_or_default()
}

/// The characters since the last space or reset.
#[derive(Clone, Default, PartialEq, Eq)]
pub(super) struct Token {
    /// The characters as typed or read.
    text: String,
    /// How many of its first characters came from a read.
    read: usize,
}

impl Token {
    /// Adds a character.
    pub(super) fn push(&mut self, c: char) {
        self.text.push(c);
    }

    /// Removes the last character; false when there was none.
    pub(super) fn pop(&mut self) -> bool {
        let popped = self.text.pop().is_some();
        self.read = self.read.min(self.units());
        popped
    }

    /// True when it holds nothing.
    pub(super) fn is_empty(&self) -> bool {
        self.text.is_empty()
    }

    /// Its characters, as typed or read.
    pub(super) fn text(&self) -> &str {
        &self.text
    }

    /// How many characters it holds, as an erase counts them.
    pub(super) fn units(&self) -> usize {
        self.text.chars().count()
    }

    /// Marks every character it holds as read.
    pub(super) fn mark_read(&mut self) {
        self.read = self.units();
    }

    /// True when some of it came from a read.
    pub(super) fn read(&self) -> bool {
        self.read > 0
    }

    /// True when some of it was typed.
    pub(super) fn typed(&self) -> bool {
        self.units() > self.read
    }

    /// Its text with every apostrophe form made the packs' own, character for character.
    pub(super) fn normal(&self) -> Cow<'_, str> {
        normalise(&self.text, apostrophe())
    }

    /// Its last `n` characters, as typed or read.
    pub(super) fn last(&self, n: usize) -> &str {
        let skip = self.units().saturating_sub(n);
        let at = self
            .text
            .char_indices()
            .nth(skip)
            .map_or(self.text.len(), |(at, _)| at);
        self.text.get(at..).unwrap_or_default()
    }

    /// The `n` `at` signs that end it when all were typed, not read.
    pub(super) fn typed_run(&self, at: char, n: usize) -> Option<&str> {
        let run = self.last(n);
        let ours = self.units().saturating_sub(n) >= self.read;
        let all = n > 0 && run.chars().filter(|&c| c == at).count() == n;
        (ours && all).then_some(run)
    }
}

/// What a normalised token holds: its open word and what comes before it.
pub(super) struct Parts<'t> {
    /// The word at the token's end; empty when a mark ends the token.
    pub(super) word: &'t str,
    /// True when anything comes before it in the token, so no chain runs into it.
    pub(super) marked: bool,
}

/// The parts of a normalised token, by the packs' word rule.
pub(super) fn parts(normal: &str) -> Parts<'_> {
    let found = spans(normal, apostrophe());
    match found.last() {
        Some(&(start, end)) if end == normal.len() => Parts {
            word: normal.get(start..).unwrap_or_default(),
            marked: start > 0,
        },
        _ => Parts {
            word: "",
            marked: !normal.is_empty(),
        },
    }
}

/// True when `word` may get guesses and be learned: it holds no digit, one script that spaces its words, and at most
/// `max_word` characters. Unspaced scripts wait for the CJK input mode, as spaces cannot end their words.
pub(super) fn learnable(word: &str, max_word: usize) -> bool {
    !word.is_empty()
        && !word.chars().any(char::is_numeric)
        && script::single(word)
        && script::of(word).is_none_or(|s| !s.gap().is_empty())
        && word.chars().count() <= max_word
}
