//! The helpers' answers (edges 4 and 9): file facts size the rows and drop an entry whose files are all gone, and a
//! drawn thumbnail is saved for its row; a picture the worker could not draw leaves the type icon.

use kx_module_api::ports::{ClipKind, FileFact, Preview};

use super::Actor;
use crate::insert::files_of;
use crate::media::{Drawn, Facts, Job, Look, Size, Why, job_of};

impl Actor {
    /// The file and media entries, which an open looks at.
    pub(super) fn file_ids(&self) -> Vec<u64> {
        let filed = self.history.entries().iter();
        let filed = filed.filter(|e| matches!(e.kind, ClipKind::File | ClipKind::Media));
        filed.map(|e| e.id).collect()
    }

    /// Asks the files thread what is at the paths of `ids`, to size their rows.
    pub(super) fn look_rows(&self, ids: &[u64]) {
        let items = files_of(ids, &self.history);
        if !items.is_empty() {
            let why = Why::Rows;
            // Only a stopped files thread refuses, and it stops with the actor.
            let _ = self.ports.files.send(Look { why, items });
        }
    }

    /// Takes what the files thread found, for the rows or for an insert.
    pub(super) fn facts(&mut self, facts: Facts) {
        match facts.why {
            Why::Rows => self.sized(facts.items),
            Why::Insert(n) => self.checked(n, &facts.items),
        }
    }

    /// Takes a picture the media thread drew: a thumbnail is saved, a preview shows while its page does.
    pub(super) fn drawn(&mut self, drawn: Drawn) {
        let Ok(pixels) = drawn.pixels else {
            return;
        };
        let id = drawn.id;
        if drawn.size == Size::Preview {
            self.redraw |= self.panel.redraw(id, Preview::Image(Some(pixels)));
            return;
        }
        let Some(entry) = self.history.get(id) else {
            return;
        };
        match self.store.put_thumb(entry, &pixels) {
            Ok(()) => {
                self.history.update(id, |e| e.thumb = true);
                self.changed();
            }
            Err(error) => tracing::warn!(%error, "a clipboard thumbnail was not saved"),
        }
    }

    /// Asks once per run for the thumbnails of `ids`, image rows shown without one.
    pub(super) fn want_thumbs(&mut self, ids: &[u64]) {
        for &id in ids {
            if !self.asked.insert(id) {
                continue;
            }
            let Some(entry) = self.history.get(id) else {
                continue;
            };
            let parts = self.store.parts(entry).ok().flatten();
            if let Some(job) = parts.and_then(|p| job_of(id, Size::Thumb, &p)) {
                self.send_job(job);
            }
        }
    }

    /// Hands `job` to the media thread.
    pub(super) fn send_job(&self, job: Job) {
        // Only a stopped media thread refuses, and it stops with the actor.
        let _ = self.ports.media.send(job);
    }

    /// Sizes each row by its files still there; an entry whose files are all gone leaves (edge 4).
    fn sized(&mut self, items: Vec<(u64, Vec<FileFact>)>) {
        for (id, found) in items {
            if !found.is_empty() && found.iter().all(|f| *f == FileFact::Gone) {
                self.remove(id);
                continue;
            }
            let before = self.history.get(id).map(|e| e.shown);
            self.history.update(id, |e| e.size_files(&found));
            if self.history.get(id).map(|e| e.shown) != before {
                self.changed();
            }
        }
    }
}
