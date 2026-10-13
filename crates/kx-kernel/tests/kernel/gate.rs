//! The event capability gate through real modules: only a granted module may use a gated event.

#![allow(
    clippy::unwrap_used,
    reason = "these fixture helpers run only in tests, where a failed setup should fail the test"
)]

use crate::rig::{Rig, Seen, failed};
use kx_kernel::Policy;
use kx_module_api::Capability::{Microphone, ReadFocusedText};
use kx_module_api::ModuleState::{Active, Failed};
use kx_module_api::{
    Bus, Capability, Event, EventError, Flow, Manifest, Module, ModuleCx, ModuleError, ModuleId,
    Subscription,
};
use std::marker::PhantomData;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

const PROBE: ModuleId = ModuleId::new("probe");
/// Holds the grant, like the keyboard that publishes typed text.
const HOLDER: ModuleId = ModuleId::new("holder");
/// Runs next to `HOLDER` without the grant, like the `ui` module.
const PEER: ModuleId = ModuleId::new("peer");
const NEED: Capability = ReadFocusedText;
const NONE: &[Capability] = &[];
const NEEDS: &[Capability] = &[NEED];

/// An event only a module holding `NEED` may see or send.
#[derive(Clone, Default)]
struct Secret;

impl Event for Secret {
    const NAME: &'static str = "secret";
    const CAPABILITY: Option<Capability> = Some(NEED);
}

/// An event with no capability.
#[derive(Clone, Default)]
struct Open;

impl Event for Open {
    const NAME: &'static str = "open";
}

fn refused() -> EventError {
    EventError::NotGranted(Secret::NAME, NEED)
}

type Tries = Vec<Result<(), EventError>>;

/// What a probe leaves behind: its `cx` results, its bus handle and how often its handlers ran.
#[derive(Clone, Default)]
struct Left {
    cx: Arc<Mutex<Tries>>,
    bus: Arc<Mutex<Option<Bus>>>,
    heard: Arc<AtomicUsize>,
    ran: Arc<AtomicUsize>,
}

impl Left {
    fn cx(&self) -> Tries {
        self.cx.lock().unwrap().clone()
    }

    fn bus(&self) -> Bus {
        self.bus.lock().unwrap().clone().unwrap()
    }

    fn heard(&self) -> usize {
        self.heard.load(Ordering::SeqCst)
    }

    fn ran(&self) -> usize {
        self.ran.load(Ordering::SeqCst)
    }
}

/// A module that tries `E` through its `cx` at start, and keeps its bus handle for later.
struct Probe<E> {
    manifest: &'static Manifest,
    strict: bool,
    left: Left,
    event: PhantomData<fn() -> E>,
}

impl<E: Event + Default> Module for Probe<E> {
    fn manifest(&self) -> &'static Manifest {
        self.manifest
    }

    fn start(&mut self, cx: &mut ModuleCx<'_>) -> Result<(), ModuleError> {
        let heard = Arc::clone(&self.left.heard);
        let seen = cx.subscribe(move |_: &E| {
            heard.fetch_add(1, Ordering::SeqCst);
        });
        if self.strict {
            seen?;
        }
        let ran = Arc::clone(&self.left.ran);
        let stopped = cx.intercept(0, move |_: &mut E| {
            ran.fetch_add(1, Ordering::SeqCst);
            Flow::Stop
        });
        *self.left.cx.lock().unwrap() = vec![seen, stopped];
        *self.left.bus.lock().unwrap() = Some(cx.bus());
        Ok(())
    }
}

/// A probe `id` of `E` that declares `caps`; the manifest leaks so it lives for the whole test.
fn probe<E: Event + Default>(
    id: ModuleId,
    caps: &'static [Capability],
    strict: bool,
) -> (Probe<E>, Left) {
    let manifest = Manifest {
        id,
        version: env!("CARGO_PKG_VERSION"),
        requires: &[],
        provides: &[],
        capabilities: caps,
        settings: None,
    };
    let left = Left::default();
    let module = Probe {
        manifest: Box::leak(Box::new(manifest)),
        strict,
        left: left.clone(),
        event: PhantomData,
    };
    (module, left)
}

/// A booted rig with one probe of `E`, and what the probe left behind.
fn boot<E: Event + Default>(policy: Policy, caps: &'static [Capability]) -> (Rig, Left) {
    let mut rig = Rig::with(policy);
    let (module, left) = probe::<E>(PROBE, caps, false);
    rig.kernel.add(Box::new(module)).unwrap();
    rig.kernel.boot().unwrap();
    assert_eq!(rig.state(PROBE), Some(Active));
    (rig, left)
}

/// The four bus calls on `E` from a worker thread, as the module would make them.
fn four<E: Event + Default>(bus: Bus) -> Tries {
    let work = move || {
        vec![
            bus.subscribe(|_: &E| {}).map(drop),
            bus.add_interceptor(0, |_: &mut E| Flow::Continue).map(drop),
            bus.publish(&E::default()),
            bus.intercept(&mut E::default()).map(drop),
        ]
    };
    std::thread::spawn(work).join().unwrap()
}

/// A kernel-side interceptor of `Secret` that counts its runs, so a test sees a chain reach it.
fn count_chain(rig: &Rig) -> (Arc<AtomicUsize>, Subscription) {
    let count = Arc::new(AtomicUsize::new(0));
    let hit = Arc::clone(&count);
    let guard = rig.kernel.bus().add_interceptor(9, move |_: &mut Secret| {
        hit.fetch_add(1, Ordering::SeqCst);
        Flow::Continue
    });
    (count, guard.unwrap())
}

#[test]
fn without_the_grant_every_gated_call_is_refused_and_nothing_moves() {
    let allowed_not_declared = Policy::new().allow(PROBE, &[NEED]);
    let declared_not_allowed = Policy::new().allow(PROBE, &[Microphone]);
    for (policy, caps) in [(allowed_not_declared, NONE), (declared_not_allowed, NEEDS)] {
        let (rig, left) = boot::<Secret>(policy, caps);
        let (watch, _sub) = Seen::<Secret>::watch(&rig.kernel.bus());
        let (chain, _guard) = count_chain(&rig);
        assert_eq!(left.cx(), [Err(refused()), Err(refused())]);
        assert_eq!(four::<Secret>(left.bus()), [Err(refused()); 4]);
        assert!(watch.all().is_empty(), "a refused publish reaches nobody");
        assert_eq!(
            chain.load(Ordering::SeqCst),
            0,
            "a refused chain never runs"
        );
        rig.kernel.bus().publish(&Secret).unwrap();
        assert_eq!(watch.all().len(), 1);
        assert_eq!(left.heard(), 0, "its refused subscribe registered nothing");
        assert_eq!(rig.kernel.bus().intercept(&mut Secret), Ok(Flow::Continue));
        assert_eq!((chain.load(Ordering::SeqCst), left.ran()), (1, 0));
    }
}

#[test]
fn with_the_grant_a_module_may_do_all_four() {
    let policy = Policy::new().allow(PROBE, &[NEED]);
    let (rig, left) = boot::<Secret>(policy, NEEDS);
    let (watch, _sub) = Seen::<Secret>::watch(&rig.kernel.bus());
    assert_eq!(left.cx(), [Ok(()), Ok(())]);
    assert_eq!(four::<Secret>(left.bus()), [Ok(()); 4]);
    assert_eq!(watch.all().len(), 1, "its publish reached the watcher");
    rig.kernel.bus().publish(&Secret).unwrap();
    assert_eq!(left.heard(), 2, "its own handler heard both");
    let flow = left.bus().intercept(&mut Secret);
    assert_eq!((flow, left.ran()), (Ok(Flow::Stop), 2));
}

#[test]
fn a_gated_publish_reaches_only_the_module_with_the_grant() {
    for peer_caps in [NONE, NEEDS] {
        let mut rig = Rig::with(Policy::new().allow(HOLDER, NEEDS));
        let (holder, a) = probe::<Secret>(HOLDER, NEEDS, false);
        let (peer, b) = probe::<Secret>(PEER, peer_caps, false);
        rig.kernel.add(Box::new(holder)).unwrap();
        rig.kernel.add(Box::new(peer)).unwrap();
        rig.kernel.boot().unwrap();
        assert_eq!([rig.state(HOLDER), rig.state(PEER)], [Some(Active); 2]);
        assert_eq!(a.cx(), [Ok(()), Ok(())]);
        assert_eq!(b.cx(), [Err(refused()), Err(refused())]);
        assert_eq!(a.bus().publish(&Secret), Ok(()));
        assert_eq!(a.bus().intercept(&mut Secret), Ok(Flow::Stop));
        assert_eq!((a.heard(), a.ran()), (1, 1));
        assert_eq!(
            (b.heard(), b.ran()),
            (0, 0),
            "the peer's handlers never run"
        );
        assert_eq!(four::<Secret>(b.bus()), [Err(refused()); 4]);
        assert_eq!(a.heard(), 1, "the peer's refused publish reaches nobody");
    }
}

#[test]
fn an_event_without_a_capability_works_for_a_module_with_no_grants() {
    let (rig, left) = boot::<Open>(Policy::new(), NONE);
    let (watch, _sub) = Seen::<Open>::watch(&rig.kernel.bus());
    assert_eq!(left.cx(), [Ok(()), Ok(())]);
    assert_eq!(four::<Open>(left.bus()), [Ok(()); 4]);
    assert_eq!(watch.all().len(), 1);
    rig.kernel.bus().publish(&Open).unwrap();
    assert_eq!(left.heard(), 2);
}

#[test]
fn the_kernels_own_handle_may_use_a_gated_event() {
    let rig = Rig::new();
    let bus = rig.kernel.bus();
    let (watch, _sub) = Seen::<Secret>::watch(&bus);
    let (chain, _guard) = count_chain(&rig);
    assert_eq!(bus.publish(&Secret), Ok(()));
    assert_eq!(bus.intercept(&mut Secret), Ok(Flow::Continue));
    assert_eq!((watch.all().len(), chain.load(Ordering::SeqCst)), (1, 1));
}

#[test]
fn a_module_that_passes_on_the_refusal_fails_its_start() {
    let mut rig = Rig::new();
    let (module, left) = probe::<Secret>(PROBE, NEEDS, true);
    rig.kernel.add(Box::new(module)).unwrap();
    let notices = rig.kernel.boot().unwrap();
    assert_eq!(rig.state(PROBE), Some(Failed));
    assert_eq!(notices, [failed(PROBE)]);
    assert!(left.cx().is_empty() && left.heard() == 0);
}
