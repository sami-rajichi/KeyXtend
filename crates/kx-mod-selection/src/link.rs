//! The Translate link (ADR-0041 decision 5, rulings G, J and L): the selection's text without hidden marks, cut at a
//! whole word so it fits both `translate_chars` and the site's whole-link cap, percent-encoded into the site's
//! pattern with the target's code.

mod sites;

pub use sites::{Site, SiteError, Sites, sites};

use std::fmt::Write as _;

use kx_module_api::ports::Link;
use kx_module_api::{caret, is_hidden};

use sites::{TEXT, TO};

/// The longest language tag `translate_to` or the site table may hold.
pub const TAG_CHARS: usize = 16;
/// What joins a tag's parts.
const TAG_JOIN: char = '-';
/// What may join the parts of a tag from the system, which may write `fr_FR`.
const OS_JOINS: [char; 2] = [TAG_JOIN, '_'];
/// The marks RFC 3986 keeps as they are, beside ASCII letters and digits; every other byte is encoded.
const UNRESERVED: &[u8] = b"-._~";
/// The bytes one encoded byte takes: `%` and two hex digits.
const ENCODED: usize = 3;

/// True for a language tag: parts of ASCII letters joined by hyphens, at most `TAG_CHARS` long.
#[must_use]
pub fn tag_ok(tag: &str) -> bool {
    let letters = |part: &str| !part.is_empty() && part.bytes().all(|b| b.is_ascii_alphabetic());
    tag.len() <= TAG_CHARS && tag.split(TAG_JOIN).all(letters)
}

impl Site {
    /// The code this site takes for `set`, or for the system's display language `os` when `set` is empty: a tag it
    /// lists whole, else the tag's first part in lower case, else its fallback.
    #[must_use]
    pub fn target(&self, set: &str, os: &str) -> String {
        let tag = if set.is_empty() { os } else { set };
        if let Some(whole) = self.whole.iter().find(|w| w.eq_ignore_ascii_case(tag)) {
            return whole.clone();
        }
        let first = tag.split(OS_JOINS).next().unwrap_or_default();
        if tag_ok(first) {
            first.to_ascii_lowercase()
        } else {
            self.fallback.clone()
        }
    }

    /// This site's page translating `text` into `code`, cut to `chars` characters and the site's limits; `None` when
    /// no text is left.
    #[must_use]
    pub fn link(&self, text: &str, code: &str, chars: u16) -> Option<Link> {
        let clean = clean(text);
        let to = encode(code);
        let room = (self.link_max)
            .saturating_sub(self.bare().len())
            .saturating_sub(to.len());
        let kept = cut(&clean, usize::from(chars.min(self.limit)), room);
        if kept.is_empty() {
            return None;
        }
        let link = (self.pattern)
            .replacen(TO, &to, 1)
            .replacen(TEXT, &encode(kept), 1);
        Link::new(link).ok()
    }
}

/// `text` with each hidden space or break made a space, so words stay apart, every other hidden mark dropped, and
/// trimmed.
fn clean(text: &str) -> String {
    let kept = text
        .chars()
        .filter_map(|c| match (is_hidden(c), c.is_whitespace()) {
            (false, _) => Some(c),
            (true, true) => Some(' '),
            (true, false) => None,
        });
    kept.collect::<String>().trim().to_owned()
}

/// The longest start of `text` within `chars` characters and `room` encoded bytes, ended at a whole word.
fn cut(text: &str, chars: usize, room: usize) -> &str {
    let mut bytes = 0_usize;
    for (count, (at, c)) in text.char_indices().enumerate() {
        bytes = bytes.saturating_add(encoded_len(c));
        if count >= chars || bytes > room {
            return whole(text, at);
        }
    }
    text
}

/// `text` cut before byte `at`, back to the last space; inside a single long word, at the last whole character.
fn whole(text: &str, at: usize) -> &str {
    let head = text.get(..at).unwrap_or_default();
    let rest = text.get(at..).unwrap_or_default();
    if rest.starts_with(char::is_whitespace) {
        return head.trim_end();
    }
    match head.rfind(char::is_whitespace) {
        Some(space) => head.get(..space).unwrap_or_default().trim_end(),
        None => text.get(..grapheme_floor(text, at)).unwrap_or_default(),
    }
}

/// The last grapheme boundary at or before `at`, so a letter keeps its marks.
fn grapheme_floor(text: &str, at: usize) -> usize {
    let start = caret::back(text, at);
    if caret::on(text, start) <= at {
        at
    } else {
        start
    }
}

/// True for a byte RFC 3986 keeps as it is.
fn unreserved(b: u8) -> bool {
    b.is_ascii_alphanumeric() || UNRESERVED.contains(&b)
}

/// The bytes `c` takes once encoded.
fn encoded_len(c: char) -> usize {
    let mut utf8 = [0; 4];
    let bytes = c.encode_utf8(&mut utf8).as_bytes();
    bytes
        .iter()
        .map(|&b| if unreserved(b) { 1 } else { ENCODED })
        .sum()
}

/// `text` percent-encoded as RFC 3986: unreserved bytes kept, every other byte of its UTF-8 as `%XX`.
fn encode(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for b in text.bytes() {
        if unreserved(b) {
            out.push(char::from(b));
        } else {
            // Writing to a `String` never fails.
            let _ = write!(out, "%{b:02X}");
        }
    }
    out
}

#[cfg(test)]
mod props;
#[cfg(test)]
mod tests;
