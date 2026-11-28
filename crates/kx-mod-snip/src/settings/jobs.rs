//! The jobs' waits and caps in `[snip]`: a recording's and a speed copy's steps and the dialogs' limits, whose ranges
//! are the ports' own, and a picture's PNG in the media worker, whose ranges are the module's.

use std::ops::RangeInclusive;

use kx_module_api::ports::{PickLimits, RecordWaits};
use kx_module_api::{SettingsError, refuse_if, within};

use super::SnipSettings;

/// The least time a picture's PNG may take before the media worker is stopped, in ms.
pub const ENCODE_MS: RangeInclusive<u32> = 1_000..=60_000;
/// The more time a PNG may take for each MB of its pixels, in ms.
pub const ENCODE_MB_MS: RangeInclusive<u32> = 10..=2_000;
/// How long the media worker may idle after a PNG, in seconds.
pub const ENCODE_IDLE_S: RangeInclusive<u32> = 1..=300;
/// The wait before a failed media worker starts again, in ms.
pub const ENCODE_RESTART_MS: RangeInclusive<u32> = 100..=30_000;

/// Why the dialogs' first restart wait was refused.
const BAD_RESTART: &str = "pick_restart_ms must not pass pick_restart_max_ms";

/// How long a picture's PNG may take in the media worker, and how that worker idles and starts again.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct EncodeWaits {
    /// The least time a PNG may take, in ms.
    pub ms: u32,
    /// The more time for each MB of its pixels, in ms.
    pub mb_ms: u32,
    /// How long the worker may idle, in seconds.
    pub idle_s: u32,
    /// The wait before a failed worker starts again, in ms.
    pub restart_ms: u32,
}

impl SnipSettings {
    /// How long a recording's or a speed copy's job may take at each step.
    #[must_use]
    pub const fn record_waits(&self) -> RecordWaits {
        RecordWaits {
            start_ms: self.record_start_ms,
            tick_ms: self.record_tick_ms,
            end_ms: self.record_end_ms,
            switch_ms: self.switch_ms,
            speed_ms: self.speed_ms,
            speed_mb_ms: self.speed_mb_ms,
        }
    }

    /// The dialogs' job memory cap and waits.
    #[must_use]
    pub const fn pick_limits(&self) -> PickLimits {
        PickLimits {
            job_mb: self.pick_mb,
            dialog_min: self.dialog_min,
            idle_s: self.pick_idle_s,
            restart_ms: self.pick_restart_ms,
            restart_max_ms: self.pick_restart_max_ms,
        }
    }

    /// How long a picture's PNG may take, and how its worker idles and starts again.
    #[must_use]
    pub const fn encode_waits(&self) -> EncodeWaits {
        EncodeWaits {
            ms: self.encode_ms,
            mb_ms: self.encode_mb_ms,
            idle_s: self.encode_idle_s,
            restart_ms: self.encode_restart_ms,
        }
    }

    /// The jobs' waits and the dialogs' limits by the ports' ranges, the restart waits' order, and the PNG's waits.
    pub(super) fn check_jobs(&self) -> Result<(), SettingsError> {
        within(
            "record_start_ms",
            self.record_start_ms,
            &RecordWaits::START_MS,
        )?;
        within("record_tick_ms", self.record_tick_ms, &RecordWaits::TICK_MS)?;
        within("record_end_ms", self.record_end_ms, &RecordWaits::END_MS)?;
        within("switch_ms", self.switch_ms, &RecordWaits::SWITCH_MS)?;
        within("speed_ms", self.speed_ms, &RecordWaits::SPEED_MS)?;
        within("speed_mb_ms", self.speed_mb_ms, &RecordWaits::SPEED_MB_MS)?;
        within("pick_mb", self.pick_mb, &PickLimits::JOB_MB)?;
        within("dialog_min", self.dialog_min, &PickLimits::DIALOG_MIN)?;
        within("pick_idle_s", self.pick_idle_s, &PickLimits::IDLE_S)?;
        within(
            "pick_restart_ms",
            self.pick_restart_ms,
            &PickLimits::RESTART_MS,
        )?;
        let most = &PickLimits::RESTART_MAX_MS;
        within("pick_restart_max_ms", self.pick_restart_max_ms, most)?;
        refuse_if(self.pick_restart_ms > self.pick_restart_max_ms, BAD_RESTART)?;
        within("encode_ms", self.encode_ms, &ENCODE_MS)?;
        within("encode_mb_ms", self.encode_mb_ms, &ENCODE_MB_MS)?;
        within("encode_idle_s", self.encode_idle_s, &ENCODE_IDLE_S)?;
        within(
            "encode_restart_ms",
            self.encode_restart_ms,
            &ENCODE_RESTART_MS,
        )
    }
}
