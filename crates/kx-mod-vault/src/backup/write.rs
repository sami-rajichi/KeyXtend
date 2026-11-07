//! The writes (design "The backup box"): `box_delay_ms` after the last save, edit or delete, each change restarting the
//! wait, the notebook is sealed once and written to every place that can take it. A place gone, full or failing waits
//! with its note and catches up on the next change, when the panel opens, and every `place_check_s`; a failed write
//! keeps the old box there, and a box too large waits for the next change or Back up now.

use std::path::PathBuf;

use kx_module_api::ports::{BoxError, FillNote, PageNote, PlaceError};
use kx_module_api::{Mono, Redacted};

use super::flow::{Asking, Flow};
use super::jobs::{Job, Write, Wrote};
use super::state::{At, Backup, Flowed, Writing};
use crate::book::Notebook;
use crate::entry::Stamp;
use crate::store::Store;

impl Backup {
    /// A save, edit or delete at `now`: a failed blob save tries again, the write goes `box_delay_ms` after the last one,
    /// and the blob says so at once.
    pub(crate) fn changed(&mut self, now: Mono) {
        self.retry();
        self.too_large = false;
        if self.ready() {
            self.due = Some(now.saturating_add(self.cfg.delay));
            self.mark_behind();
        }
    }

    /// Saves `behind` in the blob once a write is wanted, so a stop or a crash before it writes after the next start.
    fn mark_behind(&mut self) {
        if !self.kept.behind {
            self.kept.behind = true;
            self.save();
        }
    }

    /// When the actor wakes for the backup: the write due, or the next look at a waiting place unless the box is too
    /// large; none while one runs.
    pub(crate) fn next(&self) -> Option<Mono> {
        if self.writing.is_some() || !self.ready() {
            return None;
        }
        let waits = !self.too_large && self.kept.sites.iter().any(|s| s.waiting);
        let look = waits.then_some(self.look_at);
        [self.due, look].into_iter().flatten().min()
    }

    /// Writes when due, and looks at the waiting places every `place_check_s`.
    pub(crate) fn tick(&mut self, now: Mono, book: &Notebook) {
        if self.writing.is_some() || !self.ready() {
            return;
        }
        if self.due.is_some_and(|due| due <= now) {
            self.due = None;
            self.write(book, false, now);
        } else if self.look_at <= now {
            self.look_at = now.saturating_add(self.cfg.look_every);
            self.catch_up(book, now);
        }
    }

    /// Looks at the waiting places and writes those back, as when the panel opens; a box too large cannot fit yet.
    pub(crate) fn catch_up(&mut self, book: &Notebook, now: Mono) {
        if self.ready() && !self.too_large {
            self.write(book, true, now);
        }
    }

    /// Back up now: every place at once; while held, the Backup page first asks to replace the backup.
    pub(super) fn back_up_now(&mut self, at: &At<'_>) -> Flowed {
        match &mut self.flow {
            Some(Flow::Home { asking, .. }) if self.kept.held => *asking = Some(Asking::Replace),
            _ => self.write(at.book, false, at.now),
        }
        Flowed::Show
    }

    /// Ends the hold, after Replace's Yes or Bring back, and writes every place.
    pub(crate) fn release(&mut self, book: &Notebook, now: Mono) {
        (self.kept.held, self.due) = (false, None);
        self.save();
        self.write(book, false, now);
    }

    /// True while the hold keeps every box as it is.
    pub(crate) fn paused(&self) -> bool {
        self.kept.held
    }

    /// The note of a write or an import that ended while no backup page showed, taken once for the panel's own page.
    pub(crate) fn untold(&mut self) -> Option<PageNote> {
        if self.flow.is_some() {
            return None;
        }
        let note = self.note.take()?;
        Some(self.page_note(note))
    }

    /// True when there is a padlock, a place, and writes are not held.
    fn ready(&self) -> bool {
        self.kept.padlock.is_some() && !self.kept.sites.is_empty() && !self.kept.held
    }

    /// True while held, when nothing is written and a full write only makes every place wait.
    fn held_back(&mut self, waiting_only: bool) -> bool {
        if self.kept.held && !waiting_only {
            self.kept.sites.iter_mut().for_each(|s| s.waiting = true);
            self.save();
        }
        self.kept.held
    }

    /// Writes every place, or only the waiting ones, unless held; while a write runs, a full write goes as soon as it
    /// ends.
    pub(super) fn write(&mut self, book: &Notebook, waiting_only: bool, now: Mono) {
        if self.held_back(waiting_only) {
            return;
        }
        if !waiting_only {
            self.mark_behind();
        }
        if self.writing.is_some() {
            if !waiting_only {
                self.due = Some(now);
            }
            return;
        }
        let Some(padlock) = self.kept.padlock.clone() else {
            return;
        };
        let sites = self.kept.sites.iter();
        let picked = sites.filter(|s| !waiting_only || s.waiting);
        let folders: Vec<Redacted<PathBuf>> = picked.map(|s| s.place.path.clone()).collect();
        if folders.is_empty() {
            return;
        }
        let write = Write {
            padlock,
            notebook: Redacted::new(Store::part(book)),
            folders: folders.clone(),
            cap: self.cfg.cap,
            need: self.kept.sites.iter().map(|s| s.len).max().unwrap_or(0),
        };
        let round = self.post(Job::Write(write));
        let full = !waiting_only;
        self.writing = Some(Writing {
            round,
            folders,
            full,
        });
    }

    /// Takes a write's answer, written at `at`; false for an older round's, which is dropped.
    pub(super) fn wrote(
        &mut self,
        round: u32,
        wrote: Result<Wrote, BoxError>,
        at: Stamp,
        now: Mono,
    ) -> bool {
        let Some(Writing { folders, full, .. }) = self.writing.take_if(|w| w.round == round) else {
            return false;
        };
        self.look_at = now.saturating_add(self.cfg.look_every);
        if full && self.due.is_none() {
            self.kept.behind = false;
        }
        self.too_large = wrote == Err(BoxError::TooLarge);
        match wrote {
            Ok(wrote) => self.reached(&folders, &wrote, at),
            Err(BoxError::TooLarge) => {
                self.note = Some(FillNote::TooLarge);
                self.wait_for(&folders);
            }
            Err(error) => {
                tracing::warn!(%error, "the backup box was not sealed");
                self.note = Some(FillNote::BoxFailed);
                self.wait_for(&folders);
            }
        }
        self.save();
        true
    }

    /// Notes each folder's outcome: written now, waiting, or the data folder, which is no place for the box.
    fn reached(&mut self, folders: &[Redacted<PathBuf>], wrote: &Wrote, at: Stamp) {
        let (mark, len) = wrote.sealed.unzip();
        for (folder, reach) in folders.iter().zip(&wrote.reached) {
            let Some(site) = self.kept.sites.iter_mut().find(|s| s.place.path == *folder) else {
                continue;
            };
            match reach {
                Ok(()) => {
                    (site.written, site.mark, site.waiting) = (Some(at), mark, false);
                    site.len = len.unwrap_or(site.len);
                }
                Err(error) => {
                    site.waiting = true;
                    self.note = place_note(*error).or(self.note);
                }
            }
        }
        let refused = |r: &Result<(), PlaceError>| *r == Err(PlaceError::Refused);
        let gone = folders
            .iter()
            .zip(&wrote.reached)
            .filter(|(_, r)| refused(r));
        let gone: Vec<_> = gone.map(|(f, _)| f.clone()).collect();
        self.drop_sites(&gone);
    }

    /// Drops the sites in `gone`: a Remove question showing goes, as its index may name another place, and the
    /// highlight stays within the list.
    fn drop_sites(&mut self, gone: &[Redacted<PathBuf>]) {
        let before = self.kept.sites.len();
        self.kept.sites.retain(|s| !gone.contains(&s.place.path));
        let len = self.kept.sites.len();
        let Some(Flow::Home { at, asking }) = self.flow.as_mut().filter(|_| len < before) else {
            return;
        };
        *at = (*at).min(len.saturating_sub(1));
        if matches!(asking, Some(Asking::Remove(_))) {
            *asking = None;
        }
    }

    /// Makes the sites in `folders` wait.
    fn wait_for(&mut self, folders: &[Redacted<PathBuf>]) {
        let sites = self.kept.sites.iter_mut();
        sites
            .filter(|s| folders.contains(&s.place.path))
            .for_each(|s| s.waiting = true);
    }
}

/// The note a place's failed write shows: the data folder and a place that cannot be written have one; a place gone
/// or full shows only as waiting.
pub(super) fn place_note(error: PlaceError) -> Option<FillNote> {
    match error {
        PlaceError::Refused => Some(FillNote::DataFolder),
        PlaceError::ReadOnly => Some(FillNote::ReadOnly),
        PlaceError::Missing | PlaceError::Full | PlaceError::TooLarge | PlaceError::Io => None,
    }
}
