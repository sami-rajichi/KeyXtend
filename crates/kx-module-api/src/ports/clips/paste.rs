//! Paste at once (P11, ADR-0048): a module that types, as the voice module, hands a text to the clipboard history,
//! which pastes it through its own paste route and puts the clipboard back. Both events need `WriteClipboard`, as the
//! paste request does, so the asker never gains a way to read the clipboard.

use super::WRITE;
use crate::{Capability, Event, Secret};

/// The part of the asker's wait a paste may start in: the first half, so its answer always comes in time.
const START_SHARE: u32 = 2;

/// To the clipboard module: paste `text` once into the app in front, then put the clipboard back; past `wait_ms` the
/// asker types the text itself.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PasteText {
    /// Names this request, so its answer names it too.
    pub round: u32,
    /// The text, wiped on drop.
    pub text: Secret,
    /// How long the asker waits for the answer, in ms, before it types the text itself.
    pub wait_ms: u32,
}

impl PasteText {
    /// How long after the ask a paste may still start, in ms: the first part of `wait_ms`, so a paste that starts
    /// is always answered before the asker gives up.
    #[must_use]
    pub const fn start_ms(&self) -> u32 {
        self.wait_ms / START_SHARE
    }
}

impl Event for PasteText {
    const NAME: &'static str = "paste-text";
    const CAPABILITY: Option<Capability> = WRITE;
}

/// From the clipboard module: whether a `PasteText`'s text was pasted. On false not all of it went, so the asker types
/// it.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct TextPasted {
    /// The round of the request.
    pub round: u32,
    /// All of the paste went out.
    pub ok: bool,
}

impl Event for TextPasted {
    const NAME: &'static str = "text-pasted";
    const CAPABILITY: Option<Capability> = WRITE;
}

#[cfg(test)]
mod tests;
