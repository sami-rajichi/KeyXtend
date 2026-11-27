//! The module on the fake host: it starts its actor on its services, a Snip key lights the key and freezes the
//! screens, an ask tells the light and line again, the stop ends the snip, and a panic on the actor's thread ends it
//! too and tells the user; a missing service or a refused section fails the start.

use std::path::PathBuf;
use std::sync::Arc;
use std::sync::mpsc::{self, Receiver};
use std::time::Duration;

use kx_mod_snip::{ID, MANIFEST, SnipModule};
use kx_module_api::ports::{
    Calendar, CalendarKey, ClipboardWriterKey, Date, FileAskKey, FolderPath, FolderRevealKey,
    LocalTime, MediaOpenKey, MicPassKey, MicSeal, PanelLine, PicturesKey, Point, RecordAsk,
    RecorderKey, Rect, ScreenFreezeKey, SnipAct, SnipAside, SnipAsk, SnipClick, SnipFolderKey,
    SnipLine, SnipLit, SnipStage, SnipStart, SnipView,
};
use kx_module_api::{
    Bus, Event, Module, ModuleCx, ModuleError, Mono, Notice, ServiceId, ServiceKey, SettingsError,
    Subscription, keys,
};
use kx_platform_fake::{
    FakeCalendar, FakeClipboardWriter, FakeClock, FakeFileAsk, FakeFolderReveal, FakeMediaOpen,
    FakePictures, FakeRecorder, FakeScreenFreeze, FakeSnipFolder,
};
use kx_test_support::host::{FakeHost, Stored};
use toml::Value;

/// How long a test waits for the actor's thread before it fails.
const WAIT: Duration = Duration::from_secs(10);

/// `api` as the host stores service `K`, with its gate.
fn stored<K: ServiceKey>(api: Arc<K::Api>) -> (ServiceId, Stored) {
    (K::ID, (K::CAPABILITY, Box::new(api)))
}

/// `path` as a folder.
fn dir(path: &str) -> FolderPath {
    FolderPath::new(PathBuf::from(path)).unwrap()
}

/// The screen freezer, shared with the test.
fn freezer() -> FakeScreenFreeze {
    let screen = Rect {
        x: 0,
        y: 0,
        w: 64,
        h: 48,
    };
    FakeScreenFreeze::new(vec![(screen, 96)], Point { x: 1, y: 1 })
}

/// A host granting every capability the manifest declares and holding every service's fake, with `freeze` behind
/// the screen freezer.
fn host(freeze: &FakeScreenFreeze) -> FakeHost {
    let mut host = FakeHost::new(&MANIFEST);
    host.granted = MANIFEST.capabilities.to_vec();
    let clock = FakeClock::new(Mono::from_us(1_000));
    let folder = FakeSnipFolder::new(
        Arc::new(clock.clone()),
        dir("C:/Snips"),
        dir("C:/Snips/unsaved"),
    );
    let seal = MicSeal::new();
    let day = FakeCalendar::new(Date::new(2026, 11, 24).unwrap(), 9, 0, 0);
    host.clock = Arc::new(clock);
    host.services.extend([
        stored::<CalendarKey>(Arc::new(day)),
        stored::<ScreenFreezeKey>(Arc::new(freeze.clone())),
        stored::<RecorderKey>(Arc::new(FakeRecorder::new(seal.clone(), &folder))),
        stored::<MicPassKey>(Arc::new(seal)),
        stored::<FileAskKey>(Arc::new(FakeFileAsk::new(&folder))),
        stored::<MediaOpenKey>(Arc::new(FakeMediaOpen::new(&folder))),
        stored::<ClipboardWriterKey>(Arc::new(FakeClipboardWriter::new())),
        stored::<PicturesKey>(Arc::new(FakePictures::new())),
        stored::<FolderRevealKey>(Arc::new(FakeFolderReveal::new())),
        stored::<SnipFolderKey>(Arc::new(folder)),
    ]);
    host
}

/// Hands each `E` on `bus` to a channel, read with a timeout so no test sleeps.
fn ear<E: Event + Clone>(bus: &Bus) -> (Receiver<E>, Subscription) {
    let (tx, rx) = mpsc::channel();
    let ear = bus
        .subscribe(move |e: &E| drop(tx.send(e.clone())))
        .unwrap();
    (rx, ear)
}

/// Waits until `heard` gives an event `until` takes, passing over the others.
fn wait_until<E>(heard: &Receiver<E>, until: impl Fn(&E) -> bool) {
    while !until(&heard.recv_timeout(WAIT).unwrap()) {}
}

/// Waits for `want` on `heard`, passing over the others.
fn wait_for<E: PartialEq>(heard: &Receiver<E>, want: &E) {
    wait_until(heard, |e| e == want);
}

#[test]
fn the_snip_key_lights_the_key_and_freezes_and_the_stop_ends_the_snip() {
    let freeze = freezer();
    let (mut host, mut module) = (host(&freeze), SnipModule::new());
    let bus = Bus::new(Arc::clone(&host.core));
    let (lit, _lit) = ear::<SnipLit>(&bus);
    let (aside, _aside) = ear::<SnipAside>(&bus);
    let (views, _views) = ear::<SnipView>(&bus);
    module.start(&mut ModuleCx::new(&mut host)).unwrap();
    wait_for(&lit, &SnipLit::Off);
    bus.publish(&SnipStart).unwrap();
    wait_for(&lit, &SnipLit::On);
    wait_for(&aside, &SnipAside::Bubble);
    let told = views.try_iter().last().unwrap();
    let stepped = SnipAct {
        round: told.0.round,
        click: SnipClick::Aside,
    };
    bus.publish(&stepped).unwrap();
    wait_until(&views, |v| matches!(v.0.stage, SnipStage::Pick(_)));
    assert_eq!(freeze.caps().len(), 1, "the key froze the screens");
    host.held.clear();
    module.stop();
    wait_for(&lit, &SnipLit::Off);
    wait_for(&aside, &SnipAside::Back);
}

/// A calendar that panics, as a port broken on the actor's thread.
struct Broken;

impl Calendar for Broken {
    fn now(&self) -> LocalTime {
        panic!("a broken calendar");
    }
}

#[test]
fn a_panic_on_the_actors_thread_closes_the_windows_darkens_the_key_and_tells_the_user() {
    let (mut host, mut module) = (host(&freezer()), SnipModule::new());
    host.services
        .extend([stored::<CalendarKey>(Arc::new(Broken))]);
    let bus = Bus::new(Arc::clone(&host.core));
    let (lit, _lit) = ear::<SnipLit>(&bus);
    let (aside, _aside) = ear::<SnipAside>(&bus);
    let (views, _views) = ear::<SnipView>(&bus);
    let (notices, _notices) = ear::<Notice>(&bus);
    module.start(&mut ModuleCx::new(&mut host)).unwrap();
    bus.publish(&SnipStart).unwrap();
    wait_for(&aside, &SnipAside::Bubble);
    let told = views.try_iter().last().unwrap();
    let stepped = SnipAct {
        round: told.0.round,
        click: SnipClick::Aside,
    };
    bus.publish(&stepped).unwrap();
    wait_for(&aside, &SnipAside::Back);
    wait_for(&lit, &SnipLit::Off);
    wait_until(&views, |v| v.0.stage == SnipStage::Idle);
    let told = notices.recv_timeout(WAIT).unwrap();
    assert_eq!((told.key, told.module), (keys::MODULE_FAILED, Some(ID)));
    host.held.clear();
    module.stop();
}

#[test]
fn an_ask_tells_the_light_and_the_line_again_while_a_capture_runs() {
    let freeze = freezer();
    let (mut host, mut module) = (host(&freeze), SnipModule::new());
    let bus = Bus::new(Arc::clone(&host.core));
    let (lit, _lit) = ear::<SnipLit>(&bus);
    let (line, _line) = ear::<SnipLine>(&bus);
    module.start(&mut ModuleCx::new(&mut host)).unwrap();
    bus.publish(&SnipStart).unwrap();
    wait_for(&lit, &SnipLit::On);
    let picking = SnipLine(Some(PanelLine::None));
    wait_for(&line, &picking);
    bus.publish(&SnipAsk).unwrap();
    let again = lit.recv_timeout(WAIT).unwrap();
    assert_eq!(again, SnipLit::On, "lit until the capture ends");
    assert_eq!(line.recv_timeout(WAIT).unwrap(), picking);
    host.held.clear();
    module.stop();
}

#[test]
fn a_missing_service_fails_the_start() {
    let mut host = host(&freezer());
    host.services.remove(&RecorderKey::ID);
    assert!(
        SnipModule::new()
            .start(&mut ModuleCx::new(&mut host))
            .is_err()
    );
}

#[test]
fn a_section_its_check_refuses_fails_the_start() {
    let (mut host, mut module) = (host(&freezer()), SnipModule::new());
    let unlisted = (0..=u8::MAX).find(|f| !RecordAsk::FPS.contains(f));
    host.put("fps", Value::Integer(unlisted.unwrap().into()));
    let started = module.start(&mut ModuleCx::new(&mut host));
    let refused = matches!(
        started,
        Err(ModuleError::Settings(SettingsError::Invalid(why))) if why.contains("fps")
    );
    assert!(refused);
}
