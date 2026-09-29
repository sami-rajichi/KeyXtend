//! The settings store the kernel shares with every module's settings handle, behind one lock.
//! No lock is held while a module runs or the bus publishes, so a handler may write settings.

use kx_module_api::{Bus, ModuleId, Notice, SettingsSink, WriteError};
use kx_settings::{Store, StoreError};
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};
use toml::{Table, Value};

/// The store, once loaded, and the bus that carries an unsaved notice from a module's write.
#[derive(Clone)]
pub struct Shared {
    store: Arc<Mutex<Option<Store>>>,
    bus: Bus,
}

impl Shared {
    /// An empty share; `put` fills it when the kernel loads.
    #[must_use]
    pub fn new(bus: Bus) -> Self {
        Self {
            store: Arc::new(Mutex::new(None)),
            bus,
        }
    }

    /// Holds `store` from now on.
    pub fn put(&self, store: Store) {
        *self.lock() = Some(store);
    }

    /// A copy of `module`'s section, ready for `parse_as`; `None` before load or without settings.
    #[must_use]
    pub fn get(&self, module: ModuleId) -> Option<Table> {
        self.lock().as_ref()?.get(module).cloned()
    }

    /// Sets one value of `module`'s section; true when the stored value changed.
    ///
    /// # Errors
    /// `Unknown` before load, or the store's refusal; nothing changes then.
    pub fn set(&self, module: ModuleId, key: &str, value: Value) -> Result<bool, StoreError> {
        let mut guard = self.lock();
        let store = guard.as_mut().ok_or(StoreError::Unknown(module))?;
        store.set(module, key, value)
    }

    /// Saves when the file is behind; a failure is logged and gives back the notice due, if any.
    ///
    /// # Errors
    /// The `settings-unsaved` notice, or `None` when the user was already told.
    pub fn save(&self) -> Result<(), Option<Notice>> {
        let mut guard = self.lock();
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

    /// The store; a panic on another thread that held it does not stop the kernel.
    fn lock(&self) -> MutexGuard<'_, Option<Store>> {
        self.store.lock().unwrap_or_else(PoisonError::into_inner)
    }
}

impl SettingsSink for Shared {
    /// Sets and saves at once; a failed save publishes its notice after the lock is released.
    fn write(&self, module: ModuleId, key: &str, value: Value) -> Result<(), WriteError> {
        self.set(module, key, value)
            .map_err(|error| refusal(module, error))?;
        self.save().map_err(|notice| {
            if let Some(notice) = notice {
                self.bus.publish(&notice);
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
    use std::panic::{self, AssertUnwindSafe};

    const ID: ModuleId = ModuleId::new("a");
    const KEY: &str = "level";

    fn shared() -> Shared {
        Shared::new(Bus::new(Arc::new(KernelBus::new())))
    }

    #[test]
    fn before_load_a_write_is_refused_and_nothing_saves() {
        let share = shared();
        let refused = share.write(ID, KEY, Value::Integer(1));
        assert_eq!(refused, Err(WriteError::Refused(ID)));
        assert_eq!(share.get(ID), None);
        assert_eq!(share.save(), Ok(()));
    }

    #[test]
    fn a_poisoned_lock_is_recovered_not_passed_on() {
        let share = shared();
        let held = share.clone();
        let poison = panic::catch_unwind(AssertUnwindSafe(|| {
            let _guard = held.lock();
            panic::panic_any("a module thread panicked while it held the store");
        }));
        assert!(poison.is_err() && share.store.is_poisoned());
        let refused = share.write(ID, KEY, Value::Integer(1));
        assert_eq!(refused, Err(WriteError::Refused(ID)));
    }
}
