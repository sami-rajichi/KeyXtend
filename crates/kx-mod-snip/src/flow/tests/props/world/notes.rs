//! The runs' notes as the host tells them: the newest open run's, a cancelled run's late end, and a note on its way
//! before Cancel reached it.

use super::super::super::*;
use super::super::Said;
use super::World;

impl World {
    /// The newest open run tells `said`; the host deletes a faulted run's video.
    pub(super) fn note(&mut self, said: Said) -> Option<In> {
        let (video, run) = self
            .runs
            .iter_mut()
            .rev()
            .find(|(_, r)| !r.ended && !r.cancelled)?;
        let note = told(said);
        if note.ends() {
            run.ended = true;
        }
        if matches!(said, Said::Fault(_)) {
            self.unsaved.remove(video);
        }
        Some(In::Note {
            video: *video,
            note,
        })
    }

    /// A cancelled run's end, told before Cancel reached it: its video stays.
    pub(super) fn late(&mut self) -> Option<In> {
        let (video, run) = self
            .runs
            .iter_mut()
            .rev()
            .find(|(_, r)| !r.ended && r.cancelled)?;
        run.ended = true;
        self.unsaved.insert(*video);
        let note = done(RecordEnd::Stopped);
        Some(In::Note {
            video: *video,
            note,
        })
    }

    /// A cancelled run's note that was on its way before Cancel reached it; the run has not ended.
    pub(super) fn stale(&self, said: Said) -> Option<In> {
        let (video, _) = self
            .runs
            .iter()
            .rev()
            .find(|(_, r)| !r.ended && r.cancelled)?;
        let note = told(said);
        Some(In::Note {
            video: *video,
            note,
        })
    }
}

/// The note a run tells for `said`.
fn told(said: Said) -> RecordNote {
    match said {
        Said::Time => RecordNote::Time { ms: LENGTH_MS },
        Said::Done(end) => done(end),
        Said::Fault(error) => RecordNote::Fault(error),
        Said::Lost(source) => RecordNote::Lost(source),
    }
}
