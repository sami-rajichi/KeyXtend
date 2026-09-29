//! A module's live handle to its own settings section, for values it sets itself.

use crate::{ModuleId, WriteError};
use std::fmt;
use std::sync::Arc;

/// The kernel side of every settings handle; object-safe, so one store serves all modules.
pub trait SettingsSink: Send + Sync {
    /// Checks `value` for `key` in `module`'s section as a Settings-window change would, stores it and saves.
    ///
    /// # Errors
    /// `WriteError` when the key or value is refused, or the file could not be saved.
    fn write(&self, module: ModuleId, key: &str, value: toml::Value) -> Result<(), WriteError>;
}

/// Writes one module's own section at once, from any thread.
/// It never restarts the module or publishes `SettingsChanged`, since the module already knows the value.
#[derive(Clone)]
pub struct SettingsHandle {
    module: ModuleId,
    sink: Arc<dyn SettingsSink>,
}

impl SettingsHandle {
    /// A handle bound to `module`'s section; only `ModuleCx` makes one, from the manifest id.
    pub(crate) fn new(module: ModuleId, sink: Arc<dyn SettingsSink>) -> Self {
        Self { module, sink }
    }

    /// Checks, stores and saves one value of the module's own section.
    /// If the settings file was locked at start, the value holds for this run only and `set` succeeds.
    ///
    /// # Errors
    /// `Invalid` for an unknown key or a refused value, `Refused` when the section cannot change,
    /// or `Unsaved` when the file could not be written.
    pub fn set(&self, key: &str, value: toml::Value) -> Result<(), WriteError> {
        self.sink.write(self.module, key, value)
    }
}

impl fmt::Debug for SettingsHandle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut out = f.debug_struct("SettingsHandle");
        out.field("module", &self.module).finish_non_exhaustive()
    }
}
