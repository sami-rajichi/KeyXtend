//! The module's start through a small host that gates services as the kernel does.

#![allow(
    clippy::unwrap_used,
    reason = "these fixtures run only in tests, where a failed setup should fail the test"
)]

use crate::common::{Heard, bus, ids, source};
use kx_mod_layouts::{LayoutsModule, MANIFEST, SPEC};
use kx_module_api::Capability::InjectInput;
use kx_module_api::ports::{LayoutSource, LayoutSourceKey, Layouts, LayoutsKey};
use kx_module_api::{
    Bus, Capability, Clock, Host, Manifest, Module, ModuleCx, ModuleError, ModuleId, Mono,
    ServiceError, ServiceId, ServiceKey, SettingsError, SettingsSink, Subscription, WriteError,
};
use kx_platform_fake::FakeClock;
use std::any::Any;
use std::collections::HashMap;
use std::sync::Arc;

/// A stored service and the capability that gates it.
type Stored = (Option<Capability>, Box<dyn Any + Send + Sync>);

/// A sink that takes no writes; the module never writes its own settings.
struct NoWrites;

impl SettingsSink for NoWrites {
    fn write(&self, module: ModuleId, _: toml::Table) -> Result<(), WriteError> {
        Err(WriteError::Refused(module))
    }
}

/// The kernel's side of one start: grants, stored services, the section and a bus.
struct FakeHost {
    granted: Vec<Capability>,
    services: HashMap<ServiceId, Stored>,
    settings: toml::Table,
    heard: Arc<Heard>,
    held: Vec<Subscription>,
}

impl FakeHost {
    /// A host granting `granted`, holding the golden source when `with_source`, on the default settings.
    fn new(granted: &[Capability], with_source: bool) -> Self {
        let mut services = HashMap::new();
        if with_source {
            let source: Arc<dyn LayoutSource> = source();
            let stored: Stored = (LayoutSourceKey::CAPABILITY, Box::new(source));
            services.insert(LayoutSourceKey::ID, stored);
        }
        Self {
            granted: granted.to_vec(),
            services,
            settings: SPEC.defaults.parse().unwrap(),
            heard: Arc::default(),
            held: Vec::new(),
        }
    }

    /// Starts the module on this host.
    fn start(&mut self) -> Result<(), ModuleError> {
        LayoutsModule.start(&mut ModuleCx::new(self))
    }

    /// The layouts service the module provided.
    fn layouts(&self) -> Arc<dyn Layouts> {
        let (_, stored) = &self.services[&LayoutsKey::ID];
        stored.downcast_ref::<Arc<dyn Layouts>>().unwrap().clone()
    }
}

impl Host for FakeHost {
    fn manifest(&self) -> &'static Manifest {
        &MANIFEST
    }

    fn service(
        &self,
        id: ServiceId,
        cap: Option<Capability>,
    ) -> Result<&(dyn Any + Send + Sync), ServiceError> {
        let (stored, service) = self.services.get(&id).ok_or(ServiceError::Missing(id))?;
        if *stored != cap {
            return Err(ServiceError::CapabilityMismatch(id));
        }
        if let Some(cap) = stored.filter(|c| !self.granted.contains(c)) {
            return Err(ServiceError::NotGranted(id, cap));
        }
        Ok(&**service)
    }

    fn provide(
        &mut self,
        id: ServiceId,
        cap: Option<Capability>,
        service: Box<dyn Any + Send + Sync>,
    ) -> Result<(), ServiceError> {
        if let Some(cap) = cap.filter(|c| !self.granted.contains(c)) {
            return Err(ServiceError::NotGranted(id, cap));
        }
        if self.services.contains_key(&id) {
            return Err(ServiceError::AlreadyProvided(id));
        }
        self.services.insert(id, (cap, service));
        Ok(())
    }

    fn bus(&self) -> Bus {
        bus(&self.heard).restricted(&self.granted)
    }

    fn clock(&self) -> Arc<dyn Clock> {
        Arc::new(FakeClock::new(Mono::default()))
    }

    fn settings(&self) -> &toml::Table {
        &self.settings
    }

    fn settings_sink(&self) -> Arc<dyn SettingsSink> {
        Arc::new(NoWrites)
    }

    fn hold(&mut self, guard: Subscription) {
        self.held.push(guard);
    }
}

#[test]
fn it_provides_the_layouts_of_the_app_in_front() {
    let [en, fr, _] = ids();
    let mut host = FakeHost::new(&[InjectInput], true);
    assert_eq!(host.start(), Ok(()));
    let layouts = host.layouts();
    assert_eq!(layouts.current().label.id, en);
    let shorts: Vec<String> = layouts.list().into_iter().map(|l| l.short).collect();
    assert_eq!(shorts, ["EN", "FR", "ع"]);
    assert_eq!(layouts.pick(fr), Ok(()));
    assert!(layouts.refresh());
    assert_eq!(host.heard.ids(), [fr], "the change goes out on the bus");
}

#[test]
fn the_short_setting_names_the_language_key() {
    let mut host = FakeHost::new(&[InjectInput], true);
    host.settings = "short = { fr = \"Fr\" }".parse().unwrap();
    assert_eq!(host.start(), Ok(()));
    let shorts: Vec<String> = host.layouts().list().into_iter().map(|l| l.short).collect();
    assert_eq!(
        shorts,
        ["EN", "Fr", "AR"],
        "a user's map replaces the default one"
    );
}

#[test]
fn without_inject_input_it_cannot_read_the_source() {
    let mut host = FakeHost::new(&[], true);
    let refused = ServiceError::NotGranted(LayoutSourceKey::ID, InjectInput);
    assert_eq!(host.start(), Err(ModuleError::Service(refused)));
    assert!(!host.services.contains_key(&LayoutsKey::ID));
}

#[test]
fn without_a_source_it_fails_to_start() {
    let mut host = FakeHost::new(&[InjectInput], false);
    let missing = ServiceError::Missing(LayoutSourceKey::ID);
    assert_eq!(host.start(), Err(ModuleError::Service(missing)));
}

#[test]
fn bad_settings_fail_the_start_instead_of_a_panic() {
    let mut host = FakeHost::new(&[InjectInput], true);
    host.settings = "short = { AR = \"x\" }".parse().unwrap();
    let err = host.start().unwrap_err();
    assert!(
        matches!(err, ModuleError::Settings(SettingsError::Invalid(_))),
        "{err}"
    );
    assert!(!host.services.contains_key(&LayoutsKey::ID));
}
