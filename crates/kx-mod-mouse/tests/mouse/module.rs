//! The module's start through the shared fake host, which gates services as the kernel does, and its actor thread
//! over the bus.

use crate::logs::{MESSAGE, Planned, lines_of};
use crate::rig::{RCLICK, SHIFT, T0};
use kx_mod_mouse::{MANIFEST, MouseModule};
use kx_module_api::ports::{
    AssistAsk, AssistChanged, AssistNote, AssistToggle, Dir, GrabState, HookConfig, HookError,
    HookRun, LatchesAsk, LatchesChanged, PointerEngine, PointerHook, PointerHookKey, ScrollAsk,
    ScrollChanged, Toggle,
};
use kx_module_api::{
    Bus, Capability, Event, EventError, Module, ModuleCx, ModuleError, Notice, ServiceError,
    ServiceKey, SettingsError, Subscription, keys,
};
use kx_platform_fake::FakePointerHook;
use kx_test_support::host::{FakeHost, Stored};
use std::sync::Arc;
use std::sync::mpsc::{self, Receiver};
use std::time::Duration;
use tracing::Level;

/// The longest a threaded test waits for the actor; it never sleeps.
const WAIT: Duration = Duration::from_secs(10);

/// A hook that never starts.
struct Idle;

impl PointerHook for Idle {
    fn start(
        &self,
        _: Box<dyn PointerEngine>,
        _: HookConfig,
    ) -> Result<Box<dyn HookRun>, HookError> {
        Err(HookError::Thread)
    }
}

/// A hook whose start panics, as a platform bug would.
struct Broken;

impl PointerHook for Broken {
    fn start(
        &self,
        _: Box<dyn PointerEngine>,
        _: HookConfig,
    ) -> Result<Box<dyn HookRun>, HookError> {
        std::panic::panic_any(Planned)
    }
}

/// A host on the defaults granting `granted`, holding `hook` if any.
fn host(granted: &[Capability], hook: Option<Arc<dyn PointerHook>>) -> FakeHost {
    let mut host = FakeHost::new(&MANIFEST);
    host.granted = granted.to_vec();
    if let Some(hook) = hook {
        let stored: Stored = (PointerHookKey::CAPABILITY, Box::new(hook));
        host.services.insert(PointerHookKey::ID, stored);
    }
    host
}

/// A host granting `granted`, holding a hook that never starts.
fn idle(granted: &[Capability]) -> FakeHost {
    host(granted, Some(Arc::new(Idle)))
}

/// Starts the module on `host`.
fn start(host: &mut FakeHost) -> Result<(), ModuleError> {
    MouseModule::default().start(&mut ModuleCx::new(host))
}

#[test]
fn it_starts_on_the_defaults_with_the_hook_granted() {
    assert_eq!(start(&mut idle(MANIFEST.capabilities)), Ok(()));
}

#[test]
fn without_the_pointer_hook_grant_it_cannot_reach_the_hook() {
    let mut host = idle(&[Capability::InjectInput]);
    let refused = ServiceError::NotGranted(PointerHookKey::ID, Capability::PointerHook);
    assert_eq!(start(&mut host), Err(ModuleError::Service(refused)));
}

#[test]
fn without_keyboard_ui_it_cannot_tell_its_notes_and_fails_to_start() {
    let mut host = idle(&[Capability::PointerHook, Capability::InjectInput]);
    let refused = EventError::NotGranted(AssistNote::NAME, Capability::KeyboardUi);
    assert_eq!(start(&mut host), Err(ModuleError::Event(refused)));
}

#[test]
fn without_a_hook_it_fails_to_start() {
    let missing = ServiceError::Missing(PointerHookKey::ID);
    let refused = Err(ModuleError::Service(missing));
    assert_eq!(start(&mut host(MANIFEST.capabilities, None)), refused);
}

#[test]
fn a_bad_value_fails_the_start() {
    let mut host = idle(MANIFEST.capabilities);
    host.put("hold_ms", 9_000.into());
    let err = start(&mut host);
    assert!(
        matches!(err, Err(ModuleError::Settings(SettingsError::Invalid(_)))),
        "{err:?}"
    );
}

/// A running module on a fake hook, and an unlimited bus to talk to it.
struct Live {
    host: FakeHost,
    hook: Arc<FakePointerHook>,
    module: MouseModule,
    bus: Bus,
}

impl Live {
    /// Starts the module on a host that `setup` changes, after `listen` subscribes on its bus.
    fn start<T>(setup: impl FnOnce(&mut FakeHost), listen: impl FnOnce(&Bus) -> T) -> (Self, T) {
        let hook = Arc::new(FakePointerHook::new());
        let mut host = host(MANIFEST.capabilities, Some(hook.clone()));
        setup(&mut host);
        let bus = Bus::new(Arc::clone(&host.core));
        let heard = listen(&bus);
        let mut module = MouseModule::default();
        module.start(&mut ModuleCx::new(&mut host)).unwrap();
        let live = Self {
            host,
            hook,
            module,
            bus,
        };
        (live, heard)
    }

    /// Stops the module as the kernel does: its handlers leave the bus first.
    fn stop(&mut self) {
        self.host.held.clear();
        self.module.stop();
    }
}

/// Sends every `E` published on `bus` to the returned channel.
fn hear<E: Event + Clone>(bus: &Bus) -> (Subscription, Receiver<E>) {
    let (tx, rx) = mpsc::channel();
    let sub = bus
        .subscribe(move |e: &E| {
            let _ = tx.send(e.clone());
        })
        .unwrap();
    (sub, rx)
}

/// The live state with `rclick` and Grab off.
fn on(rclick: bool) -> AssistChanged {
    let grab = GrabState::Off;
    AssistChanged {
        live: true,
        rclick,
        grab,
    }
}

#[test]
fn the_actor_thread_says_it_is_live_hears_a_toggle_and_stops_with_the_module() {
    let (mut live, (_ear, told)) = Live::start(|_| {}, hear::<AssistChanged>);
    assert_eq!(told.recv_timeout(WAIT), Ok(on(false)));
    live.bus
        .publish(&AssistToggle {
            which: Toggle::RightClick,
        })
        .unwrap();
    live.bus.publish(&AssistAsk).unwrap();
    assert_eq!(
        told.recv_timeout(WAIT),
        Ok(on(false)),
        "asked after the toggle"
    );
    assert!(live.hook.running(), "the toggle started the hook");
    live.hook.settle(T0);
    assert_eq!(told.recv_timeout(WAIT), Ok(on(true)));
    live.stop();
    assert!(
        !live.hook.running(),
        "the stop joined the thread, whose run let go"
    );
    let off = AssistChanged {
        live: false,
        ..on(true)
    };
    assert_eq!(told.try_recv(), Ok(off));
}

#[test]
fn the_actor_thread_asks_for_latches_and_a_latch_starts_the_hook() {
    let listen = |bus: &Bus| (hear::<AssistChanged>(bus), hear::<LatchesAsk>(bus));
    let (mut live, ((_e1, told), (_e2, asked))) = Live::start(|_| {}, listen);
    assert_eq!(asked.recv_timeout(WAIT), Ok(LatchesAsk));
    let held = vec![SHIFT];
    live.bus.publish(&LatchesChanged { held }).unwrap();
    live.bus.publish(&AssistAsk).unwrap();
    assert_eq!(told.recv_timeout(WAIT), Ok(on(false)));
    assert_eq!(
        told.recv_timeout(WAIT),
        Ok(on(false)),
        "asked after the latch"
    );
    assert!(live.hook.running());
    live.stop();
    assert_eq!(live.hook.stops(), 1);
}

/// Waits until the actor handled every message sent before this ask.
fn synced(live: &Live, told: &Receiver<AssistChanged>) {
    live.bus.publish(&AssistAsk).unwrap();
    assert_eq!(told.recv_timeout(WAIT), Ok(on(false)));
}

#[test]
fn the_actor_thread_asks_for_the_scroll_state_and_scrolling_runs_the_hook() {
    let listen = |bus: &Bus| (hear::<AssistChanged>(bus), hear::<ScrollAsk>(bus));
    let (mut live, ((_e1, told), (_e2, asked))) = Live::start(|_| {}, listen);
    assert_eq!(asked.recv_timeout(WAIT), Ok(ScrollAsk));
    assert_eq!(told.recv_timeout(WAIT), Ok(on(false)));
    let dir = Some(Dir::Down);
    let (fast, still) = (false, None);
    live.bus
        .publish(&ScrollChanged {
            live: true,
            dir,
            fast,
        })
        .unwrap();
    synced(&live, &told);
    assert!(live.hook.running(), "scrolling starts the hook");
    let idle = ScrollChanged {
        live: true,
        dir: still,
        fast,
    };
    live.bus.publish(&idle).unwrap();
    synced(&live, &told);
    live.hook.settle(T0);
    synced(&live, &told);
    assert!(!live.hook.running(), "a still pad lets it go");
    assert!(asked.try_recv().is_err(), "asked once");
    live.stop();
}

#[test]
fn a_saved_right_click_runs_the_hook_from_the_start() {
    let saved = |host: &mut FakeHost| host.put(RCLICK, true.into());
    let (mut live, (_ear, told)) = Live::start(saved, hear::<AssistChanged>);
    assert_eq!(told.recv_timeout(WAIT), Ok(on(true)));
    assert!(live.hook.running());
    live.stop();
    assert!(!live.hook.running());
}

#[test]
fn an_actor_that_panicked_is_logged_once_when_the_module_stops() {
    let lines = lines_of(|| {
        let mut host = host(MANIFEST.capabilities, Some(Arc::new(Broken)));
        host.put(RCLICK, true.into());
        let mut module = MouseModule::default();
        module.start(&mut ModuleCx::new(&mut host)).unwrap();
        host.held.clear();
        module.stop();
    });
    let warned: Vec<_> = lines.iter().filter(|(l, _)| *l == Level::WARN).collect();
    assert_eq!(warned.len(), 1, "{lines:?}");
    let fields: Vec<_> = warned[0].1.keys().copied().collect();
    assert_eq!(fields, [MESSAGE], "the line holds no point or key");
}

#[test]
fn an_actor_that_panics_mid_run_says_at_once_that_the_module_stopped() {
    let mut host = host(MANIFEST.capabilities, Some(Arc::new(Broken)));
    let bus = Bus::new(Arc::clone(&host.core));
    let (_ear, told) = hear::<AssistChanged>(&bus);
    let (_notes, notices) = hear::<Notice>(&bus);
    let mut module = MouseModule::default();
    module.start(&mut ModuleCx::new(&mut host)).unwrap();
    assert_eq!(told.recv_timeout(WAIT), Ok(on(false)), "live");
    let which = Toggle::RightClick;
    bus.publish(&AssistToggle { which }).unwrap();
    let off = Ok(AssistChanged::OFF);
    assert_eq!(
        told.recv_timeout(WAIT),
        off,
        "the keyboard dims its keys now"
    );
    let key = notices.recv_timeout(WAIT).map(|n| n.key);
    assert_eq!(key, Ok(keys::MOUSE_STOPPED), "the user hears why");
    host.held.clear();
    module.stop();
    assert!(told.try_recv().is_err(), "said once");
    assert!(notices.try_recv().is_err(), "told once");
}
