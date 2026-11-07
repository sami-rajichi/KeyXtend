//! A panel line's caret: a byte offset on a char boundary, stepping over whole graphemes so a base letter and its
//! harakat move as one, and its place in what QML counts: UTF-16 units of the drawn text. A mask steps one dot, which is
//! one character, so it never tells where letters combine.

use crate::{is_hidden, is_rtl};
use unicode_segmentation::UnicodeSegmentation;

/// The side of the screen an arrow key points to.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Side {
    /// The Left key.
    Left,
    /// The Right key.
    Right,
}

/// `at` kept inside `text`, on the char boundary at or before it.
#[must_use]
pub fn clamped(text: &str, at: usize) -> usize {
    text.floor_char_boundary(at)
}

/// The part of `text` before the caret at `at`.
#[must_use]
pub fn before(text: &str, at: usize) -> &str {
    text.get(..clamped(text, at)).unwrap_or_default()
}

/// The start of the last drawn grapheme before `at`, past any hidden mark; `at` itself when none is left.
#[must_use]
pub fn back(text: &str, at: usize) -> usize {
    let at = clamped(text, at);
    let mut parts = before(text, at).grapheme_indices(true).rev();
    parts.find(|(_, part)| drawn(part)).map_or(at, |(i, _)| i)
}

/// The end of the first drawn grapheme after `at`, past any hidden mark; `at` itself when none is left.
#[must_use]
pub fn on(text: &str, at: usize) -> usize {
    let at = clamped(text, at);
    let mut ends = text.grapheme_indices(true).map(|(i, g)| (i + g.len(), g));
    ends.find(|(end, part)| *end > at && drawn(part))
        .map_or(at, |(end, _)| end)
}

/// Where the arrow on `side` takes the caret: back and on follow the text, so in a line that reads right to left Left
/// goes on, as Qt's logical move does.
#[must_use]
pub fn arrow(text: &str, at: usize, side: Side) -> usize {
    if (side == Side::Left) == is_rtl(text) {
        on(text, at)
    } else {
        back(text, at)
    }
}

/// True when `text` draws nothing, as hidden marks alone do.
#[must_use]
pub fn empty(text: &str) -> bool {
    !drawn(text)
}

/// The UTF-16 units QML draws before the caret at `at`; a hidden mark is not drawn.
#[must_use]
pub fn units(text: &str, at: usize) -> u32 {
    count(unit_len(before(text, at)))
}

/// The caret at `units` UTF-16 units of the drawn text, clamped, never inside a surrogate pair or a grapheme.
#[must_use]
pub fn of_units(text: &str, units: u32) -> usize {
    let limit = usize::try_from(units).unwrap_or(usize::MAX);
    let (mut seen, mut at) = (0_usize, 0);
    for (i, part) in text.grapheme_indices(true) {
        seen = seen.saturating_add(unit_len(part));
        if seen > limit {
            break;
        }
        at = i + part.len();
    }
    at
}

/// The dots a mask draws before the caret at `at`: one per character, hidden marks too.
#[must_use]
pub fn dots(text: &str, at: usize) -> u32 {
    count(before(text, at).chars().count())
}

/// The caret at `dots` dots of a mask, clamped; a dot is one character, so it may rest inside a grapheme.
#[must_use]
pub fn of_dots(text: &str, dots: u32) -> usize {
    let dots = usize::try_from(dots).unwrap_or(usize::MAX);
    text.char_indices().nth(dots).map_or(text.len(), |(i, _)| i)
}

/// The caret one dot back from `at`; `at` itself at the start.
#[must_use]
pub fn dot_back(text: &str, at: usize) -> usize {
    let at = clamped(text, at);
    let last = before(text, at).chars().next_back();
    last.map_or(at, |c| at - c.len_utf8())
}

/// The caret one dot on from `at`; `at` itself at the end.
#[must_use]
pub fn dot_on(text: &str, at: usize) -> usize {
    let at = clamped(text, at);
    let next = text.get(at..).and_then(|rest| rest.chars().next());
    next.map_or(at, |c| at + c.len_utf8())
}

/// True when `part` holds a character that shows.
fn drawn(part: &str) -> bool {
    part.chars().any(|c| !is_hidden(c))
}

/// The UTF-16 units of `part` that QML draws.
fn unit_len(part: &str) -> usize {
    let shown = part.chars().filter(|c| !is_hidden(*c));
    shown.map(char::len_utf16).sum()
}

/// `n` as the view's count, or the largest one.
fn count(n: usize) -> u32 {
    u32::try_from(n).unwrap_or(u32::MAX)
}

#[cfg(test)]
mod tests;
