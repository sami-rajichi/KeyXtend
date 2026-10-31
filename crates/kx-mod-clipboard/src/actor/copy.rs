//! Copies from the watch (spec §7 "Skipped copies", "Limits", "Order"): a skipped app's or a marked copy keeps
//! nothing, a copy of kept content moves its entry up, a new one is admitted or refused with a note, and the caps then
//! trim the oldest unpinned entries.

use kx_module_api::keys;
use kx_module_api::ports::{AppRef, ClipKind, ClipNote, ClipPart, PanelNote, SealedError};

use super::Actor;
use crate::entry::{Entry, keep_parts};
use crate::keep::{admit, trim};
use crate::media::{Size, job_of};

impl Actor {
    /// Takes `note` from the watch: a copy too big shows a note unless its app is skipped, and a busy clipboard tells
    /// nothing.
    pub(super) fn note(&mut self, note: ClipNote) {
        match note {
            ClipNote::Copied { parts, app } => self.copied(parts, app.as_ref()),
            ClipNote::Cleared { app } => self.cleared(app.as_ref()),
            ClipNote::TooBig { app, .. } if !self.follow.skips(app.as_ref()) => {
                self.say_note(PanelNote::TooBig);
            }
            ClipNote::TooBig { .. } | ClipNote::Marked { .. } | ClipNote::Busy => {}
        }
    }

    /// Shows `note` in the panel.
    pub(super) fn say_note(&mut self, note: PanelNote) {
        self.note = Some(note);
        self.redraw = true;
    }

    /// Keeps a copy of `parts` by `app`, moves up the entry holding the same content, or refuses it with a note.
    fn copied(&mut self, parts: Vec<ClipPart>, app: Option<&AppRef>) {
        if self.follow.skips(app) {
            return;
        }
        let Some(parts) = keep_parts(parts) else {
            return;
        };
        let now = self.ports.calendar.now();
        let entry = Entry::new(&parts, app.cloned(), &now, &self.settings.entry_config());
        if let Some(id) = self.twin(&entry, &parts) {
            if let Some(from) = self.history.raise(id, &now) {
                self.moved(id, from);
            }
            return self.kept(id, app);
        }
        self.refresh_room();
        let free = self.room.as_ref().map_or(u64::MAX, |r| r.free);
        if let Err(note) = admit(&self.history, &entry, free, &self.limits) {
            return self.say_note(note);
        }
        let kind = entry.kind;
        let id = self.history.add(entry);
        if self.put(id, &parts) {
            self.kept(id, app);
            self.look_new(id, kind, &parts);
            for gone in trim(&mut self.history, &self.limits) {
                self.forget(&gone);
            }
        }
    }

    /// The entry holding the same content as `entry`, a new copy of `parts`: same hash, and the stored parts equal.
    fn twin(&self, entry: &Entry, parts: &[ClipPart]) -> Option<u64> {
        let store = &self.store;
        let same = |e: &Entry| store.parts(e).ok().flatten().is_some_and(|p| p == parts);
        self.history.twin(entry.hash, same)
    }

    /// Saves the content of new entry `id`; a failed save deletes the entry, and a locked shelf says so once.
    fn put(&mut self, id: u64, parts: &[ClipPart]) -> bool {
        let Some(entry) = self.history.get(id) else {
            return false;
        };
        match self.store.put(entry, parts) {
            Ok(n) => self.history.update(id, |e| e.parts = n),
            Err(error) => {
                tracing::warn!(%error, "a clipboard copy was not saved");
                if error == SealedError::Locked {
                    self.say_once(keys::CLIP_LOCKED);
                }
                self.remove(id);
                false
            }
        }
    }

    /// Notes that `id` was kept or moved up for `app`: follow-the-clear watches it, and the note is past.
    fn kept(&mut self, id: u64, app: Option<&AppRef>) {
        self.follow.kept(id, app, self.ports.clock.now());
        self.note = None;
        self.changed();
    }

    /// A clear by the app of the last kept copy, within the window, deletes it for good unless pinned.
    fn cleared(&mut self, app: Option<&AppRef>) {
        let now = self.ports.clock.now();
        if let Some(id) = self.follow.cleared(app, now, &self.history) {
            self.remove(id);
        }
    }

    /// Asks for a new image's thumbnail unless the worker is down, or a new file row's size.
    fn look_new(&mut self, id: u64, kind: ClipKind, parts: &[ClipPart]) {
        let free = self.draws.free(Size::Thumb) && self.thumbs_at.is_none();
        match kind {
            ClipKind::Image if free && self.asked.insert(id) => {
                if let Some(job) = job_of(id, Size::Thumb, parts) {
                    self.send_job(job);
                }
            }
            ClipKind::File | ClipKind::Media => self.look_rows(),
            _ => {}
        }
    }
}
