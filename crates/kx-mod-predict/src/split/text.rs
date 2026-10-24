//! Sentences and their words for the pack builder (feature `build`): sentences end at the manifest's marks, and a
//! chain runs only across spaces and elisions.

use std::borrow::Cow;

use super::{spans, split_word};
use crate::fold::is_apostrophe;

/// One word of a sentence.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Token<'a> {
    /// Its text, with the packs' apostrophe.
    pub text: &'a str,
    /// True for the sentence's first word.
    pub first: bool,
    /// True when no hyphen, mark or split apostrophe glues it to a neighbour.
    pub alone: bool,
    /// True when a chain may run into it from the word before: only spaces or an elision part them.
    pub joined: bool,
}

/// How text splits: where sentences end, and the apostrophe the packs spell.
#[derive(Debug, Clone, Copy)]
pub struct Splitter<'a> {
    /// The characters that end a sentence.
    pub ends: &'a [char],
    /// The apostrophe every form of it becomes.
    pub apostrophe: char,
}

/// One word of a sentence before its apostrophes are split, with what parts it from its neighbours.
struct Raw<'a> {
    /// Its text.
    text: &'a str,
    /// True for the sentence's first word.
    first: bool,
    /// Whether a space or the sentence's edge comes before and after it.
    free: Edges,
    /// True when only spaces part it from the word before.
    joined: bool,
}

/// Whether each side of a word is free: a space or the sentence's edge is there, not a hyphen or a mark.
#[derive(Debug, Clone, Copy)]
struct Edges {
    /// Before the word.
    before: bool,
    /// After the word.
    after: bool,
}

impl Splitter<'_> {
    /// `text` with every apostrophe form made the packs' own; borrowed when it holds none.
    #[must_use]
    pub fn normalise<'t>(&self, text: &'t str) -> Cow<'t, str> {
        let other = |c: char| c != self.apostrophe && is_apostrophe(c);
        if !text.chars().any(other) {
            return Cow::Borrowed(text);
        }
        let swap = |c: char| if other(c) { self.apostrophe } else { c };
        Cow::Owned(text.chars().map(swap).collect())
    }

    /// The sentences of a normalised `text`: an end mark ends one only before a space or the text's end, so `U.S.`
    /// and `3.14` stay inside theirs.
    pub fn sentences<'t>(&self, text: &'t str) -> impl Iterator<Item = &'t str> {
        let mut chars = text.char_indices().peekable();
        let (mut from, mut done) = (0, false);
        std::iter::from_fn(move || {
            while let Some((at, c)) = chars.next() {
                let free = chars.peek().is_none_or(|&(_, n)| n.is_whitespace());
                if self.ends.contains(&c) && (free || c.is_whitespace()) {
                    let sentence = text.get(from..at).unwrap_or_default();
                    from = at + c.len_utf8();
                    return Some(sentence);
                }
            }
            let last = (!done).then(|| text.get(from..).unwrap_or_default());
            done = true;
            last
        })
    }

    /// Calls `each` with every word of `sentence`, splitting its apostrophes where `knows` says.
    pub fn words(
        &self,
        sentence: &str,
        knows: &dyn Fn(&str) -> bool,
        each: &mut dyn FnMut(Token<'_>),
    ) {
        let spans = spans(sentence, self.apostrophe);
        let text = |from: usize, to: usize| sentence.get(from..to).unwrap_or_default();
        let mut pieces = Vec::new();
        for (k, &(start, end)) in spans.iter().enumerate() {
            let before_from = k
                .checked_sub(1)
                .and_then(|p| spans.get(p))
                .map_or(0, |s| s.1);
            let after_to = spans.get(k + 1).map_or(sentence.len(), |s| s.0);
            let (before, after) = (text(before_from, start), text(end, after_to));
            let raw = Raw {
                text: text(start, end),
                first: k == 0,
                free: Edges {
                    before: k == 0 || before.chars().any(char::is_whitespace),
                    after: k + 1 == spans.len() || after.chars().any(char::is_whitespace),
                },
                joined: k > 0 && before.chars().all(char::is_whitespace),
            };
            pieces.clear();
            split_word(raw.text, self.apostrophe, knows, &mut pieces);
            emit(&raw, &pieces, each);
        }
    }
}

/// Gives `each` the tokens of `raw`'s `pieces`, with what parts each from its neighbours.
fn emit(raw: &Raw<'_>, pieces: &[(&str, bool)], each: &mut dyn FnMut(Token<'_>)) {
    let last = pieces.len().saturating_sub(1);
    for (j, &(text, elided)) in pieces.iter().enumerate() {
        let elided_before = j.checked_sub(1).and_then(|p| pieces.get(p)).map(|p| p.1);
        let left = elided_before.unwrap_or(raw.free.before);
        let right = if j == last { raw.free.after } else { elided };
        each(Token {
            text,
            first: raw.first && j == 0,
            alone: left && right,
            joined: elided_before.unwrap_or(raw.joined),
        });
    }
}
