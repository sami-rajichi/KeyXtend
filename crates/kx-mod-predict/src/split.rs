//! Splits text into words the way the packs spell them, one rule for the pack builder and the tracker: a word is a
//! run of letters, digits and marks joined by apostrophes or joiners, and an apostrophe keeps a known whole word
//! (`quelqu'un`), else splits at a known elision (`l'homme`) or anywhere (`Muiriel's`).

use std::borrow::Cow;

use kx_module_api::is_hidden;

use crate::fold::is_apostrophe;
use crate::script;

#[cfg(feature = "build")]
mod text;

#[cfg(feature = "build")]
pub use text::{Splitter, Token};

/// Whether each side of a word is free: a space or the text's edge is there, not a hyphen or a mark.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Edges {
    /// Before the word.
    pub before: bool,
    /// After the word.
    pub after: bool,
}

/// Where one apostrophe piece of a word stands.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Place {
    /// Both its sides are free or parted by an elision, so it counts as a word on its own.
    pub alone: bool,
    /// A chain may run into it from the word before: only spaces or an elision part them.
    pub joined: bool,
}

/// The edges of word `k` of `spans` in `text`, and whether only spaces part it from the word before or the text's
/// start. The first word's start and the last word's end are free; any other side is free when its gap holds a space.
#[must_use]
pub fn edges(text: &str, spans: &[(usize, usize)], k: usize) -> (Edges, bool) {
    let gap = |from: usize, to: usize| text.get(from..to).unwrap_or_default();
    let (start, end) = spans.get(k).copied().unwrap_or_default();
    let before_from = k
        .checked_sub(1)
        .and_then(|p| spans.get(p))
        .map_or(0, |s| s.1);
    let after_to = spans.get(k + 1).map_or(text.len(), |s| s.0);
    let (before, after) = (gap(before_from, start), gap(end, after_to));
    let free = Edges {
        before: k == 0 || before.chars().any(char::is_whitespace),
        after: k + 1 == spans.len() || after.chars().any(char::is_whitespace),
    };
    (free, before.chars().all(char::is_whitespace))
}

/// The place of each of a word's `pieces` (from `split_word`), given the word's `free` edges and whether it `joined`
/// the word before: an elision frees the side it parts and lets the chain through.
pub fn places(pieces: &[(&str, bool)], free: Edges, joined: bool) -> impl Iterator<Item = Place> {
    let last = pieces.len().saturating_sub(1);
    pieces.iter().enumerate().map(move |(j, &(_, elided))| {
        let elided_before = j.checked_sub(1).and_then(|p| pieces.get(p)).map(|p| p.1);
        let left = elided_before.unwrap_or(free.before);
        let right = if j == last { free.after } else { elided };
        Place {
            alone: left && right,
            joined: elided_before.unwrap_or(joined),
        }
    })
}

/// `text` with every apostrophe form made `apostrophe`; borrowed when it holds none.
#[must_use]
pub fn normalise(text: &str, apostrophe: char) -> Cow<'_, str> {
    let other = |c: char| c != apostrophe && is_apostrophe(c);
    if !text.chars().any(other) {
        return Cow::Borrowed(text);
    }
    let swap = |c: char| if other(c) { apostrophe } else { c };
    Cow::Owned(text.chars().map(swap).collect())
}

/// True when `c` belongs inside a word: a letter, a digit, or a mark that joins any script, and never a hidden mark
/// (`kx_module_api::is_hidden`), such as a Hangul filler.
#[must_use]
pub fn is_word_char(c: char) -> bool {
    !c.is_whitespace() && !is_hidden(c) && (c.is_alphanumeric() || script::joins_any(c))
}

/// The byte spans of the words of `text`: runs of word characters, each with the apostrophe that follows one of
/// them, so a trailing apostrophe stays in its span for `split_word` to drop, and the joiners between two of them.
#[must_use]
pub fn spans(text: &str, apostrophe: char) -> Vec<(usize, usize)> {
    let mut out = Vec::new();
    let mut start = None;
    let mut after_letter = false;
    for (at, c) in text.char_indices() {
        if is_word_char(c) || (after_letter && joins(text, at, c)) {
            start.get_or_insert(at);
            after_letter = true;
        } else if c == apostrophe && after_letter {
            after_letter = false;
        } else {
            if let Some(from) = start.take() {
                out.push((from, at));
            }
            after_letter = false;
        }
    }
    if let Some(from) = start {
        out.push((from, text.len()));
    }
    out
}

/// True when `c`, at byte `at` of `text`, is a joiner that a word character follows, after any further joiners.
fn joins(text: &str, at: usize, c: char) -> bool {
    let mut after = text.get(at..).unwrap_or_default().chars();
    script::joins_inside(c)
        && after
            .find(|&n| !script::joins_inside(n))
            .is_some_and(is_word_char)
}

/// Splits `word` at its apostrophes into `out`, each piece with true when it is an elision: a whole word `knows`
/// holds stays (`quelqu'un`), then an elision it holds comes off (`l'`), else the apostrophe goes.
pub fn split_word<'w>(
    word: &'w str,
    apostrophe: char,
    knows: &dyn Fn(&str) -> bool,
    out: &mut Vec<(&'w str, bool)>,
) {
    let mut rest = word;
    while let Some(at) = rest.find(apostrophe) {
        let (elided, tail) = rest.split_at(at + apostrophe.len_utf8());
        if knows(rest) {
            break;
        } else if knows(elided) {
            out.push((elided, true));
        } else {
            out.push((rest.get(..at).unwrap_or_default(), false));
        }
        rest = tail;
        if rest.is_empty() {
            return;
        }
    }
    out.push((rest, false));
}

#[cfg(test)]
mod tests;
