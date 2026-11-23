//! What the module tests share: the fakes behind its services, the cloud port among them, a host holding them, an
//! ear on the bus that hears from any thread, and a models folder holding English's default model.

use std::path::Path;
use std::sync::Arc;
use std::sync::mpsc::{self, Receiver, Sender};
use std::time::Duration;

use kx_mod_voice::models::table;
use kx_mod_voice::{MANIFEST, VoiceModule};
use kx_module_api::ports::{
    CalendarKey, ClipboardWriterKey, CloudButton, CloudSpeechKey, Date, Drive, FieldWatchKey,
    FolderRevealKey, MB, ModelFiles, ModelFilesKey, PanelLine, PressesWanted, SessionWatchKey,
    SpeechKey, VoiceAct, VoiceClick, VoiceLine, VoiceLit, VoiceOpen, VoicePage, VoiceSheet,
    VoiceShelfKey, VoiceTab, VoiceView, WebLinkKey,
};
use kx_module_api::{
    Bus, Event, Module, ModuleCx, ModuleError, Mono, Notice, Redacted, ServiceId, ServiceKey,
    Subscription,
};
use kx_platform_fake::{
    FakeCalendar, FakeClipboardWriter, FakeClock, FakeCloud, FakeFieldWatch, FakeFolderReveal,
    FakeModelFiles, FakeSessionWatch, FakeShelf, FakeSpeech, FakeWebLink,
};
use kx_test_support::host::{FakeHost, Stored};

/// The longest a test waits for the module's threads; a pass takes milliseconds.
pub const WAIT: Duration = Duration::from_secs(10);
/// The models folder the tests use.
pub const DIR: &str = r"D:\KeyXtend\models";

/// The fakes behind the module's services.
#[derive(Clone)]
pub struct Fakes {
    pub speech: FakeSpeech,
    pub files: FakeModelFiles,
    pub session: FakeSessionWatch,
    pub field: FakeFieldWatch,
    pub clip: Arc<FakeClipboardWriter>,
    pub reveal: Arc<FakeFolderReveal>,
    pub shelf: FakeShelf,
    pub calendar: FakeCalendar,
    pub link: Arc<FakeWebLink>,
    pub cloud: FakeCloud,
    pub clock: FakeClock,
}

impl Fakes {
    /// Drive D: holding the models folder with English's default model.
    pub fn new() -> Self {
        let fakes = Self {
            speech: FakeSpeech::new(),
            files: FakeModelFiles::new(),
            session: FakeSessionWatch::new(),
            field: FakeFieldWatch::new(),
            clip: Arc::new(FakeClipboardWriter::new()),
            reveal: Arc::new(FakeFolderReveal::new()),
            shelf: FakeShelf::new(),
            calendar: FakeCalendar::new(Date::new(2026, 11, 15).unwrap(), 12, 0, 60),
            link: Arc::new(FakeWebLink::new()),
            cloud: FakeCloud::new(),
            clock: FakeClock::new(Mono::from_us(1_000)),
        };
        fakes.files.set_drives(vec![Drive {
            root: Redacted::new(r"D:\".into()),
            name: Redacted::new("D:".into()),
            free: 900_000 * MB,
        }]);
        let row = table().unwrap().default_for("en").unwrap();
        let file = Path::new(DIR).join(&row.file);
        fakes.files.add_file(&file, row.bytes, row.print);
        fakes
    }

    /// A host granting every capability the manifest declares, holding the fakes, with the models folder set.
    pub fn host(&self) -> FakeHost {
        self.host_with(Arc::new(self.files.clone()))
    }

    /// As `host`, with `files` behind the model files service.
    pub fn host_with(&self, files: Arc<dyn ModelFiles>) -> FakeHost {
        let mut host = FakeHost::new(&MANIFEST);
        host.granted = MANIFEST.capabilities.to_vec();
        host.put("models_dir", DIR.into());
        host.clock = Arc::new(self.clock.clone());
        host.services.extend([
            stored::<SpeechKey>(Arc::new(self.speech.clone())),
            stored::<ModelFilesKey>(files),
            stored::<SessionWatchKey>(Arc::new(self.session.clone())),
            stored::<FieldWatchKey>(Arc::new(self.field.clone())),
            stored::<ClipboardWriterKey>(self.clip.clone()),
            stored::<FolderRevealKey>(self.reveal.clone()),
            stored::<VoiceShelfKey>(Arc::new(self.shelf.clone())),
            stored::<CalendarKey>(Arc::new(self.calendar.clone())),
            stored::<WebLinkKey>(self.link.clone()),
            stored::<CloudSpeechKey>(Arc::new(self.cloud.clone())),
        ]);
        host
    }
}

/// `api` as the host stores service `K`, with its gate.
fn stored<K: ServiceKey>(api: Arc<K::Api>) -> (ServiceId, Stored) {
    (K::ID, (K::CAPABILITY, Box::new(api)))
}

/// Starts `module` on `host`.
pub fn start(module: &mut VoiceModule, host: &mut FakeHost) -> Result<(), ModuleError> {
    module.start(&mut ModuleCx::new(host))
}

/// Stops `module` as the kernel does: its handlers leave the bus first.
pub fn stop(module: &mut VoiceModule, host: &mut FakeHost) {
    host.held.clear();
    module.stop();
}

/// One event the module published, as a test hears it.
#[derive(Clone, Debug, PartialEq, Eq)]
#[allow(
    clippy::large_enum_variant,
    reason = "a test keeps a few views, so their size does not matter"
)]
pub enum Said {
    View(VoicePage),
    Lit(VoiceLit),
    Presses(bool),
    Notice(&'static str),
    /// The panel's line, for the keyboard.
    Line(Option<PanelLine>),
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
            ear(&bus, &tx, |e: &VoiceView| Said::View((*e.0).clone())),
            ear(&bus, &tx, |e: &VoiceLit| Said::Lit(*e)),
            ear(&bus, &tx, |e: &PressesWanted| Said::Presses(e.on)),
            ear(&bus, &tx, |e: &Notice| Said::Notice(e.key)),
            ear(&bus, &tx, |e: &VoiceLine| Said::Line(e.0.clone())),
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

    /// Waits for `want`, passing over the others.
    pub fn wait_for(&self, want: &Said) {
        self.wait(|s| (s == *want).then_some(()));
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

/// Clicks `click` on the view of round `round`, then waits for a view whose page `shows`; its round.
pub fn click(
    bus: &Bus,
    heard: &Heard,
    round: u32,
    click: VoiceClick,
    shows: fn(&VoiceSheet) -> bool,
) -> u32 {
    bus.publish(&VoiceAct { round, click }).unwrap();
    heard.wait(|s| match s {
        Said::View(page) => page
            .panel
            .as_ref()
            .is_some_and(|p| shows(&p.sheet))
            .then_some(page.round),
        _ => None,
    })
}

/// Opens the panel on the Cloud page with its first service ticked; the round of that view.
pub fn first_service(bus: &Bus, heard: &Heard) -> u32 {
    bus.publish(&VoiceOpen).unwrap();
    let round = heard.wait(|s| match s {
        Said::View(page) if page.panel.is_some() => Some(page.round),
        _ => None,
    });
    let cloud = |s: &VoiceSheet| matches!(s, VoiceSheet::Cloud(_));
    let round = click(bus, heard, round, VoiceClick::Open(VoiceTab::Cloud), cloud);
    let ticked =
        |s: &VoiceSheet| matches!(s, VoiceSheet::Cloud(c) if c.acts.contains(&CloudButton::Facts));
    click(bus, heard, round, VoiceClick::Row(0), ticked)
}
