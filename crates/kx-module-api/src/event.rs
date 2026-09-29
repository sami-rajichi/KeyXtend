//! Typed events over the kernel's bus: broadcast notifications and ordered intercept chains.

use std::any::{Any, TypeId};
use std::sync::{Arc, Weak};

/// A value sent over the bus; `NAME` labels it in logs.
pub trait Event: Send + Sync + 'static {
    /// The event's stable kebab-case name.
    const NAME: &'static str;
}

/// What an interceptor tells the chain: go on, or stop here.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Flow {
    /// Let the next interceptor see the event.
    Continue,
    /// End the chain; later interceptors do not run.
    Stop,
}

/// A boxed broadcast handler, as the bus core stores it.
pub type NotifyFn = Box<dyn Fn(&(dyn Any + Send + Sync)) + Send + Sync>;

/// A boxed intercept handler, as the bus core stores it.
pub type InterceptFn = Box<dyn Fn(&mut (dyn Any + Send + Sync)) -> Flow + Send + Sync>;

/// The bus core's name for one registered handler.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SubscriptionId(u64);

impl SubscriptionId {
    /// Wraps the core's raw id.
    #[must_use]
    pub const fn new(raw: u64) -> Self {
        Self(raw)
    }

    /// The raw id.
    #[must_use]
    pub const fn get(self) -> u64 {
        self.0
    }
}

/// The object-safe bus the kernel implements; modules use the typed `Bus` over it.
pub trait BusCore: Send + Sync {
    /// Calls every handler subscribed to type `ty` with `event`.
    fn publish(&self, ty: TypeId, name: &'static str, event: &(dyn Any + Send + Sync));

    /// Runs the interceptors for `ty`, lowest `order` first, until one returns `Stop`.
    fn intercept(
        &self,
        ty: TypeId,
        name: &'static str,
        event: &mut (dyn Any + Send + Sync),
    ) -> Flow;

    /// Adds a broadcast handler for type `ty`.
    fn subscribe(&self, ty: TypeId, name: &'static str, handler: NotifyFn) -> SubscriptionId;

    /// Adds an interceptor for type `ty` at `order`; ties run in the order they were added.
    fn add_interceptor(
        &self,
        ty: TypeId,
        name: &'static str,
        order: i32,
        handler: InterceptFn,
    ) -> SubscriptionId;

    /// Removes a handler and ignores unknown ids.
    /// It may run inside a handler, or while a removed handler drops the guards it kept.
    fn unsubscribe(&self, id: SubscriptionId);
}

/// A cloneable typed handle to the bus, safe to move to worker threads.
#[derive(Clone)]
pub struct Bus(Arc<dyn BusCore>);

impl Bus {
    /// Wraps the kernel's bus core.
    #[must_use]
    pub fn new(core: Arc<dyn BusCore>) -> Self {
        Self(core)
    }

    /// Sends `event` to every subscriber of `E`.
    pub fn publish<E: Event>(&self, event: &E) {
        self.0.publish(TypeId::of::<E>(), E::NAME, event);
    }

    /// Runs `E`'s intercept chain on `event` and says whether a handler stopped it.
    #[must_use]
    pub fn intercept<E: Event>(&self, event: &mut E) -> Flow {
        self.0.intercept(TypeId::of::<E>(), E::NAME, event)
    }

    /// Calls `handler` for every published `E` until the returned guard drops.
    pub fn subscribe<E: Event>(
        &self,
        handler: impl Fn(&E) + Send + Sync + 'static,
    ) -> Subscription {
        let raw: NotifyFn = Box::new(move |event| {
            if let Some(event) = event.downcast_ref::<E>() {
                handler(event);
            }
        });
        let id = self.0.subscribe(TypeId::of::<E>(), E::NAME, raw);
        Subscription::new(&self.0, id)
    }

    /// Adds `handler` to `E`'s intercept chain at `order` until the returned guard drops.
    pub fn add_interceptor<E: Event>(
        &self,
        order: i32,
        handler: impl Fn(&mut E) -> Flow + Send + Sync + 'static,
    ) -> Subscription {
        let raw: InterceptFn =
            Box::new(move |event| event.downcast_mut::<E>().map_or(Flow::Continue, &handler));
        let id = self
            .0
            .add_interceptor(TypeId::of::<E>(), E::NAME, order, raw);
        Subscription::new(&self.0, id)
    }
}

/// Keeps one handler registered; dropping it unsubscribes.
/// It holds the core weakly, so a handler that keeps one makes no cycle.
#[must_use = "dropping a Subscription unsubscribes its handler"]
pub struct Subscription {
    core: Weak<dyn BusCore>,
    id: SubscriptionId,
}

impl Subscription {
    fn new(core: &Arc<dyn BusCore>, id: SubscriptionId) -> Self {
        Self {
            core: Arc::downgrade(core),
            id,
        }
    }
}

impl Drop for Subscription {
    fn drop(&mut self) {
        if let Some(core) = self.core.upgrade() {
            core.unsubscribe(self.id);
        }
    }
}
