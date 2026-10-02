//! A settings sink that checks each write against the whole section, as the kernel's store does.

use kx_module_api::{
    ModuleId, Settings, SettingsError, SettingsSink, SettingsSpec, WriteError, parse_as,
};
use kx_settings::VERSION_KEY;
use std::sync::{Mutex, MutexGuard, PoisonError};
use toml::Table;

/// One accepted write: the module and its values.
pub type Written = (ModuleId, Table);

/// Locks `mutex`; a test that panicked while holding it does not stop the others.
pub(super) fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex.lock().unwrap_or_else(PoisonError::into_inner)
}

/// Stores writes in one section: checked together by the spec when there is one, all or nothing.
/// As in the kernel's store, a write that holds the version key is refused before any check.
#[derive(Debug, Default)]
pub struct FakeSink {
    spec: Option<&'static SettingsSpec>,
    section: Mutex<Table>,
    writes: Mutex<Vec<Written>>,
    refusal: Mutex<Option<WriteError>>,
    refused_key: Mutex<Option<(String, WriteError)>>,
}

impl FakeSink {
    /// A sink over `section`, checking each write with `spec` when given.
    #[must_use]
    pub fn new(spec: Option<&'static SettingsSpec>, section: Table) -> Self {
        Self {
            spec,
            section: Mutex::new(section),
            ..Self::default()
        }
    }

    /// Every write the file got, oldest first; an unsaved one is not listed.
    #[must_use]
    pub fn writes(&self) -> Vec<Written> {
        lock(&self.writes).clone()
    }

    /// The section as it is now.
    #[must_use]
    pub fn section(&self) -> Table {
        lock(&self.section).clone()
    }

    /// The section read as `T`.
    ///
    /// # Errors
    /// The `SettingsError` of `parse_as`.
    pub fn read_as<T: Settings>(&self) -> Result<T, SettingsError> {
        parse_as(&self.section())
    }

    /// Lays `values` over the section without a write, as a user's file would.
    pub fn put(&self, values: Table) {
        lock(&self.section).extend(values);
    }

    /// Refuses every later write with `error`, or takes them again with `None`. As in the kernel's store, an
    /// `Unsaved` write still lands in the section once checked; any other refusal keeps nothing.
    pub fn refuse(&self, error: Option<WriteError>) {
        *lock(&self.refusal) = error;
    }

    /// Refuses every later write that holds `key`, with `error`, as `refuse` does.
    pub fn refuse_key(&self, key: &str, error: WriteError) {
        *lock(&self.refused_key) = Some((key.to_owned(), error));
    }

    /// The refusal due for a write of `values`, if any.
    fn refusal_for(&self, values: &Table) -> Option<WriteError> {
        let keyed = lock(&self.refused_key).clone();
        let keyed = keyed.filter(|(key, _)| values.contains_key(key));
        lock(&self.refusal).clone().or(keyed.map(|(_, e)| e))
    }
}

impl SettingsSink for FakeSink {
    fn write(&self, module: ModuleId, values: Table) -> Result<(), WriteError> {
        let refusal = self.refusal_for(&values);
        if let Some(error) = refusal.clone().filter(|e| !e.holds()) {
            return Err(error);
        }
        if values.contains_key(VERSION_KEY) {
            return Err(WriteError::Refused(module));
        }
        let mut next = self.section();
        next.extend(values.clone());
        if let Some(spec) = self.spec {
            (spec.validate)(&next)?;
        }
        *lock(&self.section) = next;
        if let Some(unsaved) = refusal {
            return Err(unsaved);
        }
        lock(&self.writes).push((module, values));
        Ok(())
    }
}
