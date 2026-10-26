//! The token: every character typed since the last space or reset, how much of it came from a read, and the open
//! word at its end.

use std::borrow::Cow;

use kx_module_api::Redacted;

use crate::email;
use crate::fold;
use crate::script::{self, Script};
use crate::split::{normalise, spans};

/// The packs' apostrophe; a broken fold table, which its tests rule out, leaves NUL, which no word holds.
pub(super) fn apostrophe() -> char {
    fold::apostrophe().unwrap_or_default()
}

/// An email a token tells: its address, and the one the same token told before, which it replaces.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Address {
    /// The address.
    pub email: Redacted<String>,
    /// The address told before from the same token, as when a domain chip landed; `None` for the first.
    pub over: Option<Redacted<String>>,
}

impl Address {
    /// `email`, replacing `over`.
    pub(super) fn of(email: &str, over: Option<&str>) -> Self {
        Self {
            email: Redacted::new(email.to_owned()),
            over: over.map(|o| Redacted::new(o.to_owned())),
        }
    }
}

/// The characters since the last space or reset.
#[derive(Clone, Default, PartialEq, Eq)]
pub(super) struct Token {
    /// The characters as typed or read.
    text: String,
    /// How many of its first characters came from a read.
    read: usize,
    /// The email already told from it, so its end tells that one no more.
    told: Option<String>,
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

    /// True when it holds the email at sign, so it is an email and never words.
    pub(super) fn has_at(&self) -> bool {
        email::at().is_some_and(|a| self.text.contains(a))
    }

    /// The email it holds, unless told already: the run `email::capture` takes as a full address, when its at sign was
    /// typed, not read.
    pub(super) fn email(&self) -> Option<&str> {
        let email = email::capture(&self.text).filter(|_| self.typed_at())?;
        (self.told.as_deref() != Some(email)).then_some(email)
    }

    /// True when its first at sign, the one `email::capture` reads an address around, was typed.
    fn typed_at(&self) -> bool {
        let sign = email::at().and_then(|a| self.text.find(a));
        let before = sign.and_then(|at| self.text.get(..at));
        before.is_some_and(|b| b.chars().count() >= self.read)
    }

    /// The email told from it last, if any.
    pub(super) fn told(&self) -> Option<&str> {
        self.told.as_deref()
    }

    /// Notes `email` as told, so `email` gives it no more.
    pub(super) fn tell(&mut self, email: &str) {
        self.told = Some(email.to_owned());
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

/// The parts of a normalised token, by the packs' word rule; joiners at its end stay in the open word, as letters may
/// follow them.
pub(super) fn parts(normal: &str) -> Parts<'_> {
    let found = spans(normal, apostrophe());
    let open = normal.trim_end_matches(script::joins_inside).len();
    match found.last() {
        Some(&(start, end)) if end == open => Parts {
            word: normal.get(start..).unwrap_or_default(),
            marked: start > 0,
        },
        _ => Parts {
            word: "",
            marked: !normal.is_empty(),
        },
    }
}

/// True when `word` may get guesses and be learned: it is `lettered` and holds at most `max_word` characters.
pub(super) fn learnable(word: &str, max_word: usize) -> bool {
    lettered(word) && word.chars().count() <= max_word
}

/// True when the learner may keep `word` it reads back: one whole word by the packs' rule, and `lettered`.
pub(crate) fn could_learn(word: &str) -> bool {
    spans(word, apostrophe()) == [(0, word.len())] && lettered(word)
}

/// True when `word` holds no digit and one script that spaces its words. Unspaced scripts wait for the CJK input
/// mode, as spaces cannot end their words.
fn lettered(word: &str) -> bool {
    !word.is_empty()
        && !word.chars().any(char::is_numeric)
        && script::single(word)
        && script::of(word).is_none_or(Script::spaced)
}
