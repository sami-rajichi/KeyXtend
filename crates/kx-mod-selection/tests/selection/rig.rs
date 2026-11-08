//! What the module tests share: the fakes behind its two services, a host holding them, an ear on the bus, and a
//! keyboard that answers each edit from inside its handler, as the real one does.

use std::sync::Arc;
use std::sync::mpsc::{self, Receiver, Sender};
use std::time::Duration;

use kx_mod_selection::{MANIFEST, SelectionModule};
use kx_module_api::ports::{
    ChipsView, EditAsk, EditDone, Piece, PressesWanted, Rect, Screen, SelKind, SelSpot,
    SelectionWatch, SelectionWatchKey, Typed, WebLink, WebLinkKey,
};
use kx_module_api::{
    Bus, Capability, Event, Module, ModuleCx, ModuleError, Notice, ServiceKey, Subscription,
};
use kx_platform_fake::{FakeSelectionWatch, FakeWebLink};
use kx_test_support::host::{FakeHost, Stored};

/// The longest a test waits for the module's thread; a pass takes milliseconds.
pub const WAIT: Duration = Duration::from_secs(10);
/// The system's display language the tests build the module with.
pub const OS: &str = "en-GB";

/// The fakes behind the module's services.
#[derive(Clone)]
pub struct Fakes {
    pub watch: FakeSelectionWatch,
    pub web: Arc<FakeWebLink>,
}

impl Fakes {
    /// A watch on no box and a link with no call.
    pub fn new() -> Self {
        Self {
            watch: FakeSelectionWatch::new(),
            web: Arc::new(FakeWebLink::new()),
        }
    }

    /// A host granting `granted` and holding the fakes.
    pub fn host(&self, granted: &[Capability]) -> FakeHost {
        host_of(Arc::new(self.watch.clone()), self.web.clone(), granted)
    }
}

/// A host granting `granted` and holding `watch` and `web`.
pub fn host_of(
    watch: Arc<dyn SelectionWatch>,
    web: Arc<dyn WebLink>,
    granted: &[Capability],
) -> FakeHost {
    let mut host = FakeHost::new(&MANIFEST);
    host.granted = granted.to_vec();
    let stored: [(_, Stored); 2] = [
        (
            SelectionWatchKey::ID,
            (SelectionWatchKey::CAPABILITY, Box::new(watch)),
        ),
        (WebLinkKey::ID, (WebLinkKey::CAPABILITY, Box::new(web))),
    ];
    host.services.extend(stored);
    host
}

/// A module built with `OS`.
pub fn module() -> SelectionModule {
    SelectionModule::new(OS)
}

/// Starts `module` on `host`.
pub fn start(module: &mut impl Module, host: &mut FakeHost) -> Result<(), ModuleError> {
    module.start(&mut ModuleCx::new(host))
}

/// Stops `module` as the kernel does: its handlers leave the bus first.
pub fn stop(module: &mut impl Module, host: &mut FakeHost) {
    host.held.clear();
    module.stop();
}

/// One event the module published, as a test hears it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Said {
    View(ChipsView),
    Presses(bool),
    Edit(EditAsk),
    Notice(&'static str),
}

/// What the module publishes, heard on the host's bus from any thread.
pub struct Heard {
    said: Receiver<Said>,
    _ears: Vec<Subscription>,
}

impl Heard {
    /// Listens on `host`'s bus from now on.
    pub fn listen(host: &FakeHost) -> Self {
        let (tx, said) = mpsc::channel();
        let bus = Bus::new(Arc::clone(&host.core));
        let ears = vec![
            ear(&bus, &tx, |e: &ChipsView| Said::View(e.clone())),
            ear(&bus, &tx, |e: &PressesWanted| Said::Presses(e.on)),
            ear(&bus, &tx, |e: &EditAsk| Said::Edit(*e)),
            ear(&bus, &tx, |e: &Notice| Said::Notice(e.key)),
        ];
        Self { said, _ears: ears }
    }

    /// Waits up to `WAIT` for the next event `pick` takes, passing over the others; fails the test after.
    pub fn wait<T>(&self, pick: impl Fn(Said) -> Option<T>) -> T {
        loop {
            let said = self.said.recv_timeout(WAIT).unwrap();
            if let Some(t) = pick(said) {
                return t;
            }
        }
    }

    /// The round of the next strip shown.
    pub fn shown(&self) -> u32 {
        self.wait(|s| match s {
            Said::View(ChipsView::Shown { round, .. }) => Some(round),
            _ => None,
        })
    }

    /// Every event heard and not yet taken, without waiting.
    pub fn drain(&self) -> Vec<Said> {
        self.said.try_iter().collect()
    }
}

/// Hands each `E` on `bus` to `tx` as `make` says.
fn ear<E: Event>(
    bus: &Bus,
    tx: &Sender<Said>,
    make: impl Fn(&E) -> Said + Send + Sync + 'static,
) -> Subscription {
    let tx = tx.clone();
    bus.subscribe(move |e: &E| drop(tx.send(make(e)))).unwrap()
}

/// A keyboard that sends each edit asked, answers yes, then tells the break, from inside its handler as the real one.
pub fn keyboard(host: &FakeHost) -> Subscription {
    let bus = Bus::new(Arc::clone(&host.core));
    let answer = bus.clone();
    let edit = move |ask: &EditAsk| {
        let done = EditDone {
            round: ask.round,
            ok: true,
        };
        answer.publish(&done).unwrap();
        answer
            .publish(&Typed {
                piece: Piece::Break,
            })
            .unwrap();
    };
    bus.subscribe(edit).unwrap()
}

/// A box of `kind` with a selection on one line; the fake gives it its id.
pub fn sel(kind: SelKind) -> SelSpot {
    let line = Rect {
        x: 100,
        y: 40,
        w: 200,
        h: 20,
    };
    let rect = Rect {
        x: 0,
        y: 0,
        w: 1920,
        h: 1040,
    };
    SelSpot {
        box_id: 0,
        kind,
        selected: true,
        first: Some(line),
        last: Some(line),
        caret: None,
        screen: Screen {
            rect,
            dpi: 96,
            primary: true,
        },
    }
}
