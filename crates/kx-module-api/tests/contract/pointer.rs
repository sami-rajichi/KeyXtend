//! The pointer hook port and the mouse-assist events: their gates, and a run reached through the gate, on the shared
//! fake hook, that stops only when dropped.

use kx_module_api::Capability::{InjectInput, PointerHook as Hook};
use kx_module_api::ports::{
    AssistChanged, AssistToggle, Button, GrabState, HookConfig, HookStats, Point, PointerAct,
    PointerEngine, PointerEvent, PointerHook, PointerHookKey, Toggle,
};
use kx_module_api::{
    Capability, Event, EventError, Host, Manifest, ModuleCx, ModuleId, Mono, ServiceError,
    ServiceKey,
};
use kx_platform_fake::FakePointerHook;
use kx_test_support::host::FakeHost;
use std::sync::{Arc, Mutex};

/// A mouse module declaring both capabilities it needs; each test grants it some, as the kernel's policy would.
static MOUSE: Manifest = Manifest {
    id: ModuleId::new("mouse"),
    version: env!("CARGO_PKG_VERSION"),
    requires: &[PointerHookKey::ID],
    provides: &[],
    capabilities: &[Hook, InjectInput],
    settings: None,
};

/// Test values; the real ones come from the mouse module's settings.
const CONFIG: HookConfig = HookConfig {
    health_ms: 1,
    health_slack_ms: 2,
    echo_ms: 3,
};

/// What the holder engine lets go of.
const LET_GO: PointerAct = PointerAct::Up(Button::Left, Point { x: 0, y: 0 });

/// An engine that passes every event and holds the left button until released.
struct Holder;

impl PointerEngine for Holder {
    fn on(&mut self, _: PointerEvent, _: Mono, _: &mut Vec<PointerAct>) -> bool {
        false
    }

    fn settle(&mut self, _: Mono, _: &mut Vec<PointerAct>) {}

    fn deadline(&self) -> Option<Mono> {
        None
    }

    fn has_notes(&self) -> bool {
        false
    }

    fn release(&mut self, acts: &mut Vec<PointerAct>) {
        acts.push(LET_GO);
    }

    fn max_acts(&self) -> usize {
        [LET_GO].len()
    }
}

/// A mouse host granted `granted`, holding the shared fake hook as the platform would provide it.
fn host(granted: &[Capability]) -> (FakeHost, Arc<FakePointerHook>) {
    let hook = Arc::new(FakePointerHook::new());
    let api: Arc<dyn PointerHook> = hook.clone();
    let mut host = FakeHost::new(&MOUSE);
    host.granted.extend(granted);
    let stored = (PointerHookKey::CAPABILITY, Box::new(api) as _);
    host.services.insert(PointerHookKey::ID, stored);
    (host, hook)
}

#[test]
fn the_hook_is_named_and_gated_by_pointer_hook() {
    assert_eq!(PointerHookKey::ID.as_str(), "pointer-hook");
    assert_eq!(PointerHookKey::CAPABILITY, Some(Hook));
}

#[test]
fn a_module_without_pointer_hook_cannot_get_the_hook() {
    let (mut host, _fake) = host(&[InjectInput]);
    let cx = ModuleCx::new(&mut host);
    let refused = ServiceError::NotGranted(PointerHookKey::ID, Hook);
    assert_eq!(cx.service::<PointerHookKey>().err(), Some(refused));
}

#[test]
fn a_run_reached_through_the_gate_stops_only_when_dropped() {
    let (mut host, fake) = host(&[Hook]);
    let hook = ModuleCx::new(&mut host)
        .service::<PointerHookKey>()
        .unwrap();
    let run = hook.start(Box::new(Holder), CONFIG).unwrap();
    run.wake();
    run.wake();
    assert_eq!(run.stats(), HookStats::default());
    assert!(fake.running(), "a wake never stops it");
    drop(run);
    let counts = (fake.wakes(), fake.stops(), fake.running());
    assert_eq!(counts, (2, 1, false), "two wakes, one drop");
    assert_eq!(fake.sent(), [LET_GO], "the drop let go");
}

#[test]
fn a_module_without_inject_input_cannot_publish_a_toggle() {
    let (host, _fake) = host(&[Hook]);
    let toggle = AssistToggle {
        which: Toggle::Grab,
    };
    let refused = EventError::NotGranted(AssistToggle::NAME, InjectInput);
    assert_eq!(host.bus().publish(&toggle), Err(refused));
    let state = AssistChanged {
        live: true,
        rclick: false,
        grab: GrabState::Off,
    };
    assert_eq!(host.bus().publish(&state), Ok(()), "the state is open");
}

#[test]
fn a_granted_toggle_reaches_its_subscriber() {
    let (host, _fake) = host(&[InjectInput]);
    let seen = Arc::new(Mutex::new(Vec::new()));
    let log = Arc::clone(&seen);
    let _sub = host
        .bus()
        .subscribe(move |t: &AssistToggle| log.lock().unwrap().push(t.which))
        .unwrap();
    let toggle = AssistToggle {
        which: Toggle::RightClick,
    };
    host.bus().publish(&toggle).unwrap();
    assert_eq!(*seen.lock().unwrap(), [Toggle::RightClick]);
}
