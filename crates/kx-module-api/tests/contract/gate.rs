//! The event capability gate: a limited `Bus` and a `ModuleCx` refuse gated events without the grant.

use kx_module_api::Capability::{Microphone, ReadFocusedText};
use kx_module_api::{
    Bus, Capability, Event, EventError, Flow, Manifest, ModuleCx, ModuleError, ModuleId,
};
use kx_test_support::host::{FakeCore, FakeHost};
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

/// The capability `Secret` needs.
const NEED: Capability = ReadFocusedText;

/// An event only a module holding `NEED` may see or send.
struct Secret;

impl Event for Secret {
    const NAME: &'static str = "secret";
    const CAPABILITY: Option<Capability> = Some(NEED);
}

/// An event with no capability.
struct Open;

impl Event for Open {
    const NAME: &'static str = "open";
}

/// A module that declares both capabilities; each test grants it some, as the kernel's policy would.
static DECLARED: Manifest = Manifest {
    id: ModuleId::new("declared"),
    version: env!("CARGO_PKG_VERSION"),
    requires: &[],
    provides: &[],
    capabilities: &[NEED, Microphone],
    settings: None,
};

fn refused() -> EventError {
    EventError::NotGranted(Secret::NAME, NEED)
}

fn open_bus() -> (Arc<FakeCore>, Bus) {
    let core = Arc::new(FakeCore::default());
    (Arc::clone(&core), Bus::new(core))
}

/// A counter and a `Secret` handler that adds to it.
fn counter() -> (Arc<AtomicUsize>, impl Fn(&Secret) + Send + Sync + 'static) {
    let count = Arc::new(AtomicUsize::new(0));
    let hits = Arc::clone(&count);
    (count, move |_: &Secret| {
        hits.fetch_add(1, Ordering::SeqCst);
    })
}

/// The four bus calls on `Secret`, as ok or the error; guards are dropped at once.
fn four(bus: &Bus) -> [Result<(), EventError>; 4] {
    [
        bus.subscribe(|_: &Secret| {}).map(drop),
        bus.add_interceptor(0, |_: &mut Secret| Flow::Continue)
            .map(drop),
        bus.publish(&Secret),
        bus.intercept(&mut Secret).map(drop),
    ]
}

#[test]
fn a_bus_without_a_limit_passes_gated_events() {
    let (_core, bus) = open_bus();
    assert_eq!(four(&bus), [Ok(()), Ok(()), Ok(()), Ok(())]);
}

#[test]
fn a_bus_without_the_grant_refuses_all_four_calls() {
    let (_core, bus) = open_bus();
    let limited = bus.restricted(&[Microphone]);
    assert_eq!(four(&limited), [Err(refused()); 4]);
    assert_eq!(four(&bus.restricted(&[])), [Err(refused()); 4]);
}

#[test]
fn a_bus_with_the_grant_may_do_all_four() {
    let (_core, bus) = open_bus();
    let limited = bus.restricted(&[Microphone, NEED]);
    assert_eq!(four(&limited), [Ok(()), Ok(()), Ok(()), Ok(())]);
}

#[test]
fn a_refused_call_registers_and_delivers_nothing() {
    let (core, bus) = open_bus();
    let limited = bus.restricted(&[]);
    let (count, handler) = counter();
    assert_eq!(limited.subscribe(handler).err(), Some(refused()));
    let hit = Arc::clone(&count);
    let stop = limited.add_interceptor(0, move |_: &mut Secret| {
        hit.fetch_add(1, Ordering::SeqCst);
        Flow::Stop
    });
    assert_eq!(stop.err(), Some(refused()));
    assert_eq!(core.count(), 0, "nothing was registered");
    let (seen, watcher) = counter();
    let _watch = bus.subscribe(watcher).unwrap();
    let hit = Arc::clone(&seen);
    let _stop = bus
        .add_interceptor(0, move |_: &mut Secret| {
            hit.fetch_add(10, Ordering::SeqCst);
            Flow::Stop
        })
        .unwrap();
    assert_eq!(limited.publish(&Secret), Err(refused()));
    assert_eq!(limited.intercept(&mut Secret).err(), Some(refused()));
    assert_eq!(seen.load(Ordering::SeqCst), 0, "nothing was delivered");
    assert_eq!(count.load(Ordering::SeqCst), 0);
}

#[test]
fn an_event_without_a_capability_passes_on_any_limit() {
    let (_core, bus) = open_bus();
    let limited = bus.restricted(&[]);
    let _sub = limited.subscribe(|_: &Open| {}).unwrap();
    let _stop = limited
        .add_interceptor(0, |_: &mut Open| Flow::Stop)
        .unwrap();
    assert_eq!(limited.publish(&Open), Ok(()));
    assert_eq!(limited.intercept(&mut Open), Ok(Flow::Stop));
}

#[test]
fn limiting_again_only_narrows() {
    let (_core, bus) = open_bus();
    let narrowed = bus.restricted(&[NEED]).restricted(&[]);
    assert_eq!(four(&narrowed), [Err(refused()); 4]);
    let widened = bus.restricted(&[]).restricted(&[NEED]);
    assert_eq!(
        four(&widened),
        [Err(refused()); 4],
        "a limit cannot be lifted"
    );
}

#[test]
fn a_clone_on_another_thread_keeps_the_limit() {
    let (_core, bus) = open_bus();
    let worker = bus.restricted(&[]).clone();
    let got = std::thread::spawn(move || four(&worker)).join().unwrap();
    assert_eq!(got, [Err(refused()); 4]);
}

/// A host granted `granted`, as the kernel would build it for a module.
fn host(granted: &[Capability]) -> FakeHost {
    let mut host = FakeHost::new(&DECLARED);
    host.granted.extend(granted);
    host
}

#[test]
fn the_cx_refuses_a_gated_event_without_the_grant() {
    let mut host = host(&[Microphone]);
    let mut cx = ModuleCx::new(&mut host);
    assert_eq!(cx.subscribe(|_: &Secret| {}), Err(refused()));
    assert_eq!(cx.intercept(0, |_: &mut Secret| Flow::Stop), Err(refused()));
    assert_eq!(four(&cx.bus()), [Err(refused()); 4]);
    assert!(host.held.is_empty(), "nothing is held");
}

#[test]
fn the_cx_accepts_a_gated_event_with_the_grant() {
    let mut host = host(&[NEED]);
    let mut cx = ModuleCx::new(&mut host);
    assert_eq!(cx.subscribe(|_: &Secret| {}), Ok(()));
    assert_eq!(cx.intercept(0, |_: &mut Secret| Flow::Stop), Ok(()));
    assert_eq!(cx.bus().intercept(&mut Secret), Ok(Flow::Stop));
    assert_eq!(host.held.len(), 2);
}

#[test]
fn an_open_event_passes_through_the_cx_without_grants() {
    let mut host = host(&[]);
    let mut cx = ModuleCx::new(&mut host);
    assert_eq!(cx.subscribe(|_: &Open| {}), Ok(()));
    assert_eq!(cx.intercept(0, |_: &mut Open| Flow::Continue), Ok(()));
}

#[test]
fn a_refusal_wraps_into_a_module_error_with_question_mark() {
    fn start(cx: &mut ModuleCx<'_>) -> Result<(), ModuleError> {
        cx.subscribe(|_: &Secret| {})?;
        Ok(())
    }
    let mut host = host(&[]);
    let err = start(&mut ModuleCx::new(&mut host)).unwrap_err();
    assert_eq!(err, ModuleError::Event(refused()));
    let text = err.to_string();
    assert!(
        text.contains("secret") && text.contains(NEED.name()),
        "{text}"
    );
}
