//! A rig for the actor: the shared fake host's bus, the fake target and clock, and every event the actor publishes.
//! The test thread hands each message and tick to the actor at a time it sets, so no test needs a thread or a sleep.

use kx_mod_scroll::{Actor, ID, MANIFEST, Msg, ScrollSettings};
use kx_module_api::ports::{
    Aim, Dir, Point, Rect, ScrollAimed, ScrollChanged, ScrollPage, ScrollTarget, TargetId,
};
use kx_module_api::{
    Bus, Clock, Event, ModuleCx, Mono, Notice, NoticeArg, Subscription, US_PER_MS, args, parse_as,
};
use kx_platform_fake::{FakeClock, FakeScrollTarget, Turn};
use kx_test_support::host::FakeHost;
use std::sync::{Arc, Mutex};
use std::time::Duration;

/// When each test starts.
pub const T0: Mono = Mono::from_us(1_000_000);
/// How close two line counts must be: far below any wheel step.
pub const EPS: f64 = 1e-9;

/// An app's pane, where the user pointed.
pub const APP: Aim = Aim {
    at: Point { x: 123, y: 345 },
    on_pointer: false,
    area: Some(Rect {
        x: 101,
        y: 202,
        w: 640,
        h: 480,
    }),
    ours: false,
    id: TargetId::new(7),
};

/// Another app's pane, for a retarget.
pub const OTHER: Aim = Aim {
    at: Point { x: 987, y: 654 },
    on_pointer: true,
    area: Some(Rect {
        x: 903,
        y: 57,
        w: 310,
        h: 720,
    }),
    ours: false,
    id: TargetId::new(9),
};

/// One of our own open lists.
pub const LIST: Aim = Aim {
    at: Point { x: 41, y: 612 },
    on_pointer: false,
    area: Some(Rect {
        x: 33,
        y: 604,
        w: 206,
        h: 151,
    }),
    ours: true,
    id: TargetId::new(11),
};

/// One event the actor published.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Heard {
    Changed(ScrollChanged),
    Aimed(ScrollAimed),
    Page(ScrollPage),
    Notice(Notice),
}

/// The live state: scrolling `dir` at the fast speed or not, or still.
pub fn state(dir: Option<Dir>, fast: bool) -> Heard {
    Heard::Changed(ScrollChanged {
        live: true,
        dir,
        fast,
    })
}

/// The live state while still.
pub fn still() -> Heard {
    state(None, false)
}

/// The outline on `aim`'s box.
pub fn outline(aim: Aim) -> Heard {
    Heard::Aimed(ScrollAimed { area: aim.area })
}

/// The outline hidden.
pub fn hidden() -> Heard {
    Heard::Aimed(ScrollAimed { area: None })
}

/// The scroll module's notice `key`.
pub fn told(key: &'static str) -> Heard {
    Heard::Notice(Notice {
        key,
        module: Some(ID),
        args: vec![(args::MODULE, NoticeArg::Text(ID.to_string()))],
    })
}

/// A turn of `aim` towards `dir` by `lines`.
pub fn turn(aim: Aim, dir: Dir, lines: f64) -> Turn {
    Turn {
        id: aim.id,
        dir,
        lines,
    }
}

/// Whether `got` turns are `want`, with each count within `EPS`.
pub fn same(got: &[Turn], want: &[Turn]) -> bool {
    let close =
        |(g, w): (&Turn, &Turn)| g.id == w.id && g.dir == w.dir && (g.lines - w.lines).abs() < EPS;
    got.len() == want.len() && got.iter().zip(want).all(close)
}

/// The actor on a fake host, its fake target and clock, and what it published.
pub struct Rig {
    pub target: FakeScrollTarget,
    pub clock: FakeClock,
    pub actor: Actor,
    pub settings: ScrollSettings,
    heard: Arc<Mutex<Vec<Heard>>>,
    _ears: Vec<Subscription>,
}

impl Rig {
    /// A rig on the defaults.
    pub fn new() -> Self {
        Self::with(|_| {})
    }

    /// A rig whose host `setup` changes first; it holds every capability the manifest declares.
    pub fn with(setup: impl FnOnce(&mut FakeHost)) -> Self {
        let mut host = FakeHost::new(&MANIFEST);
        host.granted = MANIFEST.capabilities.to_vec();
        setup(&mut host);
        let settings: ScrollSettings = parse_as(&host.settings).unwrap();
        let (target, clock) = (FakeScrollTarget::new(), FakeClock::new(T0));
        let shared: Arc<dyn ScrollTarget> = Arc::new(target.clone());
        let ticks: Arc<dyn Clock> = Arc::new(clock.clone());
        let bus = ModuleCx::new(&mut host).bus();
        let actor = Actor::new(shared, bus, ticks, &settings);
        let heard = Arc::default();
        let ears = listen(&Bus::new(Arc::clone(&host.core)), &heard);
        Self {
            target,
            clock,
            actor,
            settings,
            heard,
            _ears: ears,
        }
    }

    /// Starts the actor as its thread would.
    pub fn begin(&mut self) {
        self.actor.begin();
    }

    /// Hands `msg` to the actor at the clock's time; false once the actor is done.
    pub fn send(&mut self, msg: Msg) -> bool {
        self.actor.handle(msg)
    }

    /// Taps `dir` at `at`.
    pub fn tap_at(&mut self, dir: Dir, at: Mono) {
        self.clock.set(at);
        self.send(Msg::Tap(dir));
    }

    /// Ticks at `at`, as the thread's wait would.
    pub fn tick_at(&mut self, at: Mono) {
        self.clock.set(at);
        self.actor.tick();
    }

    /// The pointer is now on `aim`'s target.
    pub fn point(&self, aim: Aim) {
        self.target.push_aim(Ok(aim));
    }

    /// Scrolls `dir` slow at `APP` from `T0`, then forgets what it published.
    pub fn scrolling(&mut self, dir: Dir) {
        self.begin();
        self.point(APP);
        self.tap_at(dir, T0);
        self.take();
    }

    /// `n` ticks after `T0`.
    pub fn ticks(&self, n: u64) -> Mono {
        after(n * u64::from(self.settings.tick_ms))
    }

    /// `n` checks after `T0`.
    pub fn checks(&self, n: u64) -> Mono {
        after(n * u64::from(self.settings.check_ms))
    }

    /// The lines owed over `ms` at the slow speed or the fast one.
    pub fn owed(&self, fast: bool, ms: u64) -> f64 {
        let speed = if fast {
            self.settings.fast
        } else {
            self.settings.slow
        };
        f64::from(speed) * Duration::from_millis(ms).as_secs_f64()
    }

    /// What the actor published since the last call.
    pub fn take(&self) -> Vec<Heard> {
        std::mem::take(&mut *self.heard.lock().unwrap())
    }
}

/// `ms` after `T0`.
pub fn after(ms: u64) -> Mono {
    Mono::from_us(T0.as_us() + ms * u64::from(US_PER_MS))
}

/// Hears every event the actor publishes.
fn listen(bus: &Bus, heard: &Arc<Mutex<Vec<Heard>>>) -> Vec<Subscription> {
    vec![
        ear(bus, heard, |e: &ScrollChanged| Heard::Changed(*e)),
        ear(bus, heard, |e: &ScrollAimed| Heard::Aimed(*e)),
        ear(bus, heard, |e: &ScrollPage| Heard::Page(*e)),
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
