//! The helpers' answers (edges 4 and 9): file facts size the rows and drop an entry whose files are all gone, and a
//! drawn thumbnail is saved for its row; a picture the worker could not draw leaves the type icon. One thumbnail
//! draws at a time, and only the newest preview waits.

use kx_module_api::ports::{ClipKind, FileFact, Preview};

use super::Actor;
use crate::insert::files_of;
use crate::media::{Drawn, Facts, Job, Look, Size, Why, job_of};

impl Actor {
    /// Asks the files thread what is at the paths of every file row, to size them; each look asks for all, so the
    /// thread keeps only the newest.
    pub(super) fn look_rows(&self) {
        let filed = self.history.entries().iter();
        let filed = filed.filter(|e| matches!(e.kind, ClipKind::File | ClipKind::Media));
        let ids: Vec<u64> = filed.map(|e| e.id).collect();
        let items = files_of(&ids, &self.history);
        if !items.is_empty() {
            let why = Why::Rows;
            // Only a stopped files thread refuses; a row look then waits for the next open.
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

    /// Takes a picture the media thread drew: a thumbnail is saved and the next bare row asked, a preview shows while
    /// its page does and the one waiting goes.
    pub(super) fn drawn(&mut self, drawn: Drawn) {
        if let Some(job) = self.draws.done(drawn.size) {
            self.send_job(job);
        }
        if drawn.size == Size::Thumb {
            self.want_thumbs(&self.bare.clone());
        }
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

    /// Asks once per run for the thumbnails of `ids`, image rows without one, while no other is drawing.
    pub(super) fn want_thumbs(&mut self, ids: &[u64]) {
        for &id in ids {
            if !self.draws.free(Size::Thumb) {
                return;
            }
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

    /// Hands `job` to the media thread now, or keeps it while one of its size draws; a thumbnail kept is asked again.
    pub(super) fn send_job(&mut self, job: Job) {
        if !self.draws.free(job.size) {
            if job.size == Size::Preview {
                self.draws.hold(job);
            }
            return;
        }
        self.draws.went(job.size);
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
