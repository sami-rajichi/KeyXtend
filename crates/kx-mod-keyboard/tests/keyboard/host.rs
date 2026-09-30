//! A small host for one module start, with a settings sink that checks each write as the kernel does.

#![allow(
    clippy::unwrap_used,
    reason = "these fixtures run only in tests, where a failed setup should fail the test"
)]

use kx_mod_keyboard::{KeyboardSettings, MANIFEST, SPEC};
use kx_module_api::{
    Bus, BusCore, Capability, Clock, Flow, Host, InterceptFn, Manifest, ModuleCx, ModuleId, Mono,
    NotifyFn, ServiceError, ServiceId, SettingsHandle, SettingsSink, Subscription, SubscriptionId,
    WriteError,
};
use kx_platform_fake::FakeClock;
use std::any::{Any, TypeId};
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};

fn lock<T>(m: &Mutex<T>) -> MutexGuard<'_, T> {
    m.lock().unwrap_or_else(PoisonError::into_inner)
}

/// One accepted write: the module and its values.
pub type Written = (ModuleId, toml::Table);

/// The `[keyboard]` section as the kernel keeps it: each write is checked against the whole section.
pub struct Sink {
    section: Mutex<toml::Table>,
    writes: Mutex<Vec<Written>>,
    refusal: Mutex<Option<WriteError>>,
    refused_key: Mutex<Option<(String, WriteError)>>,
}

impl Sink {
    /// A section holding `section`.
    pub fn new(section: toml::Table) -> Self {
        Self {
            section: Mutex::new(section),
            writes: Mutex::default(),
            refusal: Mutex::default(),
            refused_key: Mutex::default(),
        }
    }

    /// Every accepted write, oldest first.
    pub fn writes(&self) -> Vec<Written> {
        lock(&self.writes).clone()
    }

    /// The section read as settings.
    pub fn settings(&self) -> KeyboardSettings {
        kx_module_api::parse_as(&lock(&self.section)).unwrap()
    }

    /// Refuses every later write with `error`, or takes them again with `None`.
    pub fn refuse(&self, error: Option<WriteError>) {
        *lock(&self.refusal) = error;
    }

    /// Refuses every later write that holds `key`, with `error`.
    pub fn refuse_key(&self, key: &str, error: WriteError) {
        *lock(&self.refused_key) = Some((key.to_owned(), error));
    }
}

impl SettingsSink for Sink {
    fn write(&self, module: ModuleId, values: toml::Table) -> Result<(), WriteError> {
        if let Some(error) = lock(&self.refusal).clone() {
            return Err(error);
        }
        let refused = lock(&self.refused_key).clone();
        if let Some((_, error)) = refused.filter(|(k, _)| values.contains_key(k)) {
            return Err(error);
        }
        let mut next = lock(&self.section).clone();
        next.extend(values.clone());
        (SPEC.validate)(&next)?;
        *lock(&self.section) = next;
        lock(&self.writes).push((module, values));
        Ok(())
    }
}

/// A bus core that carries nothing; the keyboard sends no events yet.
struct Quiet;

impl BusCore for Quiet {
    fn publish(&self, _: TypeId, _: &'static str, _: &(dyn Any + Send + Sync)) {}

    fn intercept(&self, _: TypeId, _: &'static str, _: &mut (dyn Any + Send + Sync)) -> Flow {
        Flow::Continue
    }

    fn subscribe(&self, _: TypeId, _: &'static str, _: NotifyFn) -> SubscriptionId {
        SubscriptionId::new(0)
    }

    fn add_interceptor(
        &self,
        _: TypeId,
        _: &'static str,
        _: i32,
        _: InterceptFn,
    ) -> SubscriptionId {
        SubscriptionId::new(0)
    }

    fn unsubscribe(&self, _: SubscriptionId) {}
}

/// The kernel's side of one start: the section, its sink and nothing else.
pub struct FakeHost {
    settings: toml::Table,
    pub sink: Arc<Sink>,
    held: Vec<Subscription>,
}

impl FakeHost {
    /// A host on the defaults with `key` set to `value`, when given.
    pub fn with(change: Option<(&str, toml::Value)>) -> Self {
        let mut settings: toml::Table = SPEC.defaults.parse().unwrap();
        if let Some((key, value)) = change {
            settings.insert(key.to_owned(), value);
        }
        Self {
            sink: Arc::new(Sink::new(settings.clone())),
            settings,
            held: Vec::new(),
        }
    }

    /// A host on the defaults.
    pub fn new() -> Self {
        Self::with(None)
    }

    /// The module's own settings handle, as `ModuleCx` hands it out.
    pub fn handle(&mut self) -> SettingsHandle {
        ModuleCx::new(self).settings_handle()
    }

    /// The settings the module starts with.
    pub fn settings(&self) -> KeyboardSettings {
        kx_module_api::parse_as(&self.settings).unwrap()
    }
}

impl Host for FakeHost {
    fn manifest(&self) -> &'static Manifest {
        &MANIFEST
    }

    fn service(
        &self,
        id: ServiceId,
        _: Option<Capability>,
    ) -> Result<&(dyn Any + Send + Sync), ServiceError> {
        Err(ServiceError::Missing(id))
    }

    fn provide(
        &mut self,
        id: ServiceId,
        _: Option<Capability>,
        _: Box<dyn Any + Send + Sync>,
    ) -> Result<(), ServiceError> {
        Err(ServiceError::NotDeclared(id))
    }

    fn bus(&self) -> Bus {
        Bus::new(Arc::new(Quiet)).restricted(&[])
    }

    fn clock(&self) -> Arc<dyn Clock> {
        Arc::new(FakeClock::new(Mono::default()))
    }

    fn settings(&self) -> &toml::Table {
        &self.settings
    }

    fn settings_sink(&self) -> Arc<dyn SettingsSink> {
        let sink: Arc<Sink> = self.sink.clone();
        sink
    }

    fn hold(&mut self, guard: Subscription) {
        self.held.push(guard);
    }
}
