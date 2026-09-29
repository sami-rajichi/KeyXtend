//! A module's view of the kernel while it starts: typed services, events, clock and settings.

use crate::{
    Bus, Capability, Clock, Event, Flow, Manifest, ServiceError, ServiceId, ServiceKey, Settings,
    SettingsError, Subscription, parse_as,
};
use std::any::Any;
use std::sync::Arc;

/// The object-safe kernel side of one module's start; `ModuleCx` is its typed front.
pub trait Host {
    /// The starting module's manifest.
    fn manifest(&self) -> &'static Manifest;

    /// The stored service `id`, once `cap` is granted and a provider exists.
    ///
    /// # Errors
    /// `ServiceError::NotGranted` or `ServiceError::Missing`.
    fn service(
        &self,
        id: ServiceId,
        cap: Option<Capability>,
    ) -> Result<&(dyn Any + Send + Sync), ServiceError>;

    /// Stores `service` under `id` for other modules.
    ///
    /// # Errors
    /// `ServiceError::AlreadyProvided` when another module provides `id`.
    fn provide(
        &mut self,
        id: ServiceId,
        service: Box<dyn Any + Send + Sync>,
    ) -> Result<(), ServiceError>;

    /// A handle to the kernel's bus.
    fn bus(&self) -> Bus;

    /// The kernel's clock.
    fn clock(&self) -> Arc<dyn Clock>;

    /// The module's settings section, already migrated and repaired.
    fn settings(&self) -> &toml::Table;

    /// Keeps `guard` alive until the module stops.
    fn hold(&mut self, guard: Subscription);
}

/// The typed front a module uses in `start`; it checks the manifest before asking the host.
pub struct ModuleCx<'a> {
    host: &'a mut dyn Host,
}

impl<'a> ModuleCx<'a> {
    /// Wraps the kernel's host for one module's start.
    #[must_use]
    pub fn new(host: &'a mut dyn Host) -> Self {
        Self { host }
    }

    /// The starting module's manifest.
    #[must_use]
    pub fn manifest(&self) -> &'static Manifest {
        self.host.manifest()
    }

    /// The service keyed by `K`, shared with its provider.
    ///
    /// # Errors
    /// `NotRequired` when the manifest lacks it, `WrongType` on a type mismatch,
    /// or the host's refusal.
    pub fn service<K: ServiceKey>(&self) -> Result<Arc<K::Api>, ServiceError> {
        if !self.manifest().requires(K::ID) {
            return Err(ServiceError::NotRequired(K::ID));
        }
        self.host
            .service(K::ID, K::CAPABILITY)?
            .downcast_ref::<Arc<K::Api>>()
            .cloned()
            .ok_or(ServiceError::WrongType(K::ID))
    }

    /// Offers `service` to other modules under `K`.
    ///
    /// # Errors
    /// `NotDeclared` when the manifest lacks it, or the host's refusal.
    pub fn provide<K: ServiceKey>(&mut self, service: Arc<K::Api>) -> Result<(), ServiceError> {
        if !self.manifest().provides(K::ID) {
            return Err(ServiceError::NotDeclared(K::ID));
        }
        self.host.provide(K::ID, Box::new(service))
    }

    /// Calls `handler` for every published `E` until the module stops.
    pub fn subscribe<E: Event>(&mut self, handler: impl Fn(&E) + Send + Sync + 'static) {
        let guard = self.host.bus().subscribe(handler);
        self.host.hold(guard);
    }

    /// Adds `handler` to `E`'s intercept chain at `order` until the module stops.
    pub fn intercept<E: Event>(
        &mut self,
        order: i32,
        handler: impl Fn(&mut E) -> Flow + Send + Sync + 'static,
    ) {
        let guard = self.host.bus().add_interceptor(order, handler);
        self.host.hold(guard);
    }

    /// A handle to the bus, for publishing or for worker threads.
    #[must_use]
    pub fn bus(&self) -> Bus {
        self.host.bus()
    }

    /// The kernel's clock.
    #[must_use]
    pub fn clock(&self) -> Arc<dyn Clock> {
        self.host.clock()
    }

    /// The module's settings, read as `T` and checked.
    ///
    /// # Errors
    /// `SettingsError::Parse` or the error from `T::check`.
    pub fn settings<T: Settings>(&self) -> Result<T, SettingsError> {
        parse_as(self.host.settings())
    }
}
