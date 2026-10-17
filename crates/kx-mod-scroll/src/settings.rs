//! The `[scroll]` settings: the two speeds, the stops, and how the platform finds, reads and moves targets.

use std::ops::RangeInclusive;

use kx_module_api::ports::ScrollConfig;
use kx_module_api::{Settings, SettingsError, SettingsSpec, validate_as, within};
use serde::{Deserialize, Serialize};

/// Either speed, in lines a second.
pub(crate) const SPEED: RangeInclusive<u32> = 1..=60;
/// The time limit after the last pad tap, in seconds, when there is one.
pub const LIMIT_S: RangeInclusive<u32> = 10..=1_800;
/// The `limit_s` that means no limit.
pub const NO_LIMIT: u32 = 0;
/// The glide's step period, in ms.
pub(crate) const TICK_MS: RangeInclusive<u32> = 8..=100;
/// How long a target may stay still before the platform tries another way, in ms.
pub(crate) const CHECK_MS: RangeInclusive<u32> = 100..=2_000;
/// The longest read of a target, in ms.
pub(crate) const READ_MS: RangeInclusive<u32> = 100..=2_000;
/// How far past the keyboard's edge the target point goes, in physical pixels.
pub(crate) const EDGE_PX: RangeInclusive<u32> = 2..=64;

/// Why a fast speed at or below the slow one was refused.
const FAST_SLOW: &str = "fast must be above slow";
/// Why a tick as long as a check or longer was refused.
const TICK_LONG: &str = "tick_ms must stay below check_ms";

/// How the kernel reads the `[scroll]` section: version 1, with the defaults embedded.
pub static SPEC: SettingsSpec = SettingsSpec {
    version: 1,
    defaults: include_str!("../defaults.toml"),
    migrations: &[],
    validate: validate_as::<ScrollSettings>,
};

/// The module's settings; every value comes from `defaults.toml` or the user's file.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ScrollSettings {
    /// The first tap's speed, in lines a second.
    pub slow: u32,
    /// The second tap's speed, in lines a second; always above `slow`.
    pub fast: u32,
    /// Scrolling stops this many seconds after the last pad tap; 0 means no limit.
    pub limit_s: u32,
    /// Scrolling stops when the target reaches its end.
    pub stop_at_end: bool,
    /// Scrolling stops at a press outside our windows.
    pub stop_on_press: bool,
    /// How often the glide steps while scrolling, in ms; always below `check_ms`, so a check spans more than one tick.
    pub tick_ms: u32,
    /// How many ms a target may stay still under our steps, short of its end, before the platform tries another way;
    /// it also sets how often a scroll aims again, reads the reach and checks the limit, and the glide's longest gap.
    pub check_ms: u32,
    /// The longest one read of a target may take, in ms.
    pub read_ms: u32,
    /// How far past the keyboard's edge the target point goes when the pointer's path is unknown, in physical pixels.
    pub edge_px: u32,
}

impl ScrollSettings {
    /// The settings in the embedded `defaults.toml`.
    ///
    /// # Errors
    /// `SettingsError::Parse` when the defaults do not parse, or the error from `check`.
    pub fn from_defaults() -> Result<Self, SettingsError> {
        SPEC.defaults_as()
    }

    /// How the platform finds, reads and moves targets: the three platform values.
    #[must_use]
    pub fn scroll_config(&self) -> ScrollConfig {
        ScrollConfig {
            check_ms: self.check_ms,
            read_ms: self.read_ms,
            edge_px: self.edge_px,
        }
    }
}

impl Settings for ScrollSettings {
    /// Each range comes first, so a value out of range names its range; then fast above slow, and a tick below a check.
    fn check(&self) -> Result<(), SettingsError> {
        within("slow", self.slow, &SPEED)?;
        within("fast", self.fast, &SPEED)?;
        if self.limit_s != NO_LIMIT && !LIMIT_S.contains(&self.limit_s) {
            let (lo, hi) = (LIMIT_S.start(), LIMIT_S.end());
            let why = format!("limit_s must be {NO_LIMIT} or from {lo} to {hi}");
            return Err(SettingsError::Invalid(why));
        }
        within("tick_ms", self.tick_ms, &TICK_MS)?;
        within("check_ms", self.check_ms, &CHECK_MS)?;
        within("read_ms", self.read_ms, &READ_MS)?;
        within("edge_px", self.edge_px, &EDGE_PX)?;
        if self.fast <= self.slow {
            return Err(SettingsError::Invalid(FAST_SLOW.to_owned()));
        }
        if self.tick_ms >= self.check_ms {
            return Err(SettingsError::Invalid(TICK_LONG.to_owned()));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests;
