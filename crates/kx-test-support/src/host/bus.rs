//! A bus core that calls every handler whatever the event type, with no lock held while they run.
//! That way the typed wrappers' downcast guard is exercised.

use super::sink::lock;
use kx_module_api::{BusCore, Flow, InterceptFn, NotifyFn, SubscriptionId};
use std::any::{Any, TypeId};
use std::sync::{Arc, Mutex, MutexGuard};

type SharedNotify = Arc<dyn Fn(&(dyn Any + Send + Sync)) + Send + Sync>;
type SharedIntercept = Arc<dyn Fn(&mut (dyn Any + Send + Sync)) -> Flow + Send + Sync>;

/// The handlers on the bus, and the last id given out.
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

/// A bus core for tests: every handler hears every event.
#[derive(Default)]
pub struct FakeCore(Mutex<Handlers>);

impl FakeCore {
    /// How many handlers are registered.
    #[must_use]
    pub fn count(&self) -> usize {
        let handlers = self.lock();
        handlers.notify.len() + handlers.intercept.len()
    }

    fn lock(&self) -> MutexGuard<'_, Handlers> {
        lock(&self.0)
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
