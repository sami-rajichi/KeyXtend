//! What the module tests share: the five fakes, a host holding them, a list saved as a run leaves it, an ear on the
//! bus, a fake keyboard and the machine's time.

use std::sync::Arc;
use std::sync::mpsc::{self, Receiver, Sender};
use std::time::{Duration, Instant};

use kx_mod_clipboard::entry::Entry;
use kx_mod_clipboard::order::History;
use kx_mod_clipboard::store::Store;
use kx_mod_clipboard::{ClipSettings, MANIFEST};
use kx_module_api::ports::{
    Calendar, CalendarKey, ClipPanel, ClipPart, ClipView, Clipboard, ClipboardKey, ClipboardWriter,
    ClipboardWriterKey, Date, HistoryWanted, PasteAsk, Pasted, Pictures, PicturesKey, SealedShelf,
    SealedShelfKey,
};
use kx_module_api::{
    Bus, Capability, Clock, Event, Module, ModuleCx, ModuleError, Mono, Notice, Redacted,
    ServiceKey, Subscription,
};
use kx_platform_fake::{FakeCalendar, FakeClipboard, FakeClipboardWriter, FakePictures, FakeShelf};
use kx_test_support::host::{FakeHost, Stored};

/// Paris in winter, in minutes east of UTC.
const PARIS: i16 = 60;
/// The longest a test waits for the module's thread; a pass takes milliseconds.
pub const WAIT: Duration = Duration::from_secs(10);
/// The fakes' day, as its folder is named.
pub const TODAY: &str = "2026-10-31";
/// A folder list's head blob.
pub const LIST: &str = "list";

/// The fakes behind the module's services.
#[derive(Clone)]
pub struct Fakes {
    pub clip: FakeClipboard,
    pub writer: FakeClipboardWriter,
    pub shelf: FakeShelf,
    pub pics: FakePictures,
    pub calendar: FakeCalendar,
}

impl Fakes {
    /// Fresh fakes on 31 October 2026 at 16:32 in Paris.
    pub fn new() -> Self {
        let day = Date::new(2026, 10, 31).unwrap();
        Self {
            clip: FakeClipboard::new(),
            writer: FakeClipboardWriter::new(),
            shelf: FakeShelf::new(),
            pics: FakePictures::new(),
            calendar: FakeCalendar::new(day, 16, 32, PARIS),
        }
    }

    /// A host granting `granted` and holding the fakes.
    pub fn host(&self, granted: &[Capability]) -> FakeHost {
        let mut host = FakeHost::new(&MANIFEST);
        host.granted = granted.to_vec();
        let clip: Arc<dyn Clipboard> = Arc::new(self.clip.clone());
        let writer: Arc<dyn ClipboardWriter> = Arc::new(self.writer.clone());
        let sealed: Arc<dyn SealedShelf> = Arc::new(self.shelf.clone());
        let pics: Arc<dyn Pictures> = Arc::new(self.pics.clone());
        let calendar: Arc<dyn Calendar> = Arc::new(self.calendar.clone());
        let stored: [(_, Stored); 5] = [
            (ClipboardKey::ID, (ClipboardKey::CAPABILITY, Box::new(clip))),
            (
                ClipboardWriterKey::ID,
                (ClipboardWriterKey::CAPABILITY, Box::new(writer)),
            ),
            (
                SealedShelfKey::ID,
                (SealedShelfKey::CAPABILITY, Box::new(sealed)),
            ),
            (PicturesKey::ID, (PicturesKey::CAPABILITY, Box::new(pics))),
            (
                CalendarKey::ID,
                (CalendarKey::CAPABILITY, Box::new(calendar)),
            ),
        ];
        host.services.extend(stored);
        host
    }

    /// Saves one text entry on the shelf, as an earlier run leaves it.
    pub fn saved(&self, text: &str) {
        let settings = ClipSettings::from_defaults().unwrap();
        let mut store = Store::new(Arc::new(self.shelf.clone()), settings.store_config());
        let parts = [ClipPart::Text(Redacted::new(text.to_owned()))];
        let mut history = History::default();
        let entry = Entry::new(&parts, None, &self.calendar.now(), &settings.entry_config());
        let id = history.add(entry);
        let n = store.put(history.get(id).unwrap(), &parts).unwrap();
        history.update(id, |e| e.parts = n);
        store.save(&history).unwrap();
    }
}

/// Starts `module` on `host`.
pub fn start(module: &mut impl Module, host: &mut FakeHost) -> Result<(), ModuleError> {
    module.start(&mut ModuleCx::new(host))
}

/// One event the module published, as a test hears it.
#[derive(Clone, Debug)]
pub enum Said {
    View(ClipView),
    Panel(bool),
    Wanted(bool),
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
            ear(&bus, &tx, |e: &ClipView| Said::View(e.clone())),
            ear(&bus, &tx, |e: &ClipPanel| Said::Panel(e.open)),
            ear(&bus, &tx, |e: &HistoryWanted| Said::Wanted(e.on)),
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

    /// Every event heard and not yet taken, without waiting.
    pub fn drain(&self) -> Vec<Said> {
        self.said.try_iter().collect()
    }

    /// Every notice key heard and not yet taken.
    pub fn notices(&self) -> Vec<&'static str> {
        let all = self.drain().into_iter();
        all.filter_map(|s| {
            if let Said::Notice(k) = s {
                Some(k)
            } else {
                None
            }
        })
        .collect()
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

/// A keyboard that answers each paste request with a paste that went out, from inside its handler as the real one.
pub fn keyboard(host: &FakeHost) -> Subscription {
    let bus = Bus::new(Arc::clone(&host.core));
    let answer = bus.clone();
    let paste = move |ask: &PasteAsk| {
        let pasted = Pasted {
            round: ask.round,
            ok: true,
        };
        answer.publish(&pasted).unwrap();
    };
    bus.subscribe(paste).unwrap()
}

/// The machine's monotonic time since the test began, for a module whose timers run on its own thread.
struct Wall(Instant);

impl Clock for Wall {
    fn now(&self) -> Mono {
        Mono::from_us(u64::try_from(self.0.elapsed().as_micros()).unwrap_or(u64::MAX))
    }
}

/// Gives `host` the machine's time, so the module's timers pass while the test waits for its events.
pub fn wall(host: &mut FakeHost) {
    host.clock = Arc::new(Wall(Instant::now()));
}
