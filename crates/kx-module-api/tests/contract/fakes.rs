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

/// A stored service and the capability that gates it.
pub type Stored = (Option<Capability>, Box<dyn Any + Send + Sync>);

/// A host with a service map, a grant list and a fake bus; it counts service lookups.
pub struct FakeHost {
    pub manifest: &'static Manifest,
    pub granted: Vec<Capability>,
    pub services: HashMap<ServiceId, Stored>,
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
        match self.services.entry(id) {
            Entry::Occupied(_) => Err(ServiceError::AlreadyProvided(id)),
            Entry::Vacant(slot) => {
                slot.insert((cap, service));
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

/// A bus core that calls every handler whatever the event type, with no lock held while they run.
/// That way the typed wrappers' downcast guard is exercised.
#[derive(Default)]
pub struct FakeCore(Mutex<Handlers>);

type SharedNotify = Arc<dyn Fn(&(dyn Any + Send + Sync)) + Send + Sync>;
type SharedIntercept = Arc<dyn Fn(&mut (dyn Any + Send + Sync)) -> Flow + Send + Sync>;

#[derive(Default)]
struct Handlers {
    last: u64,
    notify: Vec<(SubscriptionId, SharedNotify)>,
    intercept: Vec<(SubscriptionId, i32, SharedIntercept)>,
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
        let chain: Vec<_> = self
            .lock()
            .notify
            .iter()
            .map(|(_, h)| Arc::clone(h))
            .collect();
        for handler in chain {
            handler(event);
        }
    }

    fn intercept(&self, _: TypeId, _: &'static str, event: &mut (dyn Any + Send + Sync)) -> Flow {
        let mut chain: Vec<_> = {
            let handlers = self.lock();
            handlers
                .intercept
                .iter()
                .map(|(_, o, h)| (*o, Arc::clone(h)))
                .collect()
        };
        chain.sort_by_key(|(order, _)| *order);
        for (_, handler) in chain {
            if handler(event) == Flow::Stop {
                return Flow::Stop;
            }
        }
        Flow::Continue
    }

    fn subscribe(&self, _: TypeId, _: &'static str, handler: NotifyFn) -> SubscriptionId {
        let mut handlers = self.lock();
        let id = handlers.next_id();
        handlers.notify.push((id, Arc::from(handler)));
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
        handlers.intercept.push((id, order, Arc::from(handler)));
        id
    }

    fn unsubscribe(&self, id: SubscriptionId) {
        let mut handlers = self.lock();
        handlers.notify.retain(|(i, _)| *i != id);
        handlers.intercept.retain(|(i, _, _)| *i != id);
    }
}
