//! The kernel's event bus: broadcast handlers and ordered intercept chains.
//! No lock is held while a handler runs or drops, so handlers may call back into the bus.

use kx_module_api::{BusCore, Flow, InterceptFn, NotifyFn, SubscriptionId};
use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};

/// The first subscription id the bus hands out.
const FIRST_ID: u64 = 1;

/// Notify handlers share one order, so they run in subscription order.
const NOTIFY_ORDER: i32 = 0;

/// A broadcast handler, shared so a publish can run it without the lock.
type Notify = dyn Fn(&(dyn Any + Send + Sync)) + Send + Sync;

/// An intercept handler, shared so a chain can run it without the lock.
type Intercept = dyn Fn(&mut (dyn Any + Send + Sync)) -> Flow + Send + Sync;

/// One registered handler and its place in the chain.
struct Entry<F: ?Sized> {
    id: SubscriptionId,
    order: i32,
    run: Arc<F>,
}

/// Handlers of one kind per event type, each list kept in run order.
struct Table<F: ?Sized>(HashMap<TypeId, Vec<Entry<F>>>);

impl<F: ?Sized> Default for Table<F> {
    fn default() -> Self {
        Self(HashMap::new())
    }
}

impl<F: ?Sized> Table<F> {
    /// Adds `run` after every handler of the same or a lower order.
    fn add(&mut self, ty: TypeId, id: SubscriptionId, order: i32, run: Arc<F>) {
        let list = self.0.entry(ty).or_default();
        let at = list.partition_point(|e| e.order <= order);
        list.insert(at, Entry { id, order, run });
    }

    /// The handlers for `ty` in run order, copied so they run after the lock is released.
    fn chain(&self, ty: TypeId) -> Vec<(SubscriptionId, Arc<F>)> {
        self.0.get(&ty).map_or_else(Vec::new, |list| {
            list.iter().map(|e| (e.id, Arc::clone(&e.run))).collect()
        })
    }

    /// Takes out handler `id`; the caller drops it after releasing the lock.
    fn remove(&mut self, id: SubscriptionId) -> Option<Arc<F>> {
        let (ty, at) = self
            .0
            .iter()
            .find_map(|(ty, list)| Some((*ty, list.iter().position(|e| e.id == id)?)))?;
        let list = self.0.get_mut(&ty)?;
        let entry = list.remove(at);
        if list.is_empty() {
            self.0.remove(&ty);
        }
        Some(entry.run)
    }
}

/// Every handler, behind the bus's one lock.
#[derive(Default)]
struct Handlers {
    notify: Table<Notify>,
    intercept: Table<Intercept>,
}

/// The kernel's `BusCore`; a panicking handler is logged and skipped.
/// A publish or chain runs the handlers registered when it began.
pub struct KernelBus {
    handlers: Mutex<Handlers>,
    next_id: AtomicU64,
}

impl Default for KernelBus {
    fn default() -> Self {
        Self::new()
    }
}

impl KernelBus {
    /// A bus with no handlers.
    #[must_use]
    pub fn new() -> Self {
        Self {
            handlers: Mutex::default(),
            next_id: AtomicU64::new(FIRST_ID),
        }
    }

    fn lock(&self) -> MutexGuard<'_, Handlers> {
        self.handlers.lock().unwrap_or_else(PoisonError::into_inner)
    }

    fn mint(&self) -> SubscriptionId {
        SubscriptionId::new(self.next_id.fetch_add(1, Ordering::Relaxed))
    }
}

/// Logs a contained handler panic by event name and id only, never the event itself.
fn warn_panic(name: &'static str, id: SubscriptionId) {
    tracing::warn!(
        event = name,
        subscription = id.get(),
        "bus handler panicked"
    );
}

impl BusCore for KernelBus {
    fn publish(&self, ty: TypeId, name: &'static str, event: &(dyn Any + Send + Sync)) {
        let chain = self.lock().notify.chain(ty);
        for (id, run) in chain {
            if catch_unwind(AssertUnwindSafe(|| run(event))).is_err() {
                warn_panic(name, id);
            }
        }
    }

    fn intercept(
        &self,
        ty: TypeId,
        name: &'static str,
        event: &mut (dyn Any + Send + Sync),
    ) -> Flow {
        let chain = self.lock().intercept.chain(ty);
        for (id, run) in chain {
            match catch_unwind(AssertUnwindSafe(|| run(&mut *event))) {
                Ok(Flow::Stop) => return Flow::Stop,
                Ok(Flow::Continue) => {}
                Err(_) => warn_panic(name, id),
            }
        }
        Flow::Continue
    }

    fn subscribe(&self, ty: TypeId, _: &'static str, handler: NotifyFn) -> SubscriptionId {
        let id = self.mint();
        let run = Arc::from(handler);
        self.lock().notify.add(ty, id, NOTIFY_ORDER, run);
        id
    }

    fn add_interceptor(
        &self,
        ty: TypeId,
        _: &'static str,
        order: i32,
        handler: InterceptFn,
    ) -> SubscriptionId {
        let id = self.mint();
        let run = Arc::from(handler);
        self.lock().intercept.add(ty, id, order, run);
        id
    }

    fn unsubscribe(&self, id: SubscriptionId) {
        let mut handlers = self.lock();
        let removed = (handlers.notify.remove(id), handlers.intercept.remove(id));
        drop(handlers);
        // The handlers may own guards that call back in, so they drop after the lock.
        drop(removed);
    }
}

#[cfg(test)]
mod tests;
