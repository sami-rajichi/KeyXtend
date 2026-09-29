//! The typed `Bus` over a fake core: round trips, the intercept chain and subscription guards.

use crate::fakes::FakeCore;
use kx_module_api::{Bus, Event, Flow, Subscription};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, PoisonError};

#[derive(Debug, PartialEq)]
pub struct Ping(pub u32);

impl Event for Ping {
    const NAME: &'static str = "ping";
}

struct Pong;

impl Event for Pong {
    const NAME: &'static str = "pong";
}

type Seen = Arc<Mutex<Vec<u32>>>;

fn bus() -> (Arc<FakeCore>, Bus) {
    let core = Arc::new(FakeCore::default());
    (Arc::clone(&core), Bus::new(core))
}

fn record(bus: &Bus) -> (Seen, Subscription) {
    let seen = Seen::default();
    let log = Arc::clone(&seen);
    let sub = bus.subscribe(move |ping: &Ping| {
        let mut log = log.lock().unwrap_or_else(PoisonError::into_inner);
        log.push(ping.0);
    });
    (seen, sub)
}

#[test]
fn publish_reaches_a_subscriber_with_the_value() {
    let (_core, bus) = bus();
    let (seen, _sub) = record(&bus);
    bus.publish(&Ping(7));
    assert_eq!(*seen.lock().unwrap(), [7]);
}

#[test]
fn intercept_returns_stop_when_a_handler_stops() {
    let (_core, bus) = bus();
    let _late = bus.add_interceptor(2, |ping: &mut Ping| {
        ping.0 += 100;
        Flow::Continue
    });
    let _stop = bus.add_interceptor(1, |ping: &mut Ping| {
        ping.0 += 1;
        Flow::Stop
    });
    let mut ping = Ping(0);
    assert_eq!(bus.intercept(&mut ping), Flow::Stop);
    assert_eq!(ping, Ping(1));
}

#[test]
fn intercept_without_a_stop_continues_with_the_changes() {
    let (_core, bus) = bus();
    let _double = bus.add_interceptor(0, |ping: &mut Ping| {
        ping.0 *= 2;
        Flow::Continue
    });
    let mut ping = Ping(4);
    assert_eq!(bus.intercept(&mut ping), Flow::Continue);
    assert_eq!(ping, Ping(8));
}

#[test]
fn a_handler_for_another_event_type_is_skipped() {
    let (_core, bus) = bus();
    let pongs = Arc::new(AtomicUsize::new(0));
    let count = Arc::clone(&pongs);
    let _sub = bus.subscribe(move |_: &Pong| {
        count.fetch_add(1, Ordering::SeqCst);
    });
    let _stop = bus.add_interceptor(0, |_: &mut Pong| Flow::Stop);
    bus.publish(&Ping(1));
    assert_eq!(bus.intercept(&mut Ping(1)), Flow::Continue);
    assert_eq!(pongs.load(Ordering::SeqCst), 0);
}

#[test]
fn dropping_a_subscription_unsubscribes_it() {
    let (core, bus) = bus();
    let (seen, sub) = record(&bus);
    let stop = bus.add_interceptor(0, |_: &mut Ping| Flow::Stop);
    assert_eq!(core.count(), 2);
    drop((sub, stop));
    assert_eq!(core.count(), 0);
    bus.publish(&Ping(1));
    assert_eq!(bus.intercept(&mut Ping(1)), Flow::Continue);
    assert!(seen.lock().unwrap().is_empty());
}

#[test]
fn a_subscription_outliving_its_core_drops_quietly() {
    let (core, bus) = bus();
    let (_seen, sub) = record(&bus);
    let weak = Arc::downgrade(&core);
    drop((bus, core));
    assert!(weak.upgrade().is_none());
    drop(sub);
}

#[test]
fn a_handler_keeping_a_subscription_makes_no_cycle() {
    let (core, bus) = bus();
    let (_seen, inner) = record(&bus);
    let _outer = bus.subscribe(move |_: &Pong| {
        let _keep = &inner;
    });
    let weak = Arc::downgrade(&core);
    drop((bus, core));
    assert!(weak.upgrade().is_none());
}

#[test]
fn a_cloned_bus_publishes_from_a_worker_thread() {
    let (_core, bus) = bus();
    let (seen, _sub) = record(&bus);
    let worker = bus.clone();
    std::thread::spawn(move || worker.publish(&Ping(3)))
        .join()
        .unwrap();
    assert_eq!(*seen.lock().unwrap(), [3]);
}
