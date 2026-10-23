//! A rig for the actor: the shared fake host's bus, the fake dialog, and every event the actor publishes. Each open
//! runs in place and its answer is handed back at once, so no test needs a thread or a sleep.

use kx_mod_power::{Actor, ID, Job, MANIFEST, Msg, Spawn};
use kx_module_api::ports::{PowerChanged, PowerDialog};
use kx_module_api::{Bus, Event, ModuleCx, Notice, NoticeArg, Subscription, args, keys};
use kx_platform_fake::FakePowerDialog;
use kx_test_support::host::FakeHost;
use std::sync::mpsc::{self, Receiver};
use std::sync::{Arc, Mutex};

/// The module's state while it runs.
pub const LIVE: PowerChanged = PowerChanged { live: true };

/// The module's notice `key`, naming the module.
fn notice(key: &'static str) -> Notice {
    Notice {
        key,
        module: Some(ID),
        args: vec![(args::MODULE, NoticeArg::Text(ID.to_string()))],
    }
}

/// The notice that the dialog did not open.
pub fn failed() -> Notice {
    notice(keys::POWER_FAILED)
}

/// The notice that the module stopped after a panic, until the app restarts.
pub fn stopped() -> Notice {
    notice(keys::POWER_STOPPED)
}

/// One event the actor published.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Heard {
    Changed(PowerChanged),
    Notice(Notice),
}

/// The actor says the module is live.
pub const SAYS_LIVE: Heard = Heard::Changed(LIVE);
/// The actor says the module is off.
pub const SAYS_OFF: Heard = Heard::Changed(PowerChanged::OFF);

/// The actor tells the user that the dialog did not open.
pub fn tells_failed() -> Heard {
    Heard::Notice(failed())
}

/// Runs an open in place, as its own thread would.
pub fn inline(job: Job) -> bool {
    job();
    true
}

/// Refuses every open's thread, as an OS out of threads would.
pub fn refuse(_: Job) -> bool {
    false
}

/// The actor on a fake host, its fake dialog, its inbox for the opens' answers, and what it published.
pub struct Rig {
    pub dialog: FakePowerDialog,
    actor: Actor,
    inbox: Receiver<Msg>,
    heard: Arc<Mutex<Vec<Heard>>>,
    _ears: Vec<Subscription>,
}

impl Rig {
    /// A rig whose opens run in place.
    pub fn new() -> Self {
        Self::with(inline)
    }

    /// A rig whose opens start through `spawn`; it holds every capability the manifest declares.
    pub fn with(spawn: Spawn) -> Self {
        let mut host = FakeHost::new(&MANIFEST);
        host.granted = MANIFEST.capabilities.to_vec();
        let dialog = FakePowerDialog::new();
        let shared: Arc<dyn PowerDialog> = Arc::new(dialog.clone());
        let (back, inbox) = mpsc::channel();
        let bus = ModuleCx::new(&mut host).bus();
        let actor = Actor::new(shared, bus, back, spawn);
        let heard = Arc::default();
        let ears = listen(&Bus::new(Arc::clone(&host.core)), &heard);
        Self {
            dialog,
            actor,
            inbox,
            heard,
            _ears: ears,
        }
    }

    /// Starts the actor as its thread would.
    pub fn begin(&self) {
        self.actor.begin();
    }

    /// Hands `msg` to the actor, then each answer its opens sent back, as its thread would; false once it is done.
    pub fn send(&self, msg: Msg) -> bool {
        let alive = self.actor.handle(msg);
        while let Ok(answer) = self.inbox.try_recv() {
            self.actor.handle(answer);
        }
        alive
    }

    /// What the actor published since the last call.
    pub fn take(&self) -> Vec<Heard> {
        std::mem::take(&mut *self.heard.lock().unwrap())
    }
}

/// Hears every event the actor publishes.
fn listen(bus: &Bus, heard: &Arc<Mutex<Vec<Heard>>>) -> Vec<Subscription> {
    vec![
        ear(bus, heard, |e: &PowerChanged| Heard::Changed(*e)),
        ear(bus, heard, |e: &Notice| Heard::Notice(e.clone())),
    ]
}

/// Keeps each `E` in `heard` as `as_heard` says.
fn ear<E: Event>(
    bus: &Bus,
    heard: &Arc<Mutex<Vec<Heard>>>,
    as_heard: fn(&E) -> Heard,
) -> Subscription {
    let heard = Arc::clone(heard);
    let keep = move |e: &E| heard.lock().unwrap().push(as_heard(e));
    bus.subscribe(keep).unwrap()
}
