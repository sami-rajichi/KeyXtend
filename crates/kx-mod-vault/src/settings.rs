//! The `[vault]` settings (design "Settings `[vault]`", ADR-0038): the notebook's limits, Make one, the backup box and
//! its memory checks, the fill's Tab wait, Paste's wait and the vault worker. Each range is a named const, and each top
//! the worker caps is that cap, from `kx-ipc` through the ports.

use std::ops::RangeInclusive;
use std::time::Duration;

use kx_module_api::ports::{
    BoxConfig, HOURS_PER_DAY, MAX_BLOB_BYTES, MAX_BOX_BYTES, MAX_BOX_WORK, MAX_FIELD_CHARS,
    MAX_VAULT_ENTRIES, MAX_VAULT_FIELDS, MB, MIN_BOX_WORK, MIN_PER_HOUR, S_PER_MIN,
};
use kx_module_api::{Settings, SettingsError, SettingsSpec, refuse_if, validate_as, within};
use serde::{Deserialize, Serialize};

use crate::book::Limits;

/// The characters of one value, name or own label: at least a made and a backup password at their shortest.
pub const FIELD_CHARS: RangeInclusive<usize> = FIELD_LEAST..=MAX_FIELD_CHARS;
/// The most entries.
pub const MAX_ENTRIES: RangeInclusive<usize> = 1..=MAX_VAULT_ENTRIES;
/// The most fields of one entry.
pub const MAX_FIELDS: RangeInclusive<usize> = 1..=MAX_VAULT_FIELDS;
/// The characters of a made password.
pub const MAKE_CHARS: RangeInclusive<usize> = 8..=64;
/// The wait before the boxes are rewritten, in ms.
pub const BOX_DELAY_MS: RangeInclusive<u32> = 250..=60_000;
/// The fewest characters of a backup password.
pub const BOX_MIN_CHARS: RangeInclusive<usize> = 8..=64;
/// Days before one memory check.
pub const CHECK_DAY: RangeInclusive<u16> = 1..=365;
/// How many steps the check days list.
pub const CHECK_STEPS: RangeInclusive<usize> = 1..=12;
/// Wrong answers a memory check takes.
pub const CHECK_TRIES: RangeInclusive<u8> = 1..=10;
/// The wait after a fill's Tab, in ms.
pub const TAB_GAP_MS: RangeInclusive<u32> = 20..=2_000;
/// The largest box, in MB, up to the worker's cap.
pub const BOX_MB: RangeInclusive<u32> = 1..=MAX_BOX_MB;
/// The scrypt work factor of a new padlock, within what the worker makes and opens.
pub const BOX_WORK: RangeInclusive<u8> = MIN_BOX_WORK..=MAX_BOX_WORK;
/// The longest vault worker call, in ms.
pub const VAULT_MS: RangeInclusive<u32> = 2_000..=120_000;
/// Seconds the vault worker may sit idle, as the clipboard's media worker may.
pub const WORKER_IDLE_S: RangeInclusive<u32> = 5..=3_600;
/// The first wait before a failed vault worker starts again, in ms.
pub const RESTART_MS: RangeInclusive<u32> = 100..=30_000;
/// The longest wait before a failed vault worker starts again, in ms.
pub const RESTART_MAX_MS: RangeInclusive<u32> = 1_000..=600_000;
/// How long Paste waits for the clipboard history, in ms: past the clipboard's longest open wait, `open_ms` up to 2000.
pub const PASTE_WAIT_MS: RangeInclusive<u32> = 2500..=10_000;
/// Seconds between looks at a waiting backup place.
pub const PLACE_CHECK_S: RangeInclusive<u32> = 10..=3_600;
/// Hours before a memory check asks again after Later: an hour to a week.
pub const LATER_H: RangeInclusive<u16> = 1..=168;
/// The key of the memory checks switch, which the Backup page writes through the kernel.
pub const MEMORY_CHECKS: &str = "memory_checks";

/// The fewest characters a field may be set to hold: the longer of the shortest made and backup passwords.
const FIELD_LEAST: usize = if *MAKE_CHARS.start() > *BOX_MIN_CHARS.start() {
    *MAKE_CHARS.start()
} else {
    *BOX_MIN_CHARS.start()
};

/// Why a pair was refused.
const BAD_MAKE: &str = "make_chars must not pass field_chars";
const BAD_BOX_MIN: &str = "box_min_chars must not pass field_chars";
const BAD_RESTART: &str = "restart_ms must not pass restart_max_ms";

/// The worker's box cap in MB.
#[allow(
    clippy::cast_possible_truncation,
    reason = "the assert below keeps it within `u32`"
)]
const MAX_BOX_MB: u32 = (MAX_BOX_BYTES as u64 / MB) as u32;
const _: () = assert!(MAX_BOX_BYTES as u64 / MB <= u32::MAX as u64);

/// How the kernel reads the `[vault]` section: version 1, with the defaults embedded.
pub static SPEC: SettingsSpec = SettingsSpec {
    version: 1,
    defaults: include_str!("../defaults.toml"),
    migrations: &[],
    validate: validate_as::<VaultSettings>,
};

/// The module's settings; every value comes from `defaults.toml` or the user's file.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct VaultSettings {
    /// The most characters one value, name or own label holds.
    pub field_chars: usize,
    /// The most entries the notebook holds.
    pub max_entries: usize,
    /// The most fields one entry holds.
    pub max_fields: usize,
    /// The characters of a made password.
    pub make_chars: usize,
    /// The wait after the last change before the boxes are rewritten, in ms.
    pub box_delay_ms: u32,
    /// The fewest characters a backup password takes.
    pub box_min_chars: usize,
    /// Days between memory checks, one step after each right answer; the last one repeats.
    pub check_days: Vec<u16>,
    /// Wrong answers a memory check takes before a new password must be set.
    pub check_tries: u8,
    /// Memory checks ask now and then.
    pub memory_checks: bool,
    /// The wait after the Tab that Both types, in ms.
    pub tab_gap_ms: u32,
    /// The largest box, in MB.
    pub box_mb: u32,
    /// The scrypt work factor (log N) of a new padlock.
    pub box_work: u8,
    /// The longest one vault worker call may take, in ms.
    pub vault_ms: u32,
    /// Seconds the vault worker may sit idle before it stops.
    pub worker_idle_s: u32,
    /// The wait before a failed vault worker starts again, in ms; it doubles with each failure.
    pub restart_ms: u32,
    /// The longest wait before a failed vault worker starts again, in ms.
    pub restart_max_ms: u32,
    /// How long Paste waits for the clipboard history's answer, in ms.
    pub paste_wait_ms: u32,
    /// Seconds between looks at a backup place that waits, and between tries of a failed save.
    pub place_check_s: u32,
    /// Hours before a memory check asks again after Later.
    pub later_h: u16,
}

impl VaultSettings {
    /// The settings in the embedded `defaults.toml`.
    ///
    /// # Errors
    /// `SettingsError::Parse` when the defaults do not parse, or the error from `check`.
    pub fn from_defaults() -> Result<Self, SettingsError> {
        SPEC.defaults_as()
    }

    /// The notebook's limits; a whole notebook fits one sealed blob.
    #[must_use]
    pub fn limits(&self) -> Limits {
        Limits {
            field_chars: self.field_chars,
            max_entries: self.max_entries,
            max_fields: self.max_fields,
            max_bytes: MAX_BLOB_BYTES,
        }
    }

    /// How the platform runs the vault job: the call deadline, the idle stop and the restart waits.
    #[must_use]
    pub const fn box_config(&self) -> BoxConfig {
        BoxConfig {
            deadline_ms: self.vault_ms,
            idle_s: self.worker_idle_s,
            restart_ms: self.restart_ms,
            restart_max_ms: self.restart_max_ms,
        }
    }

    /// The wait after the Tab a fill types.
    #[must_use]
    pub fn tab_gap(&self) -> Duration {
        Duration::from_millis(self.tab_gap_ms.into())
    }

    /// How long Paste waits for the clipboard history's answer.
    #[must_use]
    pub fn paste_wait(&self) -> Duration {
        Duration::from_millis(self.paste_wait_ms.into())
    }

    /// The wait after the last change before the boxes are rewritten.
    #[must_use]
    pub fn box_delay(&self) -> Duration {
        Duration::from_millis(self.box_delay_ms.into())
    }

    /// The wait between looks at a backup place that waits, and between tries of a failed save.
    #[must_use]
    pub fn place_check(&self) -> Duration {
        Duration::from_secs(self.place_check_s.into())
    }

    /// The largest box, in bytes.
    #[must_use]
    pub fn box_cap(&self) -> u64 {
        u64::from(self.box_mb).saturating_mul(MB)
    }

    /// The wait after a memory check's Later.
    #[must_use]
    pub fn later(&self) -> Duration {
        hours(self.later_h.into())
    }

    /// The wait before each memory check, step by step; the last one repeats.
    #[must_use]
    pub fn check_waits(&self) -> Vec<Duration> {
        let day = |d: &u16| hours(u64::from(*d).saturating_mul(HOURS_PER_DAY.unsigned_abs()));
        self.check_days.iter().map(day).collect()
    }
}

/// `n` hours.
fn hours(n: u64) -> Duration {
    let hour = (MIN_PER_HOUR * S_PER_MIN).unsigned_abs();
    Duration::from_secs(n.saturating_mul(hour))
}

impl Settings for VaultSettings {
    /// Each range, then the check days, then the pairs.
    fn check(&self) -> Result<(), SettingsError> {
        within("field_chars", self.field_chars, &FIELD_CHARS)?;
        within("max_entries", self.max_entries, &MAX_ENTRIES)?;
        within("max_fields", self.max_fields, &MAX_FIELDS)?;
        within("make_chars", self.make_chars, &MAKE_CHARS)?;
        within("box_delay_ms", self.box_delay_ms, &BOX_DELAY_MS)?;
        within("box_min_chars", self.box_min_chars, &BOX_MIN_CHARS)?;
        within("check_tries", self.check_tries, &CHECK_TRIES)?;
        within("tab_gap_ms", self.tab_gap_ms, &TAB_GAP_MS)?;
        within("box_mb", self.box_mb, &BOX_MB)?;
        within("box_work", self.box_work, &BOX_WORK)?;
        within("vault_ms", self.vault_ms, &VAULT_MS)?;
        within("worker_idle_s", self.worker_idle_s, &WORKER_IDLE_S)?;
        within("restart_ms", self.restart_ms, &RESTART_MS)?;
        within("restart_max_ms", self.restart_max_ms, &RESTART_MAX_MS)?;
        within("paste_wait_ms", self.paste_wait_ms, &PASTE_WAIT_MS)?;
        within("place_check_s", self.place_check_s, &PLACE_CHECK_S)?;
        within("later_h", self.later_h, &LATER_H)?;
        self.check_days()?;
        refuse_if(self.make_chars > self.field_chars, BAD_MAKE)?;
        refuse_if(self.box_min_chars > self.field_chars, BAD_BOX_MIN)?;
        refuse_if(self.restart_ms > self.restart_max_ms, BAD_RESTART)
    }
}

impl VaultSettings {
    /// The check days: a count within `CHECK_STEPS`, each within `CHECK_DAY`, each longer than the one before.
    fn check_days(&self) -> Result<(), SettingsError> {
        let days = &self.check_days;
        let counted = CHECK_STEPS.contains(&days.len());
        let each = days.iter().all(|d| CHECK_DAY.contains(d));
        let rising = days.windows(2).all(|w| w[0] < w[1]);
        if counted && each && rising {
            return Ok(());
        }
        let (day, steps) = (&CHECK_DAY, &CHECK_STEPS);
        Err(SettingsError::Invalid(format!(
            "check_days must rise, each from {} to {} days, {} to {} of them",
            day.start(),
            day.end(),
            steps.start(),
            steps.end()
        )))
    }
}

#[cfg(test)]
mod tests;
