//! Paste (design "Paste"): the one take waited for, within `paste_wait_ms`, brings the clipboard's text from the
//! clipboard history into a New or Edit field. Line breaks (`LINE_BREAKS`, NEL too) at its ends go; one inside becomes
//! one space in a shown field, as a break would send the message in chat apps, and refuses a hidden one.

use std::time::Duration;

use kx_module_api::ports::{ClipTake, ClipTaken, FillNote};
use kx_module_api::text::SPACE;
use kx_module_api::{LINE_BREAKS, Mono, Zeroizing};

/// A carriage return, which a line feed right after joins into one break.
const CR: char = '\r';
/// A line feed.
const LF: char = '\n';

/// Paste's take: its round, its wait and its field's room, and when its wait ends while it waits.
#[derive(Debug)]
pub struct Pasting {
    round: u32,
    until: Option<Mono>,
    wait: Duration,
    field_chars: usize,
}

impl Pasting {
    /// Takes that wait `wait` for their answer, for fields of `field_chars` characters.
    #[must_use]
    pub const fn new(wait: Duration, field_chars: usize) -> Self {
        Self {
            round: 0,
            until: None,
            wait,
            field_chars,
        }
    }

    /// A new take at `now`; an earlier take's answer is dropped from now on.
    pub fn ask(&mut self, now: Mono) -> ClipTake {
        self.round = self.round.wrapping_add(1);
        self.until = Some(now.saturating_add(self.wait));
        ClipTake::new(self.round, self.field_chars)
    }

    /// True when `taken` answers the take waited for, which then waits no more; any other answer is dropped.
    pub fn answers(&mut self, taken: &ClipTaken) -> bool {
        if self.until.is_none() || taken.round != self.round {
            return false;
        }
        self.until = None;
        true
    }

    /// When the wait ends, while a take waits.
    #[must_use]
    pub const fn due(&self) -> Option<Mono> {
        self.until
    }

    /// True once, when the wait ran out by `now`; the answer is dropped from then on.
    pub fn expired(&mut self, now: Mono) -> bool {
        let out = self.until.is_some_and(|t| now >= t);
        if out {
            self.until = None;
        }
        out
    }

    /// Stops waiting, as the panel closes, so the answer is dropped.
    pub fn stop(&mut self) {
        self.until = None;
    }
}

/// `text` for a field that is `hidden` or not: breaks at its ends trimmed and each inside one space, in a wiped buffer
/// that holds its room from the start.
///
/// # Errors
/// `NoText` for a text of only breaks, and `Refused` for a break inside a hidden value.
pub fn flat(text: &str, hidden: bool) -> Result<Zeroizing<String>, FillNote> {
    let text = text.trim_matches(LINE_BREAKS.as_slice());
    if text.is_empty() {
        return Err(FillNote::NoText);
    }
    if hidden && text.contains(LINE_BREAKS.as_slice()) {
        return Err(FillNote::Refused);
    }
    let mut out = Zeroizing::new(String::with_capacity(text.len()));
    let mut chars = text.chars().peekable();
    while let Some(c) = chars.next() {
        if c == CR {
            chars.next_if_eq(&LF);
        }
        out.push(if LINE_BREAKS.contains(&c) { SPACE } else { c });
    }
    Ok(out)
}

#[cfg(test)]
mod tests;
