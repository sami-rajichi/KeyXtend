//! Quick-fill's take (ADR-0034, design "Paste"): the clipboard's text, read within the take's cap, goes to Quick-fill,
//! and the history forgets its newest entry when it is that text and not pinned; every note of the copy the take read
//! keeps nothing either while the watch tells it within `take_wait_ms`, from any app. Paste at once never puts that
//! text back until the history keeps it again; only its hash stays, every copy read here is wiped, and none is logged.

use kx_module_api::ports::{ClipError, ClipKind, ClipPart, ClipTake, ClipText};
use kx_module_api::{Mono, Secret, Zeroizing};

use super::Actor;
use crate::entry::Entry;

impl Actor {
    /// Answers `take` with what the clipboard holds, read within the take's cap: a longer text is `TooLong` and forgets
    /// nothing, and a held, refused or unreadable clipboard is `Busy`, logged by its kind alone. The answer names the
    /// take's asker, so only that module takes it, and the text leaves memory with it.
    pub(super) fn take(&mut self, take: ClipTake) {
        let wait = self.settings.clip_config().wait;
        let text = match self.ports.clip.text(wait, take.cap()) {
            Ok(Some(text)) => {
                self.lost.taken(text.expose(), self.ports.clock.now());
                self.forget_copy(&text);
                ClipText::Text(text)
            }
            Ok(None) => ClipText::Empty,
            Err(ClipError::TooLong) => ClipText::TooLong,
            Err(error) => {
                tracing::warn!(%error, "the clipboard's text was not taken");
                ClipText::Busy
            }
        };
        self.publish(&take.answer(text));
    }

    /// Deletes the newest entry when it is a text not pinned whose text is `text`.
    fn forget_copy(&mut self, text: &Secret) {
        let newest = self.history.entries().iter().max_by_key(|e| e.seq);
        let texts = |e: &&Entry| !e.pinned() && matches!(e.kind, ClipKind::Text | ClipKind::Link);
        let Some(entry) = newest.filter(texts) else {
            return;
        };
        let parts = match self.store.parts(entry) {
            Ok(parts) => parts.unwrap_or_default(),
            Err(error) => {
                tracing::warn!(%error, "the newest clipboard entry could not be read");
                return;
            }
        };
        let (same, id) = (holds(&parts, text), entry.id);
        wipe(parts);
        if same {
            self.remove(id);
        }
    }

    /// Takes the copy of `parts` the watch read at `read`: when it is a take's copy still due then, it is wiped and
    /// never goes back; else `parts` go on.
    pub(super) fn past_take(&mut self, parts: Vec<ClipPart>, read: Mono) -> Option<Vec<ClipPart>> {
        if !self.lost.noted(&parts, read) {
            return Some(parts);
        }
        wipe(parts);
        self.board.taken();
        None
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
pub(super) fn wipe(parts: Vec<ClipPart>) {
    for part in parts {
        match part {
            ClipPart::Text(t) => drop(Zeroizing::new(t.into_inner())),
            ClipPart::Html(b) | ClipPart::Rtf(b) => drop(Zeroizing::new(b.into_inner())),
            ClipPart::Image { bytes, .. } => drop(Zeroizing::new(bytes.into_inner())),
            ClipPart::Files(_) => {}
        }
    }
}
