//! The kernel's own settings section: the switched-off modules and the log limits.

use super::{KERNEL, Kernel};
use crate::log;
use kx_module_api::{ModuleId, Settings, SettingsError, SettingsSpec, parse_as, validate_as};
use serde::{Deserialize, Serialize};

/// The smallest log file cap, in MiB.
pub(crate) const MIN_LOG_MB: u32 = 1;
/// The largest log file cap, in MiB, so a hand-edited value cannot fill the disk.
pub(crate) const MAX_LOG_MB: u32 = 100;

/// The key of the switched-off list in the kernel's section.
pub(super) const DISABLED_KEY: &str = "disabled";

/// How the store reads the kernel's section: version 1, with the defaults embedded.
pub(super) static SPEC: SettingsSpec = SettingsSpec {
    version: 1,
    defaults: include_str!("../../defaults.toml"),
    migrations: &[],
    validate: validate_as::<KernelSettings>,
};

/// The kernel's settings: switched-off modules and the log limits.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct KernelSettings {
    /// Ids of the modules the user switched off.
    pub disabled: Vec<String>,
    /// The least important level written, one of `log::LOG_LEVELS`.
    pub log_level: String,
    /// The largest log file, in MiB.
    pub log_max_mb: u32,
}

impl KernelSettings {
    /// The settings in the embedded `defaults.toml`, for a log opened before any settings load.
    ///
    /// # Errors
    /// `SettingsError::Parse` when the defaults do not parse, or the error from `check`.
    pub fn from_defaults() -> Result<Self, SettingsError> {
        SPEC.defaults_as()
    }
}

impl Settings for KernelSettings {
    fn check(&self) -> Result<(), SettingsError> {
        let invalid = |why: &str| Err(SettingsError::Invalid(why.to_owned()));
        if log::level(&self.log_level).is_none() {
            return invalid("log_level is not a level name");
        }
        if !(MIN_LOG_MB..=MAX_LOG_MB).contains(&self.log_max_mb) {
            return invalid("log_max_mb is outside its range");
        }
        if self.disabled.iter().any(String::is_empty) {
            return invalid("disabled holds an empty id");
        }
        Ok(())
    }
}

impl Kernel {
    /// The kernel's own settings, already repaired, so the app can open its log before any module
    /// starts; `None` before `load`.
    #[must_use]
    pub fn kernel_settings(&self) -> Option<KernelSettings> {
        let table = self.store.get(KERNEL)?;
        parse_as(&table).ok()
    }

    /// The ids the user switched off; none before load.
    pub(super) fn disabled(&self) -> Vec<String> {
        self.kernel_settings()
            .map(|s| s.disabled)
            .unwrap_or_default()
    }

    pub(super) fn is_disabled(&self, id: ModuleId) -> bool {
        switched_off(&self.disabled(), id)
    }
}

/// True when module `id` is in `list`, the ids the user switched off.
pub(super) fn switched_off(list: &[String], id: ModuleId) -> bool {
    list.iter().any(|d| d == id.as_str())
}

#[cfg(test)]
mod tests;
