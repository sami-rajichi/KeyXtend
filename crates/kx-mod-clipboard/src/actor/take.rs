//! Quick-fill's take (ADR-0034, design "Paste"): the clipboard's text, read within the take's cap, goes to Quick-fill,
//! and the history forgets its newest entry when it is that text and not pinned, with an app's write of it alone just
//! before; every copy of it the watch reads within `take_wait_ms` keeps nothing either, from any app. Paste at once
//! never puts that text back until the history keeps it again; only its hash stays, every copy read here is wiped, and
//! none is logged.

use kx_module_api::ports::{ClipError, ClipKind, ClipPart, ClipTake, ClipText};
use kx_module_api::{Mono, Secret, Zeroizing};

use super::Actor;
use crate::entry::Entry;
use crate::hash;

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

    /// Deletes the newest entry when it is a text not pinned that holds `text`, then the unpinned entry of `text` alone
    /// copied within `take_wait_ms` of it, as one app may write a text alone and then with other formats.
    fn forget_copy(&mut self, text: &Secret) {
        let newest = self.history.entries().iter().max_by_key(|e| e.seq);
        let Some(entry) = newest.filter(|e| self.holds_text(e, text)) else {
            return;
        };
        let (id, at) = (entry.id, entry.at.unix_ms);
        self.remove(id);
        let hash = hash::of_text(text.expose());
        let wait = self.settings.take_wait().as_millis();
        let near = |e: &&Entry| e.hash == hash && u128::from(e.at.unix_ms.abs_diff(at)) <= wait;
        let mut twins = self.history.entries().iter().filter(near);
        if let Some(id) = twins.find(|e| self.holds_text(e, text)).map(|e| e.id) {
            self.remove(id);
        }
    }

    /// True when `entry` is a text not pinned that holds `text`; one that cannot be read is logged and does not.
    fn holds_text(&self, entry: &Entry, text: &Secret) -> bool {
        if entry.pinned() || !matches!(entry.kind, ClipKind::Text | ClipKind::Link) {
            return false;
        }
        match self.store.parts(entry) {
            Ok(parts) => {
                let parts = parts.unwrap_or_default();
                let same = holds(&parts, text);
                wipe(parts);
                same
            }
            Err(error) => {
                tracing::warn!(%error, "a clipboard entry could not be read");
                false
            }
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
