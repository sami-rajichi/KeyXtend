//! The worker jobs' waits and caps that the `[snip]` settings hold: a recording's and a re-timed copy's steps, and the
//! `pick` job's dialogs. Each value is refused outside its range, and the settings check against these ranges.

use super::file::AskError;
use std::ops::RangeInclusive;

/// How long a recording's or a re-timed copy's job may take at each step, in ms.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct RecordWaits {
    /// The longest a run's job may take to say it started: the worker's start, the codecs' load and a first frame.
    pub start_ms: u32,
    /// The longest a run may stay silent, as it tells its time each second.
    pub tick_ms: u32,
    /// The longest a job may take to end a run after Stop, or to quit after goodbye.
    pub end_ms: u32,
    /// The longest a switch waits for its answer.
    pub switch_ms: u32,
    /// A copy's least deadline, as it cannot be cancelled.
    pub speed_ms: u32,
    /// A copy's more deadline for each MB of the video copied.
    pub speed_mb_ms: u32,
}

impl RecordWaits {
    /// `start_ms`' range.
    pub const START_MS: RangeInclusive<u32> = 5_000..=120_000;
    /// `tick_ms`' range, above the job's one tick a second.
    pub const TICK_MS: RangeInclusive<u32> = 2_000..=60_000;
    /// `end_ms`' range.
    pub const END_MS: RangeInclusive<u32> = 5_000..=300_000;
    /// `switch_ms`' range.
    pub const SWITCH_MS: RangeInclusive<u32> = 1_000..=60_000;
    /// `speed_ms`' range.
    pub const SPEED_MS: RangeInclusive<u32> = 5_000..=600_000;
    /// `speed_mb_ms`' range.
    pub const SPEED_MB_MS: RangeInclusive<u32> = 500..=60_000;

    /// The name of the first wait outside its range.
    #[must_use]
    pub(crate) fn bad(&self) -> Option<&'static str> {
        first_bad!(self,
            start_ms: Self::START_MS,
            tick_ms: Self::TICK_MS,
            end_ms: Self::END_MS,
            switch_ms: Self::SWITCH_MS,
            speed_ms: Self::SPEED_MS,
            speed_mb_ms: Self::SPEED_MB_MS,
        )
    }
}

/// The `pick` job's memory cap and waits.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct PickLimits {
    /// The job's memory cap, in MB, as the shell's extensions may load into its dialogs.
    pub job_mb: u32,
    /// How long a dialog may wait on the user, in minutes, before its worker is ended.
    pub dialog_min: u16,
    /// How long the worker waits for the next dialog before it stops, in seconds.
    pub idle_s: u16,
    /// The wait before a failed worker starts again, in ms; it doubles with each failure in a row.
    pub restart_ms: u32,
    /// The longest wait before a failed worker starts again, in ms.
    pub restart_max_ms: u32,
}

impl PickLimits {
    /// `job_mb`'s range.
    pub const JOB_MB: RangeInclusive<u32> = 32..=1_024;
    /// `dialog_min`'s range.
    pub const DIALOG_MIN: RangeInclusive<u16> = 5..=480;
    /// `idle_s`' range.
    pub const IDLE_S: RangeInclusive<u16> = 10..=3_600;
    /// `restart_ms`' range.
    pub const RESTART_MS: RangeInclusive<u32> = 100..=30_000;
    /// `restart_max_ms`' range.
    pub const RESTART_MAX_MS: RangeInclusive<u32> = 1_000..=600_000;

    /// The name of the first value outside its range, or `restart_ms` when it passes `restart_max_ms`.
    #[must_use]
    pub(crate) fn bad(&self) -> Option<&'static str> {
        let order = (self.restart_ms > self.restart_max_ms).then_some("restart_ms");
        first_bad!(self,
            job_mb: Self::JOB_MB,
            dialog_min: Self::DIALOG_MIN,
            idle_s: Self::IDLE_S,
            restart_ms: Self::RESTART_MS,
            restart_max_ms: Self::RESTART_MAX_MS,
        )
        .or(order)
    }

    /// Refuses limits with a value outside its range, or a first wait longer than the longest.
    ///
    /// # Errors
    /// `AskError::Range` naming the first such value.
    pub fn check(&self) -> Result<(), AskError> {
        self.bad().map_or(Ok(()), |name| Err(AskError::Range(name)))
    }
}
