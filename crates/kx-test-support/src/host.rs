//! A fake kernel host for module tests: gated services, a bus, a clock and a settings sink that checks each write as
//! the kernel does. One host serves the API's contract tests and every module's tests.

mod bus;
mod sink;

pub use bus::FakeCore;
pub use sink::{FakeSink, Written};

use kx_module_api::{
    Bus, BusCore, Capability, Clock, Host, Manifest, ModuleCx, Mono, ServiceError, ServiceId,
    SettingsHandle, SettingsSink, Subscription,
};
use std::any::Any;
use std::cell::Cell;
use std::collections::HashMap;
use std::sync::Arc;
use toml::{Table, Value};

/// The time the fake clock always reads.
pub const NOW: Mono = Mono::from_us(42);

/// A stored service and the capability that gates it.
pub type Stored = (Option<Capability>, Box<dyn Any + Send + Sync>);

/// A clock stuck at `NOW`.
struct Fixed;

impl Clock for Fixed {
    fn now(&self) -> Mono {
        NOW
    }
}

/// The kernel's side of one module start; the fields are open so a test can set up or inspect each part.
pub struct FakeHost {
    /// The module's manifest.
    pub manifest: &'static Manifest,
    /// The capabilities the policy allows the module; as in the kernel, only those its manifest declares count.
    pub granted: Vec<Capability>,
    /// The services by id, with the capability that gates each.
    pub services: HashMap<ServiceId, Stored>,
    /// How many service lookups reached the host.
    pub lookups: Cell<usize>,
    /// The settings section the module starts with.
    pub settings: Table,
    /// The subscriptions the host holds for the module.
    pub held: Vec<Subscription>,
    /// The sink behind the module's settings handle.
    pub sink: Arc<FakeSink>,
    /// The bus core; a `FakeCore` unless a test sets its own.
    pub core: Arc<dyn BusCore>,
    /// The clock the module reads; stuck at `NOW` unless a test sets its own.
    pub clock: Arc<dyn Clock>,
}

impl FakeHost {
    /// A host for `manifest`, on its settings defaults, with a sink that checks writes by its spec.
    #[must_use]
    pub fn new(manifest: &'static Manifest) -> Self {
        let spec = manifest.settings;
        let settings: Table = spec
            .and_then(|s| s.defaults.parse().ok())
            .unwrap_or_default();
        Self {
            manifest,
            granted: Vec::new(),
            services: HashMap::new(),
            lookups: Cell::new(0),
            sink: Arc::new(FakeSink::new(spec, settings.clone())),
            settings,
            held: Vec::new(),
            core: Arc::new(FakeCore::default()),
            clock: Arc::new(Fixed),
        }
    }

    /// Sets `key` to `value` in the section the module starts with and in the sink's copy.
    pub fn put(&mut self, key: &str, value: Value) {
        self.settings.insert(key.to_owned(), value.clone());
        self.sink.put(Table::from_iter([(key.to_owned(), value)]));
    }

    /// The module's own settings handle, as `ModuleCx` hands it out.
    pub fn handle(&mut self) -> SettingsHandle {
        ModuleCx::new(self).settings_handle()
    }

    /// What the module may use: the capabilities both granted and declared, as the kernel's grants.
    fn holds(&self) -> Vec<Capability> {
        let declared = self.manifest.capabilities;
        let held = self.granted.iter().filter(|c| declared.contains(c));
        held.copied().collect()
    }
}

impl Host for FakeHost {
    fn manifest(&self) -> &'static Manifest {
        self.manifest
    }

    fn service(
        &self,
        id: ServiceId,
        cap: Option<Capability>,
    ) -> Result<&(dyn Any + Send + Sync), ServiceError> {
        self.lookups.set(self.lookups.get() + 1);
        let (stored, service) = self.services.get(&id).ok_or(ServiceError::Missing(id))?;
        if *stored != cap {
            return Err(ServiceError::CapabilityMismatch(id));
        }
        if let Some(cap) = stored.filter(|c| !self.holds().contains(c)) {
            return Err(ServiceError::NotGranted(id, cap));
        }
        Ok(&**service)
    }

    /// As the kernel: a provider must itself hold the capability that gates its service.
    fn provide(
        &mut self,
        id: ServiceId,
        cap: Option<Capability>,
        service: Box<dyn Any + Send + Sync>,
    ) -> Result<(), ServiceError> {
        if let Some(cap) = cap.filter(|c| !self.holds().contains(c)) {
            return Err(ServiceError::NotGranted(id, cap));
        }
        if self.services.contains_key(&id) {
            return Err(ServiceError::AlreadyProvided(id));
        }
        self.services.insert(id, (cap, service));
        Ok(())
    }

    fn bus(&self) -> Bus {
        Bus::new(Arc::clone(&self.core)).restricted(&self.holds())
    }

    fn clock(&self) -> Arc<dyn Clock> {
        Arc::clone(&self.clock)
    }

    fn settings(&self) -> &Table {
        &self.settings
    }

    fn settings_sink(&self) -> Arc<dyn SettingsSink> {
        let sink: Arc<FakeSink> = Arc::clone(&self.sink);
        sink
    }

    fn hold(&mut self, guard: Subscription) {
        self.held.push(guard);
    }
}

#[cfg(test)]
mod tests;
