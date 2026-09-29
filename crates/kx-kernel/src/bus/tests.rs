//! The typed `Bus` over a real `KernelBus`: routing, chains and re-entrant handlers.

use super::*;
use kx_module_api::{Bus, Event, Subscription};
use std::sync::atomic::AtomicUsize;

struct Ping;

impl Event for Ping {
    const NAME: &'static str = "ping";
}

struct Pong;

impl Event for Pong {
    const NAME: &'static str = "pong";
}

/// An event that interceptors stamp with their tags, in the order they run.
#[derive(Default)]
struct Trace(Vec<&'static str>);

impl Event for Trace {
    const NAME: &'static str = "trace";
}

type Log = Arc<Mutex<Vec<&'static str>>>;
type Count = Arc<AtomicUsize>;

fn bus() -> Bus {
    Bus::new(Arc::new(KernelBus::new()))
}

fn push(log: &Log, tag: &'static str) {
    log.lock().unwrap_or_else(PoisonError::into_inner).push(tag);
}

fn seen(log: &Log) -> Vec<&'static str> {
    log.lock().unwrap().clone()
}

fn tick(count: &Count) {
    count.fetch_add(1, Ordering::SeqCst);
}

fn ticks(count: &Count) -> usize {
    count.load(Ordering::SeqCst)
}

fn stamp(bus: &Bus, order: i32, tag: &'static str, flow: Flow) -> Subscription {
    bus.add_interceptor(order, move |trace: &mut Trace| {
        trace.0.push(tag);
        flow
    })
}

#[test]
fn publish_reaches_subscribers_in_order_and_skips_other_types() {
    let (bus, log) = (bus(), Log::default());
    let tagged = |tag| {
        let log = Arc::clone(&log);
        move |_: &Ping| push(&log, tag)
    };
    let _subs = [
        bus.subscribe(tagged("a")),
        bus.subscribe(tagged("b")),
        bus.subscribe(tagged("c")),
    ];
    let other = Arc::clone(&log);
    let _pong = bus.subscribe(move |_: &Pong| push(&other, "pong"));
    bus.publish(&Ping);
    assert_eq!(seen(&log), ["a", "b", "c"]);
}

/// A core that records what the typed `Bus` hands it, then passes the call on.
#[derive(Default)]
struct Spy {
    inner: KernelBus,
    seen: Mutex<Vec<(TypeId, &'static str)>>,
}

impl Spy {
    fn note(&self, ty: TypeId, name: &'static str) {
        self.seen.lock().unwrap().push((ty, name));
    }
}

impl BusCore for Spy {
    fn publish(&self, ty: TypeId, name: &'static str, event: &(dyn Any + Send + Sync)) {
        self.note(ty, name);
        self.inner.publish(ty, name, event);
    }

    fn intercept(&self, ty: TypeId, name: &'static str, ev: &mut (dyn Any + Send + Sync)) -> Flow {
        self.note(ty, name);
        self.inner.intercept(ty, name, ev)
    }

    fn subscribe(&self, ty: TypeId, name: &'static str, handler: NotifyFn) -> SubscriptionId {
        self.note(ty, name);
        self.inner.subscribe(ty, name, handler)
    }

    fn add_interceptor(
        &self,
        ty: TypeId,
        name: &'static str,
        order: i32,
        handler: InterceptFn,
    ) -> SubscriptionId {
        self.note(ty, name);
        self.inner.add_interceptor(ty, name, order, handler)
    }

    fn unsubscribe(&self, id: SubscriptionId) {
        self.inner.unsubscribe(id);
    }
}

#[test]
fn bus_passes_the_type_and_name_to_the_core() {
    let spy = Arc::new(Spy::default());
    let bus = Bus::new(Arc::clone(&spy) as Arc<dyn BusCore>);
    let count = Count::default();
    let hits = Arc::clone(&count);
    let _sub = bus.subscribe(move |_: &Ping| tick(&hits));
    let _chain = bus.add_interceptor(0, |_: &mut Ping| Flow::Continue);
    bus.publish(&Ping);
    assert_eq!(bus.intercept(&mut Ping), Flow::Continue);
    let ping = (TypeId::of::<Ping>(), Ping::NAME);
    assert_eq!(*spy.seen.lock().unwrap(), [ping; 4]);
    assert_eq!(ticks(&count), 1);
}

#[test]
fn interceptors_run_by_order_then_insertion() {
    let bus = bus();
    let _subs = [
        stamp(&bus, 5, "late-1", Flow::Continue),
        stamp(&bus, -1, "first", Flow::Continue),
        stamp(&bus, 5, "late-2", Flow::Continue),
        stamp(&bus, 0, "middle", Flow::Continue),
    ];
    let mut trace = Trace::default();
    assert_eq!(bus.intercept(&mut trace), Flow::Continue);
    assert_eq!(trace.0, ["first", "middle", "late-1", "late-2"]);
}

#[test]
fn stop_ends_the_chain_and_is_returned() {
    let bus = bus();
    let _subs = [
        stamp(&bus, 3, "after", Flow::Continue),
        stamp(&bus, 1, "go", Flow::Continue),
        stamp(&bus, 2, "stop", Flow::Stop),
    ];
    let mut trace = Trace::default();
    assert_eq!(bus.intercept(&mut trace), Flow::Stop);
    assert_eq!(trace.0, ["go", "stop"]);
}

#[test]
fn an_empty_chain_continues() {
    assert_eq!(bus().intercept(&mut Trace::default()), Flow::Continue);
}

#[test]
fn a_handler_may_publish_another_event() {
    let (bus, count) = (bus(), Count::default());
    let hits = Arc::clone(&count);
    let _pong = bus.subscribe(move |_: &Pong| tick(&hits));
    let inner = bus.clone();
    let _ping = bus.subscribe(move |_: &Ping| inner.publish(&Pong));
    bus.publish(&Ping);
    assert_eq!(ticks(&count), 1);
}

#[test]
fn a_handler_may_subscribe_and_the_new_handler_waits_for_the_next_publish() {
    let (bus, count) = (bus(), Count::default());
    let slot: Arc<Mutex<Option<Subscription>>> = Arc::default();
    let (inner, keep, hits) = (bus.clone(), Arc::clone(&slot), Arc::clone(&count));
    let _adder = bus.subscribe(move |_: &Ping| {
        let mut keep = keep.lock().unwrap_or_else(PoisonError::into_inner);
        if keep.is_none() {
            let hits = Arc::clone(&hits);
            *keep = Some(inner.subscribe(move |_: &Ping| tick(&hits)));
        }
    });
    bus.publish(&Ping);
    assert_eq!(ticks(&count), 0);
    bus.publish(&Ping);
    assert_eq!(ticks(&count), 1);
    assert!(slot.lock().unwrap().is_some());
}

#[test]
fn a_handler_may_drop_its_own_subscription() {
    let (bus, count) = (bus(), Count::default());
    let slot: Arc<Mutex<Option<Subscription>>> = Arc::default();
    let (own, hits) = (Arc::clone(&slot), Arc::clone(&count));
    let sub = bus.subscribe(move |_: &Ping| {
        tick(&hits);
        own.lock().unwrap_or_else(PoisonError::into_inner).take();
    });
    *slot.lock().unwrap() = Some(sub);
    bus.publish(&Ping);
    bus.publish(&Ping);
    assert_eq!(ticks(&count), 1);
}

#[test]
fn a_dropped_handler_may_own_a_subscription_that_reenters_unsubscribe() {
    let (bus, count) = (bus(), Count::default());
    let hits = Arc::clone(&count);
    let keep = bus.subscribe(move |_: &Ping| tick(&hits));
    // The owner handler holds `keep`, so dropping the owner drops `keep` too.
    let owner = bus.subscribe(move |_: &Pong| {
        let _ = &keep;
    });
    drop(owner);
    bus.publish(&Ping);
    assert_eq!(ticks(&count), 0);
}

#[test]
fn a_subscription_dropped_after_the_bus_is_quiet() {
    let bus = bus();
    let sub = bus.subscribe(|_: &Ping| {});
    drop(bus);
    drop(sub);
}

#[test]
fn a_bus_clone_publishes_from_another_thread() {
    let (bus, count) = (bus(), Count::default());
    let hits = Arc::clone(&count);
    let _sub = bus.subscribe(move |_: &Ping| tick(&hits));
    let remote = bus.clone();
    std::thread::spawn(move || remote.publish(&Ping))
        .join()
        .unwrap();
    assert_eq!(ticks(&count), 1);
}

#[test]
fn ids_start_at_one_and_unknown_ids_are_ignored() {
    let core = KernelBus::new();
    let first = core.subscribe(TypeId::of::<Ping>(), Ping::NAME, Box::new(|_| {}));
    assert_eq!(first, SubscriptionId::new(FIRST_ID));
    core.unsubscribe(SubscriptionId::new(u64::MAX));
    core.unsubscribe(first);
    core.unsubscribe(first);
}
