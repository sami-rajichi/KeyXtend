//! Runtime controls: Try again, switching a module off and on, and changing one setting.

use super::{DISABLED_KEY, KERNEL, Kernel, KernelError};
use crate::lifecycle::Step;
use crate::order;
use kx_module_api::ModuleState::{Active, Failed, Pending, Starting, Stopped, Stopping};
use kx_module_api::{ModuleId, ModuleState, SettingsChanged};
use toml::Value;

impl Kernel {
    /// Try again: starts a failed module, then, once it runs, every dependent not switched off.
    ///
    /// # Errors
    /// `Unknown`, `NotFailed`, or `NotStartable` when it is off or its plan or spec is broken.
    pub fn retry(&mut self, id: ModuleId) -> Result<(), KernelError> {
        let at = self.find(id).ok_or(KernelError::Unknown(id))?;
        if self.slots[at].state != Failed {
            return Err(KernelError::NotFailed(id));
        }
        if self.slots[at].blocked || self.is_disabled(id) {
            return Err(KernelError::NotStartable(id));
        }
        if self.start(at) {
            self.start_dependents(id, &[Failed, Pending, Stopped]);
        }
        Ok(())
    }

    /// Switches module `id` off (its dependents stop first) or on (its stopped dependents follow).
    /// The choice is saved in `[kernel] disabled`.
    ///
    /// # Errors
    /// `Reserved` for the kernel, `Unknown`, `NotBooted` or the store's refusal; nothing changes.
    pub fn set_enabled(&mut self, id: ModuleId, on: bool) -> Result<(), KernelError> {
        if id == KERNEL {
            return Err(KernelError::Reserved(id));
        }
        let at = self.find(id).ok_or(KernelError::Unknown(id))?;
        let mut list = self.disabled();
        let listed = list.iter().any(|d| d == id.as_str());
        if on {
            list.retain(|d| d != id.as_str());
        } else if !listed {
            list.push(id.as_str().to_owned());
        }
        let value = Value::Array(list.into_iter().map(Value::String).collect());
        let store = self.store.as_mut().ok_or(KernelError::NotBooted)?;
        store.set(KERNEL, DISABLED_KEY, value)?;
        self.save();
        if on {
            self.switch_on(at);
        } else {
            self.switch_off(at);
        }
        Ok(())
    }

    /// Sets one value of `module`'s settings, saves it and announces it; a running module restarts.
    ///
    /// # Errors
    /// `SwitchKey` for the kernel's disabled list, `NotBooted` or a store refusal; nothing changes.
    pub fn set_setting(
        &mut self,
        module: ModuleId,
        key: &str,
        value: Value,
    ) -> Result<(), KernelError> {
        if module == KERNEL && key == DISABLED_KEY {
            return Err(KernelError::SwitchKey);
        }
        let store = self.store.as_mut().ok_or(KernelError::NotBooted)?;
        store.set(module, key, value)?;
        self.save();
        self.bus.publish(&SettingsChanged { module });
        if let Some(at) = self.active(module) {
            self.restart(at);
        }
        Ok(())
    }

    /// Stops module `at`'s active dependents, then the module, which ends `Stopped`.
    fn switch_off(&mut self, at: usize) {
        self.stop_dependents(self.slots[at].manifest.id);
        match self.slots[at].state {
            Active => self.stop(at),
            Pending | Failed => {
                self.step(at, Step::Skip);
            }
            Starting | Stopping | Stopped => {}
        }
    }

    /// Starts module `at` unless it runs, then its stopped dependents that are switched on.
    fn switch_on(&mut self, at: usize) {
        if self.slots[at].state == Active || self.start(at) {
            self.start_dependents(self.slots[at].manifest.id, &[Stopped]);
        }
    }

    /// Stops module `at` and its active dependents, then starts them so they read the new settings.
    /// A dependent whose provider failed to come back fails too, so Try again restarts it.
    fn restart(&mut self, at: usize) {
        let stopped = self.stop_dependents(self.slots[at].manifest.id);
        self.stop(at);
        self.start(at);
        for id in stopped.into_iter().rev() {
            if let Some(dependent) = self.find(id) {
                self.start(dependent);
            }
        }
    }

    /// Stops the active modules that need `id`, dependents first, and returns them in that order.
    fn stop_dependents(&mut self, id: ModuleId) -> Vec<ModuleId> {
        let mut stopped = Vec::new();
        for dependent in order::dependents(&self.plan, id) {
            if let Some(at) = self.active(dependent) {
                self.stop(at);
                stopped.push(dependent);
            }
        }
        stopped
    }

    /// Starts, in start order, the dependents of `id` that are in one of `states`, switched on,
    /// startable and no longer waiting for a provider.
    fn start_dependents(&mut self, id: ModuleId, states: &[ModuleState]) {
        let disabled = self.disabled();
        for dependent in order::dependents(&self.plan, id).into_iter().rev() {
            let Some(at) = self.find(dependent) else {
                continue;
            };
            let slot = &self.slots[at];
            let off = disabled.iter().any(|d| d == dependent.as_str());
            let wanted = states.contains(&slot.state) && !slot.blocked && !off;
            if wanted && self.waiting_on(at).is_none() {
                self.start(at);
            }
        }
    }
}
