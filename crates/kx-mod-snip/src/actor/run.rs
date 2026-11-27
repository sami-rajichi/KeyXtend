//! A recording's run, owned so it is never left recording: dropped before its last note, whether the actor lets it go,
//! its message goes unheard or the thread panics, it is cancelled and its video deleted.

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use kx_module_api::ports::{RecordRun, SnipFile, SnipFolder};

use super::Leftovers;
use super::files::remove;

/// Set by the run's note stream as it tells its last note, after which its kept video stays.
pub(super) type Ended = Arc<AtomicBool>;

/// A recording's run, and what deletes its video.
pub(crate) struct Run {
    inner: Box<dyn RecordRun>,
    ended: Ended,
    folder: Arc<dyn SnipFolder>,
    leftovers: Leftovers,
    /// Not yet cancelled.
    armed: bool,
}

impl Run {
    /// Owns `inner`, whose stream sets `ended`; its video goes from `folder`, a refused delete into `leftovers`.
    pub(super) fn new(
        inner: Box<dyn RecordRun>,
        ended: Ended,
        folder: Arc<dyn SnipFolder>,
        leftovers: Leftovers,
    ) -> Self {
        Self {
            inner,
            ended,
            folder,
            leftovers,
            armed: true,
        }
    }

    /// The video it records into.
    pub(super) fn file(&self) -> &SnipFile {
        self.inner.file()
    }

    /// The run, for a call.
    pub(super) fn run(&mut self) -> &mut dyn RecordRun {
        &mut *self.inner
    }

    /// Cancels it as the flow asked and deletes its video at once, as the run may have ended before its stream said so;
    /// a video the port already deleted is no error.
    pub(super) fn cancel(mut self) {
        self.armed = false;
        self.inner.cancel();
        self.delete();
    }

    /// True once its last note was told.
    fn ended(&self) -> bool {
        self.ended.load(Ordering::Acquire)
    }

    /// Deletes its video; a refused delete waits for the next start.
    fn delete(&self) {
        remove(&*self.folder, &self.leftovers, self.inner.file());
    }
}

impl Drop for Run {
    /// A run dropped before its last note is cancelled and its video deleted; an ended one keeps its video.
    fn drop(&mut self) {
        if self.armed && !self.ended() {
            self.inner.cancel();
            self.delete();
        }
    }
}
