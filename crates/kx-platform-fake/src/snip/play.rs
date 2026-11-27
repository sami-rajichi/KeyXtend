//! A fake player for Snip's videos: it plays only a video held directly in the Snips folder or its unsaved part,
//! records every video asked, and refuses while the test sets an error. Its `Debug` prints counts, never a path.

use super::folder::FakeSnipFolder;
use crate::{lock, wait_for};
use kx_module_api::ports::{FileKind, MediaOpen, PlayError, SnipFile};
use std::fmt;
use std::sync::{Arc, Condvar, Mutex};
use std::time::Duration;

/// What the player was asked and how it answers.
#[derive(Default)]
struct State {
    /// Every video asked, oldest first.
    played: Vec<SnipFile>,
    /// The error later plays give, once a video is found.
    error: Option<PlayError>,
}

/// The state, and the signal each play gives once answered, for a test waiting on another thread.
#[derive(Default)]
struct Shared {
    state: Mutex<State>,
    changed: Condvar,
}

/// A player shared by its clones, which finds videos in `folder`'s table.
#[derive(Clone)]
pub struct FakeMediaOpen {
    shared: Arc<Shared>,
    folder: FakeSnipFolder,
}

impl FakeMediaOpen {
    /// A player with no play yet, finding videos in `folder`'s table.
    #[must_use]
    pub fn new(folder: &FakeSnipFolder) -> Self {
        let shared = Arc::default();
        let folder = folder.clone();
        Self { shared, folder }
    }

    /// Every video asked, refused ones too, oldest first.
    #[must_use]
    pub fn played(&self) -> Vec<SnipFile> {
        lock(&self.shared.state).played.clone()
    }

    /// Refuses later plays of a found video with `error`, or takes them again with `None`.
    pub fn set_error(&self, error: Option<PlayError>) {
        lock(&self.shared.state).error = error;
    }

    /// Waits up to `within` until `count` plays were answered, so a test never sleeps; true when they were.
    #[must_use]
    pub fn wait_played(&self, count: usize, within: Duration) -> bool {
        let shared = &self.shared;
        wait_for(&shared.state, &shared.changed, within, |s| {
            s.played.len() >= count
        })
    }

    /// The answer to playing `video` when `error` is set.
    fn answer(&self, video: &SnipFile, error: Option<PlayError>) -> Result<(), PlayError> {
        if video.kind() != FileKind::Mp4 || !self.folder.in_snips(video) {
            return Err(PlayError::BadPath);
        }
        if self.folder.stored(video).is_none() {
            return Err(PlayError::Missing);
        }
        error.map_or(Ok(()), Err)
    }
}

/// Shows the count, never a path.
impl fmt::Debug for FakeMediaOpen {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let state = lock(&self.shared.state);
        f.debug_struct("FakeMediaOpen")
            .field("played", &state.played.len())
            .field("error", &state.error)
            .finish_non_exhaustive()
    }
}

impl MediaOpen for FakeMediaOpen {
    fn play(&self, video: &SnipFile) -> Result<(), PlayError> {
        let mut state = lock(&self.shared.state);
        let answer = self.answer(video, state.error);
        state.played.push(video.clone());
        drop(state);
        self.shared.changed.notify_all();
        answer
    }
}

#[cfg(test)]
mod tests;
