//! What the clipboard holds, as the watch and our own writes told it, so Paste at once puts an entry back whole but
//! never a skipped app's or a marked copy (owner, 2026-11-17, ADR-0048), nor one that left the history or a take read,
//! for which it leaves the clipboard empty (owner's check, 2026-11-21). A copy made before the start is unknown, our
//! writes' echoes keep the board, and a presence marker is checked live at the read.

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
    /// The content of an entry the history deleted, or a copy a take read; it never goes back.
    Gone,
}

/// What the clipboard holds, how many of our writes the watch has not told back yet, and whether it may hold a
/// skipped app's or a marked copy.
#[derive(Debug, Default)]
pub(super) struct Board {
    held: Held,
    echoes: u32,
    skipped: bool,
}

impl Board {
    /// The watch told `note`, from a skipped app or unread when `skipped`; true for our own write's echo, which keeps
    /// the board. Any other note drops the echoes still due, and a mark among them holds like a skipped app's note, as
    /// number markers alone may mark a password; only another app's copy, a clear or our own write lets either go.
    pub(super) fn heard(&mut self, note: &ClipNote, skipped: bool) -> bool {
        let cleared = matches!(note, ClipNote::Cleared { .. });
        let marked = matches!(note, ClipNote::Marked { .. });
        if marked && self.echoes > 0 && !skipped {
            self.echoes -= 1;
            return true;
        }
        self.echoes = 0;
        self.skipped = (skipped || marked) && !cleared;
        self.held = if cleared { Held::Empty } else { Held::Other };
        false
    }

    /// The copy just told was kept as entry `id`.
    pub(super) fn kept(&mut self, id: u64) {
        self.held = Held::Entry(id);
    }

    /// Entry `id` left the history, so its content never goes back if the board holds it.
    pub(super) fn gone(&mut self, id: u64) {
        if self.held == Held::Entry(id) {
            self.held = Held::Gone;
        }
    }

    /// True when the board holds entry `id`, so the watch told the copy that entry keeps.
    pub(super) fn holds(&self, id: u64) -> bool {
        self.held == Held::Entry(id)
    }

    /// The copy just told is one a take read, so it never goes back.
    pub(super) fn taken(&mut self) {
        self.held = Held::Gone;
    }

    /// We wrote what `held` names, so its echo is due and no skipped app's copy is left.
    pub(super) fn wrote(&mut self, held: Held) {
        (self.held, self.skipped) = (held, false);
        self.echoes = self.echoes.saturating_add(1);
    }
}

/// What to write back after a paste, and what the board then holds.
#[derive(Debug)]
pub(super) struct Back {
    pub(super) parts: Vec<ClipPart>,
    pub(super) held: Held,
}

/// What a kept entry on the board gives back.
enum Found {
    /// All its stored parts.
    Whole(Vec<ClipPart>),
    /// The clipboard's text differs, so the entry is no longer there.
    Moved,
    /// It may still be there but cannot come back whole, so nothing goes back.
    Partial,
}

impl Actor {
    /// What to put back after a paste: the entry on the board, whole; else the clipboard's unmarked text alone; or
    /// nothing for an empty clipboard or a text that left the history or a take read. `None` for a skipped app's or a
    /// marked copy, an unreadable clipboard, an entry that cannot come back whole, or no text the history can name, as
    /// it may be a picture.
    pub(super) fn previous(&mut self) -> Option<Back> {
        if self.board.skipped {
            tracing::info!(
                "a skipped app's or a marked copy may be on the clipboard, so the text is typed"
            );
            return None;
        }
        let text = self.unmarked().ok()?;
        let held = self.board.held;
        if let Held::Entry(id) = held {
            match self.whole(id, text.as_ref()) {
                Found::Whole(parts) => return Some(Back { parts, held }),
                Found::Partial => {
                    tracing::info!(
                        "the entry on the clipboard cannot come back whole, so the text is typed"
                    );
                    return None;
                }
                Found::Moved => {}
            }
        }
        let gone = held == Held::Gone || text.as_ref().is_some_and(|t| self.lost.has(t.expose()));
        let (parts, held) = match (text, held) {
            // The read text is wiped as it drops, and never written.
            (Some(_), _) if gone => {
                tracing::info!(
                    "a forgotten or taken copy is on the clipboard, so it is emptied after the paste"
                );
                (Vec::new(), Held::Empty)
            }
            (Some(text), _) => (vec![ClipPart::Text(own(text))], Held::Other),
            (None, Held::Empty) => (Vec::new(), held),
            (None, _) => return None,
        };
        Some(Back { parts, held })
    }

    /// The clipboard's text once the live check finds no private marker on it; a marked, long or unreadable one is
    /// logged, and only a failed read warns.
    fn unmarked(&self) -> Result<Option<Secret>, ClipError> {
        let (wait, cap) = (self.settings.wait(), self.settings.back_chars);
        let read = self.ports.clip.unmarked_text(wait, cap);
        match &read {
            Ok(_) => {}
            Err(ClipError::Marked) => {
                tracing::info!("a private copy is on the clipboard, so the text is typed");
            }
            Err(ClipError::TooLong) => {
                tracing::info!("a text past back_chars is on the clipboard, so the text is typed");
            }
            Err(error) => tracing::warn!(%error, "the clipboard was not read before a paste"),
        }
        read
    }

    /// Entry `id` as the clipboard holds it: whole when the text agrees, or neither has text, and a picture needs no
    /// DIB made; moved when the text differs; partial otherwise, a store that cannot read it included.
    fn whole(&self, id: u64, text: Option<&Secret>) -> Found {
        let stored = self.history.get(id).map(|e| self.store.parts(e));
        let Some(Ok(Some(parts))) = stored else {
            return Found::Partial;
        };
        let same = text_of(&parts) == text.map(|t| t.expose().as_str());
        if same && !wants_dib(&parts) {
            return Found::Whole(parts);
        }
        wipe(parts);
        if same { Found::Partial } else { Found::Moved }
    }
}

/// Moves `text` into a clipboard part, leaving nothing behind to wipe.
pub(super) fn own(text: Secret) -> Redacted<String> {
    let mut text = text.into_inner();
    Redacted::new(mem::take(&mut *text))
}
