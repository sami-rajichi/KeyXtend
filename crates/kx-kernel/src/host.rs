//! The `Host` a module sees while it starts: services, bus, clock and settings.

use crate::grants::Grants;
use crate::registry::{Owner, Registry, Service};
use kx_module_api::{
    Bus, Capability, Clock, Host, Manifest, ModuleId, ServiceError, ServiceId, Subscription,
};
use std::any::Any;
use std::sync::Arc;

/// What a started module keeps: its handler guards and the services it provided.
#[must_use = "release a Held to remove the module's services"]
pub struct Held {
    module: ModuleId,
    guards: Vec<Subscription>,
    provided: Vec<ServiceId>,
}

impl Held {
    /// The services the module provided, in order; only tests read it, as release removes them by owner.
    #[cfg(test)]
    #[must_use]
    pub fn provided(&self) -> &[ServiceId] {
        &self.provided
    }

    /// Unsubscribes the module's handlers; its services stay in the registry.
    pub fn drop_handlers(&mut self) {
        self.guards.clear();
    }

    /// Unsubscribes any handlers left, then removes and drops the module's services.
    pub fn release(mut self, registry: &mut Registry) {
        self.drop_handlers();
        drop(registry.remove_owned(self.module));
    }
}

/// The kernel's `Host` for one module's start; it borrows the registry directly, not through a lock.
pub struct StartHost<'a> {
    manifest: &'static Manifest,
    registry: &'a mut Registry,
    grants: Grants,
    bus: Bus,
    clock: Arc<dyn Clock>,
    settings: toml::Table,
    held: Held,
}

impl<'a> StartHost<'a> {
    /// A host for `manifest`'s module; a module without settings gets an empty table.
    #[must_use]
    pub fn new(
        manifest: &'static Manifest,
        registry: &'a mut Registry,
        grants: Grants,
        bus: Bus,
        clock: Arc<dyn Clock>,
        settings: Option<toml::Table>,
    ) -> Self {
        let held = Held {
            module: manifest.id,
            guards: Vec::new(),
            provided: Vec::new(),
        };
        Self {
            manifest,
            registry,
            grants,
            bus,
            clock,
            settings: settings.unwrap_or_default(),
            held,
        }
    }

    /// Ends the start and hands over what the module set up.
    pub fn into_held(self) -> Held {
        self.held
    }
}

impl Host for StartHost<'_> {
    fn manifest(&self) -> &'static Manifest {
        self.manifest
    }

    fn service(
        &self,
        id: ServiceId,
        cap: Option<Capability>,
    ) -> Result<&(dyn Any + Send + Sync), ServiceError> {
        self.registry.lookup(id, cap, &self.grants)
    }

    /// A provider must itself hold the capability that gates its service.
    fn provide(
        &mut self,
        id: ServiceId,
        cap: Option<Capability>,
        service: Service,
    ) -> Result<(), ServiceError> {
        if let Some(cap) = cap.filter(|&cap| !self.grants.has(cap)) {
            return Err(ServiceError::NotGranted(id, cap));
        }
        let owner = Owner::Module(self.manifest.id);
        self.registry.provide(owner, id, cap, service)?;
        self.held.provided.push(id);
        Ok(())
    }

    fn bus(&self) -> Bus {
        self.bus.clone()
    }

    fn clock(&self) -> Arc<dyn Clock> {
        Arc::clone(&self.clock)
    }

    fn settings(&self) -> &toml::Table {
        &self.settings
    }

    fn hold(&mut self, guard: Subscription) {
        self.held.guards.push(guard);
    }
}

#[cfg(test)]
mod tests;
