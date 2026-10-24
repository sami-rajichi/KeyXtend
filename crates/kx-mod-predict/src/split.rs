//! Splits text into words the way the packs spell them, one rule for the pack builder and the tracker: a word is a
//! run of letters, digits and marks joined by apostrophes, and an apostrophe keeps a known whole word (`quelqu'un`),
//! else splits at a known elision (`l'homme`) or anywhere (`Muiriel's`).

use crate::script;

#[cfg(feature = "build")]
mod text;

#[cfg(feature = "build")]
pub use text::{Splitter, Token};

/// True when `c` belongs inside a word: a letter, a digit, or a mark that joins any script.
#[must_use]
pub fn is_word_char(c: char) -> bool {
    !c.is_whitespace() && (c.is_alphanumeric() || script::joins_any(c))
}

/// The byte spans of the words of `text`: runs of word characters, each with the apostrophe that follows one of
/// them, so a trailing apostrophe stays in its span for `split_word` to drop.
#[must_use]
pub fn spans(text: &str, apostrophe: char) -> Vec<(usize, usize)> {
    let mut out = Vec::new();
    let mut start = None;
    let mut after_letter = false;
    for (at, c) in text.char_indices() {
        if is_word_char(c) {
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

#[cfg(all(test, feature = "build"))]
mod tests;
