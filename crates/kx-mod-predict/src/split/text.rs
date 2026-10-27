//! Sentences and their words for the pack builder (feature `build`): sentences end at the manifest's marks, and a
//! chain runs only across spaces and elisions.

use std::borrow::Cow;

use super::{Edges, edges, normalise, places, spans, split_word};

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

impl Splitter<'_> {
    /// `text` with every apostrophe form made the packs' own; borrowed when it holds none.
    #[must_use]
    pub fn normalise<'t>(&self, text: &'t str) -> Cow<'t, str> {
        normalise(text, self.apostrophe)
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
        let mut pieces = Vec::new();
        for (k, &(start, end)) in spans.iter().enumerate() {
            let (free, spaced) = edges(sentence, &spans, k);
            let raw = Raw {
                text: sentence.get(start..end).unwrap_or_default(),
                first: k == 0,
                free,
                joined: k > 0 && spaced,
            };
            pieces.clear();
            split_word(raw.text, self.apostrophe, knows, &mut pieces);
            emit(&raw, &pieces, each);
        }
    }
}

/// Gives `each` the tokens of `raw`'s `pieces`, placed by the shared rule.
fn emit(raw: &Raw<'_>, pieces: &[(&str, bool)], each: &mut dyn FnMut(Token<'_>)) {
    let placed = pieces.iter().zip(places(pieces, raw.free, raw.joined));
    for (j, (&(text, _), place)) in placed.enumerate() {
        each(Token {
            text,
            first: raw.first && j == 0,
            alone: place.alone,
            joined: place.joined,
        });
    }
}
