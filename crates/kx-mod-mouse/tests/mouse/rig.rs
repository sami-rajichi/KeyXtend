//! A rig for the actor: the shared fake host's bus and settings sink, the fake hook, and every event the actor
//! publishes. The test thread pumps the hook engine's messages into the actor, so no test needs a thread.

use kx_mod_mouse::{Actor, ID, MANIFEST, MouseSettings, Msg};
use kx_module_api::ports::{
    AssistChanged, AssistNote, Button, GrabState, LatchesAsk, LatchesUsed, Point, PointerEvent,
    PointerHook, PointerMoved, Toggle,
};
use kx_module_api::{
    Bus, Event, ModuleCx, Mono, Notice, NoticeArg, Subscription, US_PER_MS, args, parse_as,
};
use kx_platform_fake::FakePointerHook;
use kx_test_support::host::FakeHost;
use std::sync::mpsc::{self, Receiver};
use std::sync::{Arc, Mutex};

/// The settings key of Right-click mode.
pub const RCLICK: &str = "rclick";
/// A screen at 100 % scale.
pub const DPI: u32 = 96;
/// Times the tests feed at, in order.
pub const T0: Mono = Mono::from_us(1_000);
pub const T1: Mono = Mono::from_us(2_000);
pub const T2: Mono = Mono::from_us(3_000);
/// Points far apart, so a move between them is never still.
pub const A: Point = Point { x: 100, y: 100 };
pub const B: Point = Point { x: 400, y: 300 };
pub const C: Point = Point { x: 700, y: 500 };

/// One event the actor published.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Heard {
    Changed(AssistChanged),
    Note(AssistNote),
    Moved(PointerMoved),
    Used,
    AskLatches,
    Notice(Notice),
}

/// The live state the keyboard hears.
pub fn shown(rclick: bool, grab: GrabState) -> Heard {
    let live = true;
    Heard::Changed(AssistChanged { live, rclick, grab })
}

/// The mouse module's notice `key`.
pub fn told(key: &'static str) -> Heard {
    Heard::Notice(Notice {
        key,
        module: Some(ID),
        args: vec![(args::MODULE, NoticeArg::Text(ID.to_string()))],
    })
}

/// A left press outside our windows at `at`.
pub fn down(at: Point) -> PointerEvent {
    let button = Button::Left;
    PointerEvent::Down {
        button,
        at,
        ours: false,
        dpi: DPI,
    }
}

/// A left release at `at`.
pub fn up(at: Point) -> PointerEvent {
    let button = Button::Left;
    PointerEvent::Up { button, at }
}

/// A move to `at`.
pub fn moved(at: Point) -> PointerEvent {
    PointerEvent::Move { at }
}

/// When a hold pressed at `t0` fires, by `settings`.
pub fn due(settings: &MouseSettings, t0: Mono) -> Mono {
    let hold = u64::from(settings.hold_ms) * u64::from(US_PER_MS);
    Mono::from_us(t0.as_us() + hold)
}

/// The actor on a fake host, its fake hook, and what it published.
pub struct Rig {
    pub host: FakeHost,
    pub hook: Arc<FakePointerHook>,
    pub actor: Actor,
    pub settings: MouseSettings,
    /// The actor said it is done, as its thread would end.
    pub ended: bool,
    inbox: Receiver<Msg>,
    heard: Arc<Mutex<Vec<Heard>>>,
    _ears: Vec<Subscription>,
}

impl Rig {
    /// A rig on the defaults.
    pub fn new() -> Self {
        Self::with(|_| {})
    }

    /// A rig whose settings have Right-click saved on.
    pub fn saved_rclick() -> Self {
        Self::with(|host| host.put(RCLICK, true.into()))
    }

    /// A rig whose host `setup` changes first; it holds every capability the manifest declares.
    pub fn with(setup: impl FnOnce(&mut FakeHost)) -> Self {
        let mut host = FakeHost::new(&MANIFEST);
        host.granted = MANIFEST.capabilities.to_vec();
        setup(&mut host);
        let settings: MouseSettings = parse_as(&host.settings).unwrap();
        let hook = Arc::new(FakePointerHook::new());
        let shared: Arc<dyn PointerHook> = hook.clone();
        let (tx, inbox) = mpsc::channel();
        let cx = ModuleCx::new(&mut host);
        let (bus, store) = (cx.bus(), cx.settings_handle());
        let actor = Actor::new(shared, bus, store, settings.clone(), tx);
        let heard = Arc::default();
        let ears = listen(&Bus::new(Arc::clone(&host.core)), &heard);
        Self {
            host,
            hook,
            actor,
            settings,
            ended: false,
            inbox,
            heard,
            _ears: ears,
        }
    }

    /// Starts the actor as its thread would.
    pub fn begin(&mut self) {
        self.actor.begin();
        self.pump();
    }

    /// Hands `msg` to the actor, then pumps; false once the actor is done.
    pub fn send(&mut self, msg: Msg) -> bool {
        let going = self.actor.handle(msg);
        self.pump();
        going
    }

    /// The keyboard flips `which`.
    pub fn toggle(&mut self, which: Toggle) {
        self.send(Msg::Toggle(which));
    }

    /// Feeds `event` to the live hook at `now`, then pumps; true when it was dropped.
    pub fn feed(&mut self, event: PointerEvent, now: Mono) -> bool {
        let dropped = self.hook.feed(event, now);
        self.pump();
        dropped
    }

    /// Settles the live hook at `now`, as a wake or its deadline would, then pumps.
    pub fn settle(&mut self, now: Mono) {
        if self.hook.running() {
            self.hook.settle(now);
        }
        self.pump();
    }

    /// Hands the hook engine's messages to the actor until none wait.
    pub fn pump(&mut self) {
        while let Ok(msg) = self.inbox.try_recv() {
            self.ended |= !self.actor.handle(msg);
        }
    }

    /// What the actor published since the last call.
    pub fn take(&self) -> Vec<Heard> {
        std::mem::take(&mut *self.heard.lock().unwrap())
    }
}

/// Hears every event the actor publishes.
fn listen(bus: &Bus, heard: &Arc<Mutex<Vec<Heard>>>) -> Vec<Subscription> {
    vec![
        ear(bus, heard, |e: &AssistChanged| Heard::Changed(*e)),
        ear(bus, heard, |e: &AssistNote| Heard::Note(*e)),
        ear(bus, heard, |e: &PointerMoved| Heard::Moved(*e)),
        ear(bus, heard, |_: &LatchesUsed| Heard::Used),
        ear(bus, heard, |_: &LatchesAsk| Heard::AskLatches),
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
