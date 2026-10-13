//! The `[mouse]` settings: Right-click mode, the hold's time and radius, the ring, and how the hook's host runs.

use std::ops::RangeInclusive;

use kx_module_api::ports::HookConfig;
use kx_module_api::{Settings, SettingsError, SettingsSpec, validate_as, within};
use serde::{Deserialize, Serialize};

/// How long a still press waits before it right-clicks, in ms: 0.5 s to 3.0 s, as spec §5.1 says.
pub(crate) const HOLD_MS: RangeInclusive<u32> = 500..=3_000;
/// How far a still press may move, in logical pixels.
pub(crate) const STILL_PX: RangeInclusive<u32> = 1..=40;
/// How long after the press the ring shows, in ms; it ends before the shortest hold.
pub(crate) const RING_AFTER_MS: RangeInclusive<u32> = 0..=400;
/// The health check's period, in ms.
pub(crate) const HEALTH_MS: RangeInclusive<u32> = 250..=10_000;
/// How long the hooks may miss input before they are put back, in ms.
pub(crate) const HEALTH_SLACK_MS: RangeInclusive<u32> = 100..=5_000;
/// How long input waits behind our own before it goes on anyway, in ms.
pub(crate) const ECHO_MS: RangeInclusive<u32> = 20..=2_000;

/// Why a ring at or past the shortest hold was refused.
const RING_LATE: &str = "ring_after_ms must stay below the shortest hold_ms";

/// How the kernel reads the `[mouse]` section: version 1, with the defaults embedded.
pub static SPEC: SettingsSpec = SettingsSpec {
    version: 1,
    defaults: include_str!("../defaults.toml"),
    migrations: &[],
    validate: validate_as::<MouseSettings>,
};

/// The module's settings; every value comes from `defaults.toml` or the user's file.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MouseSettings {
    /// Right-click mode is on; the module writes it on each toggle.
    pub rclick: bool,
    /// A still press right-clicks after this, in ms.
    pub hold_ms: u32,
    /// Moves up to this far, in logical pixels scaled by the screen's DPI, still count as still.
    pub still_px: u32,
    /// The ring shows this long after the press, in ms.
    pub ring_after_ms: u32,
    /// How often the health check runs while the hook runs, in ms.
    pub health_ms: u32,
    /// How long the hooks may miss input before they are put back, in ms.
    pub health_slack_ms: u32,
    /// How long input waits behind our own before it goes on anyway, in ms.
    pub echo_ms: u32,
}

impl MouseSettings {
    /// The settings in the embedded `defaults.toml`.
    ///
    /// # Errors
    /// `SettingsError::Parse` when the defaults do not parse, or the error from `check`.
    pub fn from_defaults() -> Result<Self, SettingsError> {
        SPEC.defaults_as()
    }

    /// How the pointer hook's host runs: the three host values.
    #[must_use]
    pub fn hook_config(&self) -> HookConfig {
        HookConfig {
            health_ms: self.health_ms,
            health_slack_ms: self.health_slack_ms,
            echo_ms: self.echo_ms,
        }
    }
}

impl Settings for MouseSettings {
    /// The ring's cross-rule comes first, so a ring at or past the shortest hold says why.
    fn check(&self) -> Result<(), SettingsError> {
        if self.ring_after_ms >= *HOLD_MS.start() {
            return Err(SettingsError::Invalid(RING_LATE.to_owned()));
        }
        within("hold_ms", self.hold_ms, &HOLD_MS)?;
        within("still_px", self.still_px, &STILL_PX)?;
        within("ring_after_ms", self.ring_after_ms, &RING_AFTER_MS)?;
        within("health_ms", self.health_ms, &HEALTH_MS)?;
        within("health_slack_ms", self.health_slack_ms, &HEALTH_SLACK_MS)?;
        within("echo_ms", self.echo_ms, &ECHO_MS)
    }
}

#[cfg(test)]
mod tests;
