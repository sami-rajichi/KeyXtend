//! The pointer hook port and the mouse-assist events: their gates, and a run that stops only when dropped.

use kx_module_api::Capability::{InjectInput, PointerHook as Hook};
use kx_module_api::ports::{
    AssistChanged, AssistToggle, Button, GrabState, HookConfig, HookError, HookRun, HookStats,
    Point, PointerAct, PointerEngine, PointerEvent, PointerHook, PointerHookKey, Toggle,
};
use kx_module_api::{
    Capability, Event, EventError, Host, Manifest, ModuleCx, ModuleId, Mono, ServiceError,
    ServiceKey,
};
use kx_test_support::host::FakeHost;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, PoisonError};

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

    fn release(&mut self, acts: &mut Vec<PointerAct>) {
        acts.push(LET_GO);
    }
}

/// What the fake runs did: wakes, drops and the input their drops let go.
#[derive(Default)]
struct Log {
    wakes: AtomicUsize,
    drops: AtomicUsize,
    sent: Mutex<Vec<PointerAct>>,
}

/// A run that keeps its engine and lets go only when dropped, as a real host does.
struct FakeRun {
    engine: Mutex<Box<dyn PointerEngine>>,
    log: Arc<Log>,
}

impl HookRun for FakeRun {
    fn wake(&self) {
        self.log.wakes.fetch_add(1, Ordering::SeqCst);
    }

    fn stats(&self) -> HookStats {
        HookStats::default()
    }
}

impl Drop for FakeRun {
    fn drop(&mut self) {
        let engine = self
            .engine
            .get_mut()
            .unwrap_or_else(PoisonError::into_inner);
        let mut sent = self.log.sent.lock().unwrap_or_else(PoisonError::into_inner);
        engine.release(&mut sent);
        self.log.drops.fetch_add(1, Ordering::SeqCst);
    }
}

/// A hook whose runs all write to one log.
#[derive(Default)]
struct FakeHook {
    log: Arc<Log>,
}

impl PointerHook for FakeHook {
    fn start(
        &self,
        engine: Box<dyn PointerEngine>,
        _: HookConfig,
    ) -> Result<Box<dyn HookRun>, HookError> {
        let engine = Mutex::new(engine);
        let log = Arc::clone(&self.log);
        Ok(Box::new(FakeRun { engine, log }))
    }
}

/// A mouse host granted `granted`, holding the fake hook as the platform would provide it.
fn host(granted: &[Capability]) -> (FakeHost, Arc<Log>) {
    let hook = FakeHook::default();
    let log = Arc::clone(&hook.log);
    let api: Arc<dyn PointerHook> = Arc::new(hook);
    let mut host = FakeHost::new(&MOUSE);
    host.granted.extend(granted);
    let stored = (PointerHookKey::CAPABILITY, Box::new(api) as _);
    host.services.insert(PointerHookKey::ID, stored);
    (host, log)
}

#[test]
fn the_hook_is_named_and_gated_by_pointer_hook() {
    assert_eq!(PointerHookKey::ID.as_str(), "pointer-hook");
    assert_eq!(PointerHookKey::CAPABILITY, Some(Hook));
}

#[test]
fn a_module_without_pointer_hook_cannot_get_the_hook() {
    let (mut host, _log) = host(&[InjectInput]);
    let cx = ModuleCx::new(&mut host);
    let refused = ServiceError::NotGranted(PointerHookKey::ID, Hook);
    assert_eq!(cx.service::<PointerHookKey>().err(), Some(refused));
}

#[test]
fn a_run_stops_only_when_dropped() {
    let (mut host, log) = host(&[Hook]);
    let hook = ModuleCx::new(&mut host)
        .service::<PointerHookKey>()
        .unwrap();
    let run = hook.start(Box::new(Holder), CONFIG).unwrap();
    run.wake();
    run.wake();
    assert_eq!(run.stats(), HookStats::default());
    assert_eq!(log.drops.load(Ordering::SeqCst), 0, "a wake never stops it");
    drop(run);
    let counts = [&log.wakes, &log.drops].map(|n| n.load(Ordering::SeqCst));
    assert_eq!(counts, [2, 1], "two wakes, one drop");
    assert_eq!(*log.sent.lock().unwrap(), [LET_GO], "the drop let go");
}

#[test]
fn a_module_without_inject_input_cannot_publish_a_toggle() {
    let (host, _log) = host(&[Hook]);
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
    let (host, _log) = host(&[InjectInput]);
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
