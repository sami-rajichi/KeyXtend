//! The snip's videos in the unsaved part by the flow's numbers, their deletes, and the sweep at each start (edge
//! 15): a video marked copied stays `copied_days` from its mark, and every other PNG and MP4 no job uses goes. Only
//! files the folder lists or the recorder made are deleted, so nothing outside the unsaved part is touched.

use std::collections::{HashMap, HashSet};
use std::time::Duration;

use kx_module_api::ports::{HOURS_PER_DAY, MB, MIN_PER_HOUR, S_PER_MIN, SnipFile, SnipFolder};

use super::ports::held;
use super::{Actor, Leftovers};
use crate::flow::Video;

/// Seconds in a day, from the calendar's own counts.
const DAY_S: i64 = S_PER_MIN * MIN_PER_HOUR * HOURS_PER_DAY;

/// The running snip's videos, and the re-timed copies deleted before their job answered.
#[derive(Debug, Default)]
pub(super) struct Files {
    made: HashMap<Video, SnipFile>,
    /// Each goes once its Copy or Save answers, which also forgets it.
    doomed: HashSet<Video>,
}

impl Files {
    /// The file of `video`, if known.
    pub(super) fn file(&self, video: Video) -> Option<SnipFile> {
        self.made.get(&video).cloned()
    }

    /// The snip ended: its videos are forgotten, while a copy still awaited stays doomed until its answer.
    pub(super) fn forget(&mut self) {
        self.made.clear();
    }

    /// The videos known or awaited.
    #[cfg(test)]
    pub(super) fn count(&self) -> usize {
        self.made.len() + self.doomed.len()
    }
}

/// Deletes `file` from `folder`'s unsaved part; one it refuses joins `leftovers`, which the next sweep tries again.
pub(super) fn remove(folder: &dyn SnipFolder, leftovers: &Leftovers, file: &SnipFile) {
    if let Err(error) = folder.remove(file) {
        tracing::warn!(%error, "a video could not be deleted, so the next start tries again");
        held(leftovers).missed.push(file.clone());
    }
}

/// `count` days as a span.
fn days(count: u8) -> Duration {
    let day = u64::try_from(DAY_S).unwrap_or(u64::MAX);
    Duration::from_secs(day.saturating_mul(u64::from(count)))
}

impl Actor {
    /// `video` was written into `file`; one the flow deleted meanwhile goes at once.
    pub(super) fn made(&mut self, video: Video, file: SnipFile) {
        if self.files.doomed.remove(&video) {
            self.remove(&file);
        } else {
            self.files.made.insert(video, file);
        }
    }

    /// A Copy or Save answered, with the re-timed copy `into` it was asked for: written as `made`, or never written or
    /// already deleted by its job.
    pub(super) fn copied_into(&mut self, into: Option<Video>, made: Option<SnipFile>) {
        match (into, made) {
            (Some(into), Some(file)) => self.made(into, file),
            (Some(into), None) => {
                self.files.doomed.remove(&into);
            }
            (None, _) => {}
        }
    }

    /// The flow deletes `video`: a Copy or Save in flight using it deletes it once done, a copy still awaited goes once
    /// its job answers, and a re-timed copy never written needs nothing.
    pub(super) fn delete(&mut self, video: Video) {
        if self.keeping.defer(video) {
            self.files.made.remove(&video);
            return;
        }
        if let Some(file) = self.files.made.remove(&video) {
            self.remove(&file);
        } else if self.keeping.awaits(video) {
            self.files.doomed.insert(video);
        }
    }

    /// True when the Snips folder's drive holds less than the snip's `min_free_mb` (edge 8); a drive the folder cannot
    /// read is not low, as the recorder checks the room again at its start.
    pub(super) fn low(&self) -> bool {
        match self.ports.folder.free() {
            Ok(free) => free < u64::from(self.held.min_free_mb).saturating_mul(MB),
            Err(error) => {
                tracing::warn!(%error, "the Snips folder's room could not be read");
                false
            }
        }
    }

    /// Deletes `file` of the unsaved part; one the folder refuses is tried again at the next start.
    pub(super) fn remove(&self, file: &SnipFile) {
        remove(&*self.ports.folder, &self.ports.leftovers, file);
    }

    /// The sweep of the unsaved part at the module's start and each snip's: a delete missed earlier goes even when the
    /// list fails. Then every PNG and MP4 that no job uses goes but a video marked copied within `copied_days`, and
    /// marks whose video is gone go too (edge 15).
    pub(super) fn sweep(&mut self) {
        if self.seen.hidden {
            return;
        }
        let (missed, used) = {
            let mut held = held(&self.ports.leftovers);
            (std::mem::take(&mut held.missed), held.used.clone())
        };
        for file in &missed {
            if used.contains(file) {
                held(&self.ports.leftovers).missed.push(file.clone());
            } else {
                self.remove(file);
            }
        }
        let listed = match self.ports.folder.unsaved() {
            Ok(listed) => listed,
            Err(error) => return tracing::warn!(%error, "the unsaved part could not be swept"),
        };
        let kept = days(self.settings.copied_days);
        for found in listed {
            let fresh = found.copied.is_some_and(|age| age < kept);
            if !fresh && !missed.contains(&found.file) && !used.contains(&found.file) {
                self.remove(&found.file);
            }
        }
        if let Err(error) = self.ports.folder.prune_marks() {
            tracing::warn!(%error, "the unsaved part's old copy marks could not be deleted");
        }
    }
}
