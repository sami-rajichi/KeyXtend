//! Services, grants and guards through `ModuleCx` over a `StartHost`, with small sample keys.

use super::*;
use crate::bus::KernelBus;
use crate::grants::Policy;
use kx_module_api::Capability::Microphone;
use kx_module_api::ServiceError::{AlreadyProvided, CapabilityMismatch, Missing, NotGranted};
use kx_module_api::{Event, ModuleCx, Mono, ServiceKey};
use kx_platform_fake::FakeClock;
use std::sync::atomic::{AtomicUsize, Ordering};

trait Greet: Send + Sync {}

struct Hello;

impl Greet for Hello {}

/// An open service anyone who requires it may use.
struct Greeter;

impl ServiceKey for Greeter {
    type Api = dyn Greet;
    const ID: ServiceId = ServiceId::new("greeter");
    const CAPABILITY: Option<Capability> = None;
}

/// A service gated on the microphone capability.
struct Mic;

impl ServiceKey for Mic {
    type Api = dyn Greet;
    const ID: ServiceId = ServiceId::new("mic");
    const CAPABILITY: Option<Capability> = Some(Microphone);
}

/// A key with `Mic`'s id that names no capability.
struct SneakyMic;

impl ServiceKey for SneakyMic {
    type Api = dyn Greet;
    const ID: ServiceId = Mic::ID;
    const CAPABILITY: Option<Capability> = None;
}

struct Ping;

impl Event for Ping {
    const NAME: &'static str = "ping";
}

const START: Mono = Mono::from_us(7);

/// Each sample module declares the microphone; the policy denies it to the rogue.
const fn sample(id: &'static str, requires: &'static [ServiceId]) -> Manifest {
    Manifest {
        id: ModuleId::new(id),
        version: env!("CARGO_PKG_VERSION"),
        requires,
        provides: &[Greeter::ID, Mic::ID],
        capabilities: &[Microphone],
        settings: None,
    }
}

static PROVIDER: Manifest = sample("provider", &[]);
static CONSUMER: Manifest = sample("consumer", &[Greeter::ID, Mic::ID]);
static ROGUE: Manifest = sample("rogue", &[Greeter::ID, Mic::ID]);

/// The kernel side the hosts share: one registry, bus, clock and policy.
struct Env {
    registry: Registry,
    policy: Policy,
    bus: Bus,
    clock: FakeClock,
}

impl Env {
    fn new() -> Self {
        Self {
            registry: Registry::new(),
            policy: Policy::new()
                .allow(PROVIDER.id, &[Microphone])
                .allow(CONSUMER.id, &[Microphone]),
            bus: Bus::new(Arc::new(KernelBus::new())),
            clock: FakeClock::new(START),
        }
    }

    fn host(
        &mut self,
        manifest: &'static Manifest,
        settings: Option<toml::Table>,
    ) -> StartHost<'_> {
        let grants = Grants::new(manifest, &self.policy);
        let clock = Arc::new(self.clock.clone());
        StartHost::new(
            manifest,
            &mut self.registry,
            grants,
            self.bus.clone(),
            clock,
            settings,
        )
    }

    fn provide<K: ServiceKey<Api = dyn Greet>>(
        &mut self,
        manifest: &'static Manifest,
        service: Arc<dyn Greet>,
    ) -> Result<(), ServiceError> {
        let mut host = self.host(manifest, None);
        ModuleCx::new(&mut host).provide::<K>(service)
    }

    fn get<K: ServiceKey<Api = dyn Greet>>(
        &mut self,
        manifest: &'static Manifest,
    ) -> Result<Arc<dyn Greet>, ServiceError> {
        let mut host = self.host(manifest, None);
        ModuleCx::new(&mut host).service::<K>()
    }

    fn refusal<K: ServiceKey<Api = dyn Greet>>(
        &mut self,
        manifest: &'static Manifest,
    ) -> Option<ServiceError> {
        self.get::<K>(manifest).err()
    }
}

/// A `Ping` handler and the count of its calls.
fn counter() -> (Arc<AtomicUsize>, impl Fn(&Ping) + Send + Sync + 'static) {
    let count = Arc::new(AtomicUsize::new(0));
    let hits = Arc::clone(&count);
    (count, move |_: &Ping| {
        hits.fetch_add(1, Ordering::SeqCst);
    })
}

fn hello() -> Arc<dyn Greet> {
    Arc::new(Hello)
}

#[test]
fn a_provided_service_is_the_same_arc_for_another_module() {
    let (mut env, greeter, mic) = (Env::new(), hello(), hello());
    let mut host = env.host(&PROVIDER, None);
    let mut cx = ModuleCx::new(&mut host);
    cx.provide::<Greeter>(Arc::clone(&greeter)).unwrap();
    cx.provide::<Mic>(Arc::clone(&mic)).unwrap();
    assert_eq!(host.into_held().provided(), [Greeter::ID, Mic::ID]);
    let got = env.get::<Greeter>(&CONSUMER).unwrap();
    assert!(Arc::ptr_eq(&got, &greeter));
    assert!(Arc::ptr_eq(&env.get::<Mic>(&CONSUMER).unwrap(), &mic));
}

#[test]
fn an_unprovided_service_is_missing() {
    let err = Env::new().refusal::<Greeter>(&CONSUMER);
    assert_eq!(err, Some(Missing(Greeter::ID)));
}

#[test]
fn a_key_naming_another_capability_is_a_mismatch() {
    let mut env = Env::new();
    env.provide::<Mic>(&PROVIDER, hello()).unwrap();
    let err = env.refusal::<SneakyMic>(&CONSUMER);
    assert_eq!(err, Some(CapabilityMismatch(Mic::ID)));
}

#[test]
fn a_requester_without_the_grant_is_not_granted() {
    let mut env = Env::new();
    env.provide::<Mic>(&PROVIDER, hello()).unwrap();
    let err = env.refusal::<Mic>(&ROGUE);
    assert_eq!(err, Some(NotGranted(Mic::ID, Microphone)));
}

#[test]
fn a_provider_without_its_gating_grant_is_not_granted() {
    let mut env = Env::new();
    let refused = env.provide::<Mic>(&ROGUE, hello());
    assert_eq!(refused, Err(NotGranted(Mic::ID, Microphone)));
    assert_eq!(env.refusal::<Mic>(&CONSUMER), Some(Missing(Mic::ID)));
}

#[test]
fn a_second_provider_is_refused_and_the_first_is_kept() {
    let (mut env, first) = (Env::new(), hello());
    let kept = env.provide::<Greeter>(&PROVIDER, Arc::clone(&first));
    assert_eq!(kept, Ok(()));
    let refused = env.provide::<Greeter>(&ROGUE, hello());
    assert_eq!(refused, Err(AlreadyProvided(Greeter::ID)));
    assert!(Arc::ptr_eq(&env.get::<Greeter>(&CONSUMER).unwrap(), &first));
}

#[test]
fn a_module_cannot_provide_a_platform_service() {
    let mut env = Env::new();
    let platform = Box::new(hello());
    env.registry
        .provide(Owner::Platform, Greeter::ID, None, platform)
        .unwrap();
    let refused = env.provide::<Greeter>(&PROVIDER, hello());
    assert_eq!(refused, Err(AlreadyProvided(Greeter::ID)));
}

#[test]
fn release_removes_the_services_and_unsubscribes_the_guards() {
    let (mut env, (count, handler)) = (Env::new(), counter());
    let mut host = env.host(&PROVIDER, None);
    let mut cx = ModuleCx::new(&mut host);
    cx.provide::<Greeter>(hello()).unwrap();
    cx.subscribe(handler);
    let held = host.into_held();
    env.bus.publish(&Ping);
    held.release(&mut env.registry);
    env.bus.publish(&Ping);
    assert_eq!(count.load(Ordering::SeqCst), 1);
    assert_eq!(
        env.refusal::<Greeter>(&CONSUMER),
        Some(Missing(Greeter::ID))
    );
}

#[test]
fn remove_owned_keeps_the_platform_services() {
    let mut env = Env::new();
    let (registry, platform, module): (_, Service, Service) =
        (&mut env.registry, Box::new(hello()), Box::new(hello()));
    let mic = registry.provide(Owner::Platform, Mic::ID, Some(Microphone), platform);
    let provider = Owner::Module(PROVIDER.id);
    let greeter = registry.provide(provider, Greeter::ID, None, module);
    assert_eq!((mic, greeter), (Ok(()), Ok(())));
    assert_eq!(registry.remove_owned(PROVIDER.id).len(), 1);
    assert!(registry.remove_owned(PROVIDER.id).is_empty());
    assert_eq!(env.refusal::<Mic>(&CONSUMER), None);
    assert_eq!(
        env.refusal::<Greeter>(&CONSUMER),
        Some(Missing(Greeter::ID))
    );
}

#[test]
fn the_host_passes_on_the_clock_bus_and_settings() {
    let mut env = Env::new();
    let table: toml::Table = toml::from_str("speed = 3").unwrap();
    let host = env.host(&PROVIDER, Some(table.clone()));
    assert_eq!(host.settings(), &table);
    assert_eq!(host.clock().now(), START);
    let (count, handler) = counter();
    let _sub = host.bus().subscribe(handler);
    drop(host);
    env.bus.publish(&Ping);
    assert_eq!(count.load(Ordering::SeqCst), 1);
}

#[test]
fn a_module_without_settings_gets_an_empty_table() {
    let mut env = Env::new();
    let host = env.host(&PROVIDER, None);
    assert!(host.settings().is_empty());
    assert_eq!(host.manifest().id, PROVIDER.id);
}
