//! The settings store and the one notice path the kernel shares with every module's settings handle.
//! No lock is held while a module runs or the bus publishes, so a handler may write settings.

use crate::bus::announce;
use kx_module_api::{Bus, ModuleId, Notice, SettingsSink, WriteError};
use kx_settings::{Store, StoreError};
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};
use toml::{Table, Value};

/// The store, once loaded, and the notice path: the bus, plus the list `load` or `start` returns.
#[derive(Clone)]
pub struct Shared {
    store: Arc<Mutex<Option<Store>>>,
    /// The notices kept while `load` or `start` runs, from the kernel and from handles alike.
    told: Arc<Mutex<Option<Vec<Notice>>>>,
    bus: Bus,
}

impl Shared {
    /// An empty share; `put` fills it when the kernel loads.
    #[must_use]
    pub fn new(bus: Bus) -> Self {
        Self {
            store: Arc::new(Mutex::new(None)),
            told: Arc::new(Mutex::new(None)),
            bus,
        }
    }

    /// Holds `store` from now on.
    pub fn put(&self, store: Store) {
        *lock(&self.store) = Some(store);
    }

    /// A copy of `module`'s section, ready for `parse_as`; `None` before load or without settings.
    #[must_use]
    pub fn get(&self, module: ModuleId) -> Option<Table> {
        lock(&self.store).as_ref()?.get(module).cloned()
    }

    /// Sets one value of `module`'s section; true when the stored value changed.
    ///
    /// # Errors
    /// `Unknown` before load, or the store's refusal; nothing changes then.
    pub fn set(&self, module: ModuleId, key: &str, value: Value) -> Result<bool, StoreError> {
        self.set_all(module, Table::from_iter([(key.to_owned(), value)]))
    }

    /// Sets several values of `module`'s section, checked together; true when the stored values changed.
    ///
    /// # Errors
    /// `Unknown` before load, or the store's refusal; nothing changes then.
    pub fn set_all(&self, module: ModuleId, values: Table) -> Result<bool, StoreError> {
        let mut guard = lock(&self.store);
        let store = guard.as_mut().ok_or(StoreError::Unknown(module))?;
        store.set_all(module, values)
    }

    /// Saves when the file is behind; a failure is logged and gives back the notice due, if any.
    ///
    /// # Errors
    /// The `settings-unsaved` notice, or `None` when the user was already told.
    pub fn save(&self) -> Result<(), Option<Notice>> {
        let mut guard = lock(&self.store);
        let Some(store) = guard.as_mut().filter(|s| s.needs_save()) else {
            return Ok(());
        };
        store.save().map_err(|unsaved| {
            tracing::warn!(error = %unsaved.error, "settings not saved");
            unsaved.notice
        })
    }

    /// This share as the sink behind every module's settings handle.
    #[must_use]
    pub fn sink(&self) -> Arc<dyn SettingsSink> {
        Arc::new(self.clone())
    }

    /// Publishes `notice`, and keeps it while `load` or `start` runs so their list holds it too.
    pub fn notify(&self, notice: Notice) {
        announce(&self.bus, &notice);
        if let Some(told) = lock(&self.told).as_mut() {
            told.push(notice);
        }
    }

    /// Starts keeping notices for `load` or `start` to return.
    pub fn collect(&self) {
        *lock(&self.told) = Some(Vec::new());
    }

    /// Stops keeping notices and returns the ones kept.
    #[must_use]
    pub fn collected(&self) -> Vec<Notice> {
        lock(&self.told).take().unwrap_or_default()
    }
}

/// Locks `mutex`; a panic on another thread that held it does not stop the kernel.
pub(crate) fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex.lock().unwrap_or_else(PoisonError::into_inner)
}

impl SettingsSink for Shared {
    /// Sets all the values and saves once; a failed save tells its notice after the store lock is released.
    fn write(&self, module: ModuleId, values: Table) -> Result<(), WriteError> {
        self.set_all(module, values)
            .map_err(|error| refusal(module, error))?;
        self.save().map_err(|notice| {
            if let Some(notice) = notice {
                self.notify(notice);
            }
            WriteError::Unsaved
        })
    }
}

/// A store refusal as a module sees it: its validator's error, or a section that cannot change.
fn refusal(module: ModuleId, error: StoreError) -> WriteError {
    match error {
        StoreError::Settings(invalid) => WriteError::Invalid(invalid),
        _ => WriteError::Refused(module),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bus::KernelBus;
    use kx_module_api::keys;
    use std::panic::{self, AssertUnwindSafe};

    const ID: ModuleId = ModuleId::new("a");
    const KEY: &str = "level";

    fn shared() -> Shared {
        Shared::new(Bus::new(Arc::new(KernelBus::new())))
    }

    /// `KEY` set to `value`, as a handle's one-value write.
    fn one(value: Value) -> Table {
        Table::from_iter([(KEY.to_owned(), value)])
    }

    #[test]
    fn before_load_a_write_is_refused_and_nothing_saves() {
        let share = shared();
        let refused = share.write(ID, one(Value::Integer(1)));
        assert_eq!(refused, Err(WriteError::Refused(ID)));
        assert_eq!(share.get(ID), None);
        assert_eq!(share.save(), Ok(()));
    }

    #[test]
    fn notices_are_kept_only_while_collecting() {
        let share = shared();
        let notice = |key| Notice {
            key,
            module: None,
            args: Vec::new(),
        };
        share.notify(notice(keys::SETTINGS_RESET));
        share.collect();
        share.notify(notice(keys::SETTINGS_UNSAVED));
        assert_eq!(share.collected(), [notice(keys::SETTINGS_UNSAVED)]);
        share.notify(notice(keys::MODULE_FAILED));
        assert_eq!(share.collected(), []);
    }

    #[test]
    fn a_poisoned_lock_is_recovered_with_what_it_held() {
        let share = shared();
        share.collect();
        let held = share.clone();
        let poison = panic::catch_unwind(AssertUnwindSafe(|| {
            let _store = lock(&held.store);
            let _told = lock(&held.told);
            panic::panic_any("a module thread panicked while it held both locks");
        }));
        assert!(poison.is_err() && share.store.is_poisoned() && share.told.is_poisoned());
        let refused = share.write(ID, one(Value::Integer(1)));
        assert_eq!(refused, Err(WriteError::Refused(ID)));
        let notice = || Notice {
            key: keys::SETTINGS_UNSAVED,
            module: None,
            args: Vec::new(),
        };
        share.notify(notice());
        assert_eq!(share.collected(), [notice()], "still collecting");
    }
}
