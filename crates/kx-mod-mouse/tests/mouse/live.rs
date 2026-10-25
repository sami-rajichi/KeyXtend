//! A running mouse module on the shared fake host and a fake hook, and the waits its threaded tests use.

use kx_mod_mouse::{MANIFEST, MouseModule};
use kx_module_api::ports::{AssistAsk, AssistChanged, GrabState, PointerHook, PointerHookKey};
use kx_module_api::{Bus, Capability, Event, Module, ModuleCx, ServiceKey, Subscription};
use kx_platform_fake::FakePointerHook;
use kx_test_support::host::{FakeHost, Stored};
use std::sync::Arc;
use std::sync::mpsc::{self, Receiver};
use std::time::Duration;

/// The longest a threaded test waits for the actor; it never sleeps.
pub const WAIT: Duration = Duration::from_secs(10);

/// A host on the defaults granting `granted`, holding `hook` if any.
pub fn host(granted: &[Capability], hook: Option<Arc<dyn PointerHook>>) -> FakeHost {
    let mut host = FakeHost::new(&MANIFEST);
    host.granted = granted.to_vec();
    if let Some(hook) = hook {
        let stored: Stored = (PointerHookKey::CAPABILITY, Box::new(hook));
        host.services.insert(PointerHookKey::ID, stored);
    }
    host
}

/// A running module on a fake hook, and an unlimited bus to talk to it.
pub struct Live {
    host: FakeHost,
    pub hook: Arc<FakePointerHook>,
    module: MouseModule,
    pub bus: Bus,
}

impl Live {
    /// Starts the module on a host that `setup` changes, after `listen` subscribes on its bus.
    pub fn start<T>(
        setup: impl FnOnce(&mut FakeHost),
        listen: impl FnOnce(&Bus) -> T,
    ) -> (Self, T) {
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

    /// Starts the stopped module again on the same host.
    pub fn restart(&mut self) {
        self.module
            .start(&mut ModuleCx::new(&mut self.host))
            .unwrap();
    }

    /// Stops the module as the kernel does: its handlers leave the bus first.
    pub fn stop(&mut self) {
        self.host.held.clear();
        self.module.stop();
    }
}

/// Sends every `E` published on `bus` to the returned channel.
pub fn hear<E: Event + Clone>(bus: &Bus) -> (Subscription, Receiver<E>) {
    let (tx, rx) = mpsc::channel();
    let sub = bus
        .subscribe(move |e: &E| {
            let _ = tx.send(e.clone());
        })
        .unwrap();
    (sub, rx)
}

/// The live state with `rclick` and Grab off.
pub fn on(rclick: bool) -> AssistChanged {
    let grab = GrabState::Off;
    AssistChanged {
        live: true,
        rclick,
        grab,
    }
}

/// Waits until the actor handled every message sent before this ask.
pub fn synced(live: &Live, told: &Receiver<AssistChanged>) {
    live.bus.publish(&AssistAsk).unwrap();
    assert_eq!(told.recv_timeout(WAIT), Ok(on(false)));
}
