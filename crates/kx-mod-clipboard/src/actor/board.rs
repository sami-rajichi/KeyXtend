//! What the clipboard holds, as the watch and our own writes told it, so Paste at once can put it back (owner,
//! 2026-11-17). Our writes come back from the watch as `Marked`, so each is counted and its echo keeps the board; a
//! mark another app set makes it private, and a private or unknown clipboard is never read or written back (ADR-0048).

use std::mem;

use kx_module_api::ports::{ClipNote, ClipPart};
use kx_module_api::{Redacted, Secret};

use super::Actor;
use super::paste::wants_dib;
use super::take::wipe;
use crate::entry::text_of;

/// What the clipboard holds, as far as the history knows.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub(super) enum Held {
    /// Not seen since the start, or a change the watch missed: it may be private.
    #[default]
    Unknown,
    /// A copy another app marked private, as a password manager does, or a skipped app's copy.
    Private,
    /// A copy the history did not keep: its text alone may come back.
    Plain,
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
    /// The watch told `note`; true for our own write's echo, which keeps the board. Another app's mark makes it
    /// private, a clear empties it, a copy is plain until kept, and a missed change is unknown.
    pub(super) fn heard(&mut self, note: &ClipNote) -> bool {
        self.held = match note {
            ClipNote::Marked { .. } if self.echoes > 0 => {
                self.echoes -= 1;
                return true;
            }
            ClipNote::Marked { .. } => Held::Private,
            ClipNote::Cleared { .. } => Held::Empty,
            ClipNote::Copied { .. } | ClipNote::TooBig { .. } => Held::Plain,
            ClipNote::Busy => Held::Unknown,
        };
        false
    }

    /// The copy just told is `held`: an entry kept, or a skipped app's private copy.
    pub(super) fn set(&mut self, held: Held) {
        self.held = held;
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
    /// What to put back after a paste: the entry on the board, whole, when the clipboard's text agrees; a plain
    /// copy's text alone; or nothing for an empty clipboard. `None`, read or not, for anything else.
    pub(super) fn previous(&mut self) -> Option<Back> {
        let held = self.board.held;
        if matches!(held, Held::Unknown | Held::Private) {
            tracing::info!(
                ?held,
                "the clipboard cannot be put back, so the text is typed"
            );
            return None;
        }
        let text = match self
            .ports
            .clip
            .text(self.settings.wait(), self.settings.back_chars)
        {
            Ok(text) => text,
            Err(error) => {
                tracing::warn!(%error, "the clipboard was not read before a paste");
                return None;
            }
        };
        let parts = match (held, text) {
            (Held::Entry(id), text) => self.whole(id, text.as_ref()),
            (Held::Plain, Some(text)) => Some(vec![ClipPart::Text(own(text))]),
            (Held::Empty, None) => Some(Vec::new()),
            _ => None,
        };
        parts.map(|parts| Back { parts, held })
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
