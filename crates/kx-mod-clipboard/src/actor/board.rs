//! What the clipboard holds, as the watch and our own writes told it, so Paste at once can put an entry back whole
//! (owner, 2026-11-17, ADR-0048). Our writes come back from the watch as `Marked`, so each is counted and its echo
//! keeps the board. Privacy is checked live at the read: a clipboard with a private marker on it is never read.

use std::mem;

use kx_module_api::ports::{ClipError, ClipNote, ClipPart};
use kx_module_api::{Redacted, Secret};

use super::Actor;
use super::paste::wants_dib;
use super::take::wipe;
use crate::entry::text_of;

/// What the clipboard holds, as far as the history knows.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub(super) enum Held {
    /// Anything the history cannot name; its unmarked text alone may come back.
    #[default]
    Other,
    /// Nothing.
    Empty,
    /// Entry `id`'s content.
    Entry(u64),
}

/// What the clipboard holds, and how many of our writes the watch has not told back yet.
#[derive(Debug, Default)]
pub(super) struct Board {
    held: Held,
    echoes: u32,
}

impl Board {
    /// The watch told `note`; true for our own write's echo, which keeps the board. A clear empties it, and anything
    /// else is other until a copy is kept.
    pub(super) fn heard(&mut self, note: &ClipNote) -> bool {
        match note {
            ClipNote::Marked { .. } if self.echoes > 0 => {
                self.echoes -= 1;
                return true;
            }
            ClipNote::Cleared { .. } => self.held = Held::Empty,
            _ => self.held = Held::Other,
        }
        false
    }

    /// The copy just told was kept as entry `id`.
    pub(super) fn kept(&mut self, id: u64) {
        self.held = Held::Entry(id);
    }

    /// We wrote what `held` names, so its echo is due.
    pub(super) fn wrote(&mut self, held: Held) {
        self.held = held;
        self.echoes = self.echoes.saturating_add(1);
    }
}

/// What to write back after a paste, and what the board then holds.
#[derive(Debug)]
pub(super) struct Back {
    pub(super) parts: Vec<ClipPart>,
    pub(super) held: Held,
}

impl Actor {
    /// What to put back after a paste, once the live check finds no private marker: the entry on the board, whole,
    /// when the clipboard's text agrees; else the clipboard's text alone; or nothing for an empty clipboard. `None`
    /// for a marked or unreadable clipboard, or one with no text the history cannot name, as it may hold a picture.
    pub(super) fn previous(&mut self) -> Option<Back> {
        let read = self
            .ports
            .clip
            .unmarked_text(self.settings.wait(), self.settings.back_chars);
        let text = match read {
            Ok(text) => text,
            Err(ClipError::Marked) => {
                tracing::info!("a private copy is on the clipboard, so the text is typed");
                return None;
            }
            Err(error) => {
                tracing::warn!(%error, "the clipboard was not read before a paste");
                return None;
            }
        };
        let held = self.board.held;
        if let Held::Entry(id) = held
            && let Some(parts) = self.whole(id, text.as_ref())
        {
            return Some(Back { parts, held });
        }
        let (parts, held) = match (text, held) {
            (Some(text), _) => (vec![ClipPart::Text(own(text))], Held::Other),
            (None, Held::Empty) => (Vec::new(), held),
            (None, _) => return None,
        };
        Some(Back { parts, held })
    }

    /// Entry `id`'s stored parts when the clipboard holds them: the same text, or no text in either, and a picture
    /// that needs no DIB made.
    fn whole(&self, id: u64, text: Option<&Secret>) -> Option<Vec<ClipPart>> {
        let entry = self.history.get(id)?;
        let parts = self.store.parts(entry).ok().flatten()?;
        let same = text_of(&parts) == text.map(|t| t.expose().as_str());
        if same && !wants_dib(&parts) {
            return Some(parts);
        }
        wipe(parts);
        None
    }
}

/// Moves `text` into a clipboard part, leaving nothing behind to wipe.
pub(super) fn own(text: Secret) -> Redacted<String> {
    let mut text = text.into_inner();
    Redacted::new(mem::take(&mut *text))
}
