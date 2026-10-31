//! What waits for the helper threads, bounded with no setting: one thumbnail drawing and one preview with the newest
//! waiting, so at most three image copies are in flight; insert checks before the newest row look.

use std::collections::VecDeque;
use std::sync::mpsc::Receiver;

use super::{Job, Look, Size, Why};

/// The pictures handed to `kx-clip-media`, as the actor keeps them.
#[derive(Debug, Default)]
pub struct Draws {
    /// A thumbnail is drawing.
    thumb: bool,
    /// A preview is drawing.
    preview: bool,
    /// The newest preview asked while one was drawing.
    waiting: Option<Job>,
}

impl Draws {
    /// True when a job of `size` may go now; a thumbnail that may not is asked again at the next view.
    #[must_use]
    pub fn free(&self, size: Size) -> bool {
        match size {
            Size::Thumb => !self.thumb,
            Size::Preview => !self.preview,
        }
    }

    /// Notes that a job of `size` went.
    pub fn went(&mut self, size: Size) {
        *self.slot(size) = true;
    }

    /// Keeps `job`, a preview, until the one drawing ends; it replaces one already waiting.
    pub fn hold(&mut self, job: Job) {
        self.waiting = Some(job);
    }

    /// The thread answered a job of `size`; gives the preview waiting, which goes at once.
    pub fn done(&mut self, size: Size) -> Option<Job> {
        *self.slot(size) = false;
        let next = (size == Size::Preview)
            .then(|| self.waiting.take())
            .flatten();
        self.preview |= next.is_some();
        next
    }

    /// The flag of `size`.
    fn slot(&mut self, size: Size) -> &mut bool {
        match size {
            Size::Thumb => &mut self.thumb,
            Size::Preview => &mut self.preview,
        }
    }
}

/// The looks waiting for `kx-clip-files`: insert checks in order, then only the newest row look.
#[derive(Default)]
pub(super) struct Looks {
    inserts: VecDeque<Look>,
    rows: Option<Look>,
}

impl Looks {
    /// The next look to answer, after taking all that `inbox` holds; `None` once it closed and none is left.
    pub(super) fn next(&mut self, inbox: &Receiver<Look>) -> Option<Look> {
        if self.inserts.is_empty() && self.rows.is_none() {
            let first = inbox.recv().ok()?;
            self.add(first);
        }
        for look in inbox.try_iter() {
            self.add(look);
        }
        self.inserts.pop_front().or_else(|| self.rows.take())
    }

    /// Queues `look`; a row look replaces the one waiting, as each asks for every file row.
    fn add(&mut self, look: Look) {
        match look.why {
            Why::Insert(_) => self.inserts.push_back(look),
            Why::Rows => self.rows = Some(look),
        }
    }
}
