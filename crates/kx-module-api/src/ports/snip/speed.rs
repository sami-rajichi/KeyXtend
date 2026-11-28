//! A video's speed (design "Videos", ADR-0060): a checked speed the frames list, and the ask for a re-timed copy, which
//! the `record` job writes into the unsaved part only.

use super::file::{FileKind, SnipFile};
use super::record::{RecordAsk, RecordError, refuse};
use super::waits::RecordWaits;

/// How many times faster a video plays, one of `SpeedAsk::SPEEDS`; 1 plays it as it is.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Speed(u8);

impl Speed {
    /// The first of `SpeedAsk::SPEEDS`, which plays a video as recorded.
    pub const PLAIN: Self = Self(SpeedAsk::SPEEDS[0]);

    /// `times` as a speed; `None` unless `SpeedAsk::SPEEDS` lists it.
    #[must_use]
    pub fn new(times: u8) -> Option<Self> {
        SpeedAsk::SPEEDS.contains(&times).then_some(Self(times))
    }

    /// How many times faster.
    #[must_use]
    pub const fn times(self) -> u8 {
        self.0
    }

    /// The speed after this one in `SpeedAsk::SPEEDS`, as each Speed click steps; after the last, `PLAIN` again.
    #[must_use]
    pub fn next(self) -> Self {
        let at = SpeedAsk::SPEEDS.iter().position(|&times| times == self.0);
        let next = at.and_then(|at| SpeedAsk::SPEEDS.get(at + 1));
        next.map_or(Self::PLAIN, |&times| Self(times))
    }
}

/// A re-timed copy of a video into a new video of the unsaved part, its frames dropped, as Speed and a Copy or Save at
/// that speed ask; a Save then copies it through the dialog's answer, so the job writes nowhere else.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SpeedAsk {
    /// The video copied.
    pub from: SnipFile,
    /// How many times faster it plays.
    pub speed: Speed,
    /// No copy starts while its drive holds less than this many MB of `MB` bytes (`min_free_mb`).
    pub min_free_mb: u32,
    /// The job's memory cap, in MB of `MB` bytes (`record_mb`).
    pub job_mb: u32,
    /// How long the job may take: its copy's deadline and its goodbye.
    pub waits: RecordWaits,
}

impl SpeedAsk {
    /// The speeds, the frames' own.
    pub const SPEEDS: [u8; 3] = kx_ipc::SNIP_SPEEDS;

    /// Refuses an ask naming a picture, or with a floor, a memory cap or a wait outside its range.
    ///
    /// # Errors
    /// `RecordError::BadPath` for a picture, else `Range` naming the first such value.
    pub fn check(&self) -> Result<(), RecordError> {
        if self.from.kind() != FileKind::Mp4 {
            return Err(RecordError::BadPath);
        }
        let bad = first_bad!(self,
            min_free_mb: RecordAsk::MIN_FREE_MB,
            job_mb: RecordAsk::JOB_MB,
        );
        refuse(bad.or_else(|| self.waits.bad()))
    }
}
