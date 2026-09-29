//! `ModuleCx` over a fake host: manifest checks, typed services, held handlers, clock and settings.

use crate::bus::Ping;
use crate::fakes::{FakeHost, NOW};
use kx_module_api::{
    Capability, Flow, Manifest, Module, ModuleCx, ModuleError, ModuleId, ServiceError, ServiceId,
    ServiceKey, Settings,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

trait Greet: Send + Sync {
    fn greet(&self) -> &'static str;
}

struct Hello;

impl Greet for Hello {
    fn greet(&self) -> &'static str {
        "hello"
    }
}

struct Greeter;

impl ServiceKey for Greeter {
    type Api = dyn Greet;
    const ID: ServiceId = ServiceId::new("greeter");
    const CAPABILITY: Option<Capability> = None;
}

struct Mic;

impl ServiceKey for Mic {
    type Api = dyn Greet;
    const ID: ServiceId = ServiceId::new("mic");
    const CAPABILITY: Option<Capability> = Some(Capability::Microphone);
}

static BARE: Manifest = Manifest {
    id: ModuleId::new("bare"),
    version: env!("CARGO_PKG_VERSION"),
    requires: &[],
    provides: &[],
    capabilities: &[],
    settings: None,
};

static FULL: Manifest = Manifest {
    id: ModuleId::new("full"),
    version: env!("CARGO_PKG_VERSION"),
    requires: &[Greeter::ID, Mic::ID],
    provides: &[Greeter::ID],
    capabilities: &[Capability::Microphone],
    settings: None,
};

#[derive(Serialize, Deserialize)]
struct Speed {
    speed: u32,
}

impl Settings for Speed {}

struct Sample;

impl Module for Sample {
    fn manifest(&self) -> &'static Manifest {
        &FULL
    }

    fn start(&mut self, cx: &mut ModuleCx<'_>) -> Result<(), ModuleError> {
        cx.provide::<Greeter>(Arc::new(Hello))?;
        let _mic = cx.service::<Mic>()?;
        Ok(())
    }
}

#[test]
fn service_not_required_is_refused_without_asking_the_host() {
    let mut host = FakeHost::new(&BARE);
    let cx = ModuleCx::new(&mut host);
    assert_eq!(
        cx.service::<Greeter>().err(),
        Some(ServiceError::NotRequired(Greeter::ID))
    );
    assert_eq!(host.lookups.get(), 0);
}

#[test]
fn provide_not_declared_is_refused() {
    let mut host = FakeHost::new(&BARE);
    let mut cx = ModuleCx::new(&mut host);
    let refused = Err(ServiceError::NotDeclared(Greeter::ID));
    assert_eq!(cx.provide::<Greeter>(Arc::new(Hello)), refused);
    assert!(host.services.is_empty());
}

#[test]
fn a_provided_service_comes_back_as_the_same_arc() {
    let mut host = FakeHost::new(&FULL);
    let mut cx = ModuleCx::new(&mut host);
    let hello: Arc<dyn Greet> = Arc::new(Hello);
    cx.provide::<Greeter>(Arc::clone(&hello)).unwrap();
    let got = cx.service::<Greeter>().unwrap();
    assert!(Arc::ptr_eq(&got, &hello));
    assert_eq!(got.greet(), "hello");
}

#[test]
fn a_stored_value_of_another_type_is_wrong_type() {
    let mut host = FakeHost::new(&FULL);
    host.services.insert(Greeter::ID, Box::new(7_u32));
    let cx = ModuleCx::new(&mut host);
    assert_eq!(
        cx.service::<Greeter>().err(),
        Some(ServiceError::WrongType(Greeter::ID))
    );
}

#[test]
fn host_refusals_pass_through() {
    let mut host = FakeHost::new(&FULL);
    let cx = ModuleCx::new(&mut host);
    let not_granted = ServiceError::NotGranted(Mic::ID, Capability::Microphone);
    assert_eq!(cx.service::<Mic>().err(), Some(not_granted));
    assert_eq!(
        cx.service::<Greeter>().err(),
        Some(ServiceError::Missing(Greeter::ID))
    );
    assert_eq!(host.lookups.get(), 2);
}

#[test]
fn handlers_are_held_until_the_host_lets_go() {
    let mut host = FakeHost::new(&BARE);
    let hits = Arc::new(AtomicUsize::new(0));
    let (seen, stopped) = (Arc::clone(&hits), Arc::clone(&hits));
    let mut cx = ModuleCx::new(&mut host);
    cx.subscribe(move |_: &Ping| {
        seen.fetch_add(1, Ordering::SeqCst);
    });
    cx.intercept(0, move |_: &mut Ping| {
        stopped.fetch_add(1, Ordering::SeqCst);
        Flow::Stop
    });
    let bus = cx.bus();
    bus.publish(&Ping(1));
    assert_eq!(bus.intercept(&mut Ping(1)), Flow::Stop);
    assert_eq!((hits.load(Ordering::SeqCst), host.held.len()), (2, 2));
    host.held.clear();
    bus.publish(&Ping(1));
    assert_eq!(bus.intercept(&mut Ping(1)), Flow::Continue);
    assert_eq!(hits.load(Ordering::SeqCst), 2);
}

#[test]
fn manifest_clock_and_settings_come_from_the_host() {
    let mut host = FakeHost::new(&BARE);
    host.settings = "speed = 3".parse().unwrap();
    let cx = ModuleCx::new(&mut host);
    assert_eq!(cx.manifest().id, BARE.id);
    assert_eq!(cx.clock().now(), NOW);
    assert_eq!(cx.settings::<Speed>().map(|s| s.speed), Ok(3));
}

#[test]
fn a_boxed_module_starts_through_the_typed_front() {
    let mut module: Box<dyn Module> = Box::new(Sample);
    let mut host = FakeHost::new(module.manifest());
    let result = module.start(&mut ModuleCx::new(&mut host));
    let refused = ServiceError::NotGranted(Mic::ID, Capability::Microphone);
    assert_eq!(result, Err(ModuleError::Service(refused)));
    assert!(host.services.contains_key(&Greeter::ID));
    module.stop();
}
