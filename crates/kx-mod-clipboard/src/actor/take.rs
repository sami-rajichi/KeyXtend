//! Quick-fill's take (ADR-0034, design "Paste"): the clipboard's text, read within the take's cap, goes to Quick-fill,
//! and the history forgets its newest entry when that entry is this copy and not pinned, else the next copy keeps
//! nothing if it is that text, as the watch's note can come after the take. Every copy read here is wiped, none is
//! logged.

use kx_module_api::ports::{ClipError, ClipKind, ClipPart, ClipTake, ClipTaken, ClipText};
use kx_module_api::{Secret, Zeroizing};

use super::Actor;
use crate::entry::Entry;

impl Actor {
    /// Answers `take` with what the clipboard holds, read within the take's cap: a longer text is `TooLong` and forgets
    /// nothing, and a held, refused or unreadable clipboard is `Busy`, logged by its kind alone.
    pub(super) fn take(&mut self, take: ClipTake) {
        let wait = self.settings.clip_config().wait;
        let text = match self.ports.clip.text(wait, take.cap()) {
            Ok(Some(text)) => {
                self.missed = (!self.forget_copy(&text)).then(|| text.clone());
                ClipText::Text(text)
            }
            Ok(None) => ClipText::Empty,
            Err(ClipError::TooLong) => ClipText::TooLong,
            Err(error) => {
                tracing::warn!(%error, "the clipboard's text was not taken");
                ClipText::Busy
            }
        };
        self.publish(&ClipTaken {
            round: take.round,
            text,
        });
    }

    /// Deletes the newest entry when it is a text not pinned whose text is `text`; true when it did.
    fn forget_copy(&mut self, text: &Secret) -> bool {
        let newest = self.history.entries().iter().max_by_key(|e| e.seq);
        let texts = |e: &&Entry| !e.pinned() && matches!(e.kind, ClipKind::Text | ClipKind::Link);
        let Some(entry) = newest.filter(texts) else {
            return false;
        };
        let parts = match self.store.parts(entry) {
            Ok(parts) => parts.unwrap_or_default(),
            Err(error) => {
                tracing::warn!(%error, "the newest clipboard entry could not be read");
                return false;
            }
        };
        let same = holds(&parts, text);
        let id = entry.id;
        wipe(parts);
        if same {
            self.remove(id);
        }
        same
    }

    /// Ends the wait for the copy the last take missed: `parts` go on, unless they hold that text, so are wiped.
    pub(super) fn past_take(&mut self, parts: Vec<ClipPart>) -> Option<Vec<ClipPart>> {
        match self.missed.take() {
            Some(text) if holds(&parts, &text) => {
                wipe(parts);
                None
            }
            _ => Some(parts),
        }
    }
}

/// True when `parts` has `text` as a text part.
fn holds(parts: &[ClipPart], text: &Secret) -> bool {
    let held = text.expose().as_str();
    parts
        .iter()
        .any(|p| matches!(p, ClipPart::Text(t) if t.expose() == held))
}

/// Wipes the content of `parts` as it drops.
fn wipe(parts: Vec<ClipPart>) {
    for part in parts {
        match part {
            ClipPart::Text(t) => drop(Zeroizing::new(t.into_inner())),
            ClipPart::Html(b) | ClipPart::Rtf(b) => drop(Zeroizing::new(b.into_inner())),
            ClipPart::Image { bytes, .. } => drop(Zeroizing::new(bytes.into_inner())),
            ClipPart::Files(_) => {}
        }
    }
}
