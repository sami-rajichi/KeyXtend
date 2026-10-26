//! The word tracker (spec §6 "Knowing the word"): it follows the keyboard's own pieces to know the word being typed
//! and the words before it, hands out finished words and emails, and asks for a read when it loses track.

use std::fmt;

use kx_module_api::Redacted;
use kx_module_api::ports::{FieldChange, Piece};

use crate::split::{Place, edges, places, spans, split_word};

mod show;
mod token;

pub use show::{EmailAsk, Plan, Query, Reading, Show};
pub(crate) use token::could_learn;
use token::{Token, apostrophe, learnable};

/// What the tracker needs from the settings; the module fills it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TrackConfig {
    /// The most words kept before the current one (`[predict] context_words`).
    pub context_words: usize,
    /// The longest word that gets guesses and is learned, in characters (`[predict] max_word`).
    pub max_word: usize,
}

/// What a step hands to the module.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Told {
    /// A finished word to learn.
    Learn {
        /// The word, as typed with the packs' apostrophe.
        word: Redacted<String>,
        /// The words before it, oldest first.
        context: Redacted<Vec<String>>,
    },
    /// A finished token that looks like an email.
    Email(Redacted<String>),
    /// The words before the cursor are unknown: read the box.
    Read,
}

/// What the tracker knows of the words before the token.
#[derive(Clone, PartialEq, Eq)]
enum State {
    /// Nothing, until a read or a finished word tells it.
    Unknown,
    /// The words before the token, oldest first, at most `context_words`.
    Known(Vec<String>),
}

/// Where a character came from.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Source {
    /// The keyboard typed it.
    Typed,
    /// A read of the box gave it.
    Read,
}

/// Follows what the keyboard typed in the focused box.
pub struct Tracker {
    /// The settings.
    cfg: TrackConfig,
    /// What it knows of the words before the token.
    state: State,
    /// The characters since the last space or reset.
    token: Token,
    /// The focused box, once told.
    field: Option<u64>,
}

impl fmt::Debug for Tracker {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let known = self.context().map(<[String]>::len);
        f.debug_struct("Tracker")
            .field("known", &known)
            .field("units", &self.token.units())
            .finish_non_exhaustive()
    }
}

impl Tracker {
    /// A tracker that knows nothing yet, so the module reads the box first.
    #[must_use]
    pub fn new(cfg: TrackConfig) -> Self {
        Self {
            cfg,
            state: State::Unknown,
            token: Token::default(),
            field: None,
        }
    }

    /// Follows one piece the keyboard typed; `knows` says which words the language holds, to split apostrophes.
    pub fn typed(&mut self, piece: &Piece, knows: &dyn Fn(&str) -> bool) -> Vec<Told> {
        let mut told = Vec::new();
        match piece {
            Piece::Text(text) => {
                for c in text.expose().chars() {
                    self.feed(c, knows, Source::Typed, &mut told);
                }
            }
            Piece::Erase if !self.token.pop() => {
                self.state = State::Unknown;
                told.push(Told::Read);
            }
            Piece::Erase => {}
            Piece::Advance => {
                self.end(knows, Source::Typed, &mut told);
                self.state = State::Known(Vec::new());
            }
            Piece::Break => return self.reset(),
        }
        told
    }

    /// Ends the word unlearned and forgets the context, for a break, a click outside our windows, a new box or a
    /// layout switch; the module then reads the box.
    pub fn reset(&mut self) -> Vec<Told> {
        self.state = State::Unknown;
        self.token = Token::default();
        vec![Told::Read]
    }

    /// Resets when focus moved to a new box; the same box again changes nothing.
    pub fn focus(&mut self, change: FieldChange) -> Vec<Told> {
        if !self.leaves(change) {
            return Vec::new();
        }
        self.field = Some(change.box_id);
        self.reset()
    }

    /// True when `change` moves focus to another box than the one followed.
    #[must_use]
    pub fn leaves(&self, change: FieldChange) -> bool {
        self.field != Some(change.box_id)
    }

    /// The open token's email, told once: a full address with some of it typed, as the user leaves it or a domain chip
    /// completes it. Its end then tells it no more.
    pub fn email(&mut self) -> Option<Redacted<String>> {
        let email = self.token.email()?.to_owned();
        self.token.tell(&email);
        Some(Redacted::new(email))
    }

    /// Takes a read of the words before the cursor: they fill the context, and a last word with no space after it
    /// stays open, marked as read. Nothing read is learned; `None` (the app does not tell) changes nothing.
    pub fn read(&mut self, text: Option<&str>, knows: &dyn Fn(&str) -> bool) {
        let Some(text) = text else {
            return;
        };
        self.state = State::Unknown;
        self.token = Token::default();
        let mut unused = Vec::new();
        for c in text.chars() {
            self.feed(c, knows, Source::Read, &mut unused);
        }
        self.token.mark_read();
        if self.state == State::Unknown {
            self.state = State::Known(Vec::new());
        }
    }

    /// What the lane may offer now.
    #[must_use]
    pub fn show(&self, knows: &dyn Fn(&str) -> bool) -> Show {
        show::show(&self.token, self.before(), self.cfg, knows)
    }

    /// True while it holds a token or knows a word before it, so the module wants to hear clicks.
    #[must_use]
    pub fn holds(&self) -> bool {
        !self.token.is_empty() || !self.before().is_empty()
    }

    /// The words before the token, oldest first; `None` while they are unknown.
    #[must_use]
    pub fn context(&self) -> Option<&[String]> {
        match &self.state {
            State::Unknown => None,
            State::Known(words) => Some(words),
        }
    }

    /// How many characters the token holds.
    #[cfg(test)]
    fn units(&self) -> usize {
        self.token.units()
    }

    /// The words before the token, or none while they are unknown.
    fn before(&self) -> &[String] {
        self.context().unwrap_or_default()
    }

    /// Takes one character: a space ends the token, anything else joins it.
    fn feed(&mut self, c: char, knows: &dyn Fn(&str) -> bool, from: Source, told: &mut Vec<Told>) {
        if c.is_whitespace() {
            self.end(knows, from, told);
        } else {
            self.token.push(c);
        }
    }

    /// Ends the token whole, as a space or Enter does; only a token with typed characters hands anything out.
    fn end(&mut self, knows: &dyn Fn(&str) -> bool, from: Source, told: &mut Vec<Told>) {
        let token = std::mem::take(&mut self.token);
        if token.is_empty() {
            return;
        }
        let mut before = match std::mem::replace(&mut self.state, State::Unknown) {
            State::Unknown => Vec::new(),
            State::Known(words) => words,
        };
        let told = (from == Source::Typed && token.typed()).then_some(told);
        self.finish(&token, &mut before, knows, told);
        self.state = State::Known(before);
    }

    /// Ends `token` after the words `before`: an email is told; otherwise each word is split and placed by the
    /// build's rule, and a mark after the last word clears the context. `told` is `None` for a read.
    fn finish(
        &self,
        token: &Token,
        before: &mut Vec<String>,
        knows: &dyn Fn(&str) -> bool,
        mut told: Option<&mut Vec<Told>>,
    ) {
        if token.has_at() {
            finish_email(token, before, told);
            return;
        }
        let normal = token.normal();
        let found = spans(&normal, apostrophe());
        let mut pieces = Vec::new();
        for (k, &(start, end)) in found.iter().enumerate() {
            let (free, joined) = edges(&normal, &found, k);
            pieces.clear();
            let word = normal.get(start..end).unwrap_or_default();
            split_word(word, apostrophe(), knows, &mut pieces);
            for (&(piece, _), place) in pieces.iter().zip(places(&pieces, free, joined)) {
                self.take(piece, place, before, told.as_deref_mut());
            }
        }
        if found.last().is_none_or(|&(_, end)| end < normal.len()) {
            before.clear();
        }
    }

    /// Learns `piece` after `before` when it stands alone and may be learned; a piece that does not join clears the
    /// context first, and one that is not learned ends it.
    fn take(
        &self,
        piece: &str,
        place: Place,
        before: &mut Vec<String>,
        told: Option<&mut Vec<Told>>,
    ) {
        if !place.joined {
            before.clear();
        }
        if !place.alone || !learnable(piece, self.cfg.max_word) {
            before.clear();
            return;
        }
        if let Some(told) = told {
            let (word, context) = (
                Redacted::new(piece.to_owned()),
                Redacted::new(before.clone()),
            );
            told.push(Told::Learn { word, context });
        }
        push_capped(before, piece.to_owned(), self.cfg.context_words);
    }
}

/// Tells the email `token` holds, unless told already, and clears the context, as an email is never a word.
fn finish_email(token: &Token, before: &mut Vec<String>, told: Option<&mut Vec<Told>>) {
    if let (Some(told), Some(email)) = (told, token.email()) {
        told.push(Told::Email(Redacted::new(email.to_owned())));
    }
    before.clear();
}

/// Adds `word` after `words`, dropping the oldest past `cap`.
fn push_capped(words: &mut Vec<String>, word: String, cap: usize) {
    words.push(word);
    let over = words.len().saturating_sub(cap);
    words.drain(..over);
}

#[cfg(test)]
mod props;
#[cfg(test)]
mod tests;
