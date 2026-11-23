//! A take of the clipboard's text (ADR-0034, ADR-0047): Quick-fill's Paste or the key page's asks the clipboard module
//! for the text, and its answer names the asking module back, so no other module takes it.

use super::READ;
use crate::ports::MAX_FIELD_CHARS;
use crate::{Capability, Event, ModuleId, Secret};

/// A CR LF is two characters until Paste turns it into one space, so a take reads twice a field's characters.
const SLACK: usize = 2;

/// From Quick-fill or the key page: Paste asks the clipboard module for the clipboard's current text, and for its
/// history to forget that copy.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct ClipTake {
    /// The module that asks, which the answer names so no other module takes it.
    pub asker: ModuleId,
    /// Names this request, so its answer names it too.
    pub round: u32,
    /// The most characters to read; a longer text is refused.
    pub max_chars: usize,
}

impl ClipTake {
    /// Module `asker`'s take of round `round` for a field of `field_chars` characters, with room for each break to be
    /// two.
    #[must_use]
    pub const fn new(asker: ModuleId, round: u32, field_chars: usize) -> Self {
        Self {
            asker,
            round,
            max_chars: field_chars.saturating_mul(SLACK),
        }
    }

    /// The most characters the clipboard reads: `max_chars`, but never past the room of the longest field.
    #[must_use]
    pub const fn cap(&self) -> usize {
        let most = MAX_FIELD_CHARS.saturating_mul(SLACK);
        if self.max_chars < most {
            self.max_chars
        } else {
            most
        }
    }

    /// The answer to this take, naming its asker and round, with what the clipboard held.
    #[must_use]
    pub const fn answer(&self, text: ClipText) -> ClipTaken {
        ClipTaken {
            asker: self.asker,
            round: self.round,
            text,
        }
    }
}

impl Event for ClipTake {
    const NAME: &'static str = "clip-take";
    const CAPABILITY: Option<Capability> = READ;
}

/// What a take found on the clipboard.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ClipText {
    /// The clipboard's text.
    Text(Secret),
    /// The clipboard holds no text, such as an image.
    Empty,
    /// The text is past the take's cap, so it was refused.
    TooLong,
    /// Another app held the clipboard, the system refused it, or its text could not be read.
    Busy,
}

/// From the clipboard module: what the take found. Its history forgets its newest entry only when the take read the
/// text in full and that entry is this copy; it is not for a copy left out of history (as a password manager's), one
/// refused as too big or under the disk floor, or one made while the keyboard was off.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ClipTaken {
    /// The module that asked, as the take named it; only that module takes the answer.
    pub asker: ModuleId,
    /// The round of the request.
    pub round: u32,
    /// What the take found.
    pub text: ClipText,
}

impl ClipTaken {
    /// True when this answers module `asker`'s take of round `round`.
    #[must_use]
    pub fn is_for(&self, asker: ModuleId, round: u32) -> bool {
        self.asker == asker && self.round == round
    }
}

impl Event for ClipTaken {
    const NAME: &'static str = "clip-taken";
    const CAPABILITY: Option<Capability> = READ;
}
