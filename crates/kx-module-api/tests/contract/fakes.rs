//! Small fakes of the kernel side: a bus core and a host.

use kx_module_api::{
    Bus, BusCore, Capability, Clock, Flow, Host, InterceptFn, Manifest, Mono, NotifyFn,
    ServiceError, ServiceId, Subscription, SubscriptionId,
};
use std::any::{Any, TypeId};
use std::cell::Cell;
use std::collections::HashMap;
use std::collections::hash_map::Entry;
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};

/// The time the fake clock always reads.
pub const NOW: Mono = Mono::from_us(42);

/// A clock stuck at `NOW`.
struct Fixed;

impl Clock for Fixed {
    fn now(&self) -> Mono {
        NOW
    }
}

/// A host with a service map, a grant list and a fake bus; it counts service lookups.
pub struct FakeHost {
    pub manifest: &'static Manifest,
    pub granted: Vec<Capability>,
    pub services: HashMap<ServiceId, Box<dyn Any + Send + Sync>>,
    pub lookups: Cell<usize>,
    pub settings: toml::Table,
    pub held: Vec<Subscription>,
    core: Arc<FakeCore>,
}

impl FakeHost {
    pub fn new(manifest: &'static Manifest) -> Self {
        Self {
            manifest,
            granted: Vec::new(),
            services: HashMap::new(),
            lookups: Cell::new(0),
            settings: toml::Table::new(),
            held: Vec::new(),
            core: Arc::default(),
        }
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
        if let Some(cap) = cap.filter(|c| !self.granted.contains(c)) {
            return Err(ServiceError::NotGranted(id, cap));
        }
        self.services
            .get(&id)
            .map(|s| &**s)
            .ok_or(ServiceError::Missing(id))
    }

    fn provide(
        &mut self,
        id: ServiceId,
        service: Box<dyn Any + Send + Sync>,
    ) -> Result<(), ServiceError> {
        match self.services.entry(id) {
            Entry::Occupied(_) => Err(ServiceError::AlreadyProvided(id)),
            Entry::Vacant(slot) => {
                slot.insert(service);
                Ok(())
            }
        }
    }

    fn bus(&self) -> Bus {
        let core: Arc<FakeCore> = Arc::clone(&self.core);
        Bus::new(core)
    }

    fn clock(&self) -> Arc<dyn Clock> {
        Arc::new(Fixed)
    }

    fn settings(&self) -> &toml::Table {
        &self.settings
    }

    fn hold(&mut self, guard: Subscription) {
        self.held.push(guard);
    }
}

/// A bus core that calls every handler whatever the event type.
/// That way the typed wrappers' downcast guard is exercised.
#[derive(Default)]
pub struct FakeCore(Mutex<Handlers>);

#[derive(Default)]
struct Handlers {
    last: u64,
    notify: Vec<(SubscriptionId, NotifyFn)>,
    intercept: Vec<(SubscriptionId, i32, InterceptFn)>,
}

impl Handlers {
    fn next_id(&mut self) -> SubscriptionId {
        self.last += 1;
        SubscriptionId::new(self.last)
    }
}

impl FakeCore {
    /// How many handlers are registered.
    pub fn count(&self) -> usize {
        let handlers = self.lock();
        handlers.notify.len() + handlers.intercept.len()
    }

    fn lock(&self) -> MutexGuard<'_, Handlers> {
        self.0.lock().unwrap_or_else(PoisonError::into_inner)
    }
}

impl BusCore for FakeCore {
    fn publish(&self, _: TypeId, _: &'static str, event: &(dyn Any + Send + Sync)) {
        for (_, handler) in &self.lock().notify {
            handler(event);
        }
    }

    fn intercept(&self, _: TypeId, _: &'static str, event: &mut (dyn Any + Send + Sync)) -> Flow {
        let handlers = self.lock();
        let mut chain: Vec<_> = handlers.intercept.iter().collect();
        chain.sort_by_key(|(_, order, _)| *order);
        for (_, _, handler) in chain {
            if handler(event) == Flow::Stop {
                return Flow::Stop;
            }
        }
        Flow::Continue
    }

    fn subscribe(&self, _: TypeId, _: &'static str, handler: NotifyFn) -> SubscriptionId {
        let mut handlers = self.lock();
        let id = handlers.next_id();
        handlers.notify.push((id, handler));
        id
    }

    fn add_interceptor(
        &self,
        _: TypeId,
        _: &'static str,
        order: i32,
        handler: InterceptFn,
    ) -> SubscriptionId {
        let mut handlers = self.lock();
        let id = handlers.next_id();
        handlers.intercept.push((id, order, handler));
        id
    }

    fn unsubscribe(&self, id: SubscriptionId) {
        let mut handlers = self.lock();
        handlers.notify.retain(|(i, _)| *i != id);
        handlers.intercept.retain(|(i, _, _)| *i != id);
    }
}
