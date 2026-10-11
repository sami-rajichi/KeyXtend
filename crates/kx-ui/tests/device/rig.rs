//! What the device test drives, the ticks it steps Qt's loop by, and the order of its steps.

use crate::face::Face;
use crate::logs::Lines;
use crate::{board, checks, left, rebuild};
use cxx::UniquePtr;
use cxx_qt_lib::{QByteArray, QGuiApplication, QQmlApplicationEngine, QUrl};
use kx_module_api::ports::{BASE_DPI, Point, Rect, Screen};
use kx_platform_fake::{
    FakeBackdrop, FakeMemory, FakeScreens, FakeSystemLook, FakeTray, FakeWindowGuard,
};
use kx_test_support::host::FakeHost;
use kx_ui::{Hub, MANIFEST, Pick, Shell, UiPorts, UiSettings};
use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering};
use tracing::Level;

/// The fake tray's picture side, and the one screen's work area.
const TRAY_PX: u32 = 16;
const SCREEN: Rect = Rect {
    x: 0,
    y: 0,
    w: 1920,
    h: 1040,
};
/// Where the tray icon is clicked: on the taskbar below that work area, this far in from its corner.
const TRAY_IN: i32 = 10;
pub(crate) const TRAY_AT: Point = Point {
    x: SCREEN.x + SCREEN.w - TRAY_IN,
    y: SCREEN.y + SCREEN.h + TRAY_IN,
};
/// How many windows the fake guard reports, so each guard run ends at once.
pub(crate) const GUARDED: usize = 2;
/// The ticker ends Qt's loop at each tick, so the test can look between ticks; its period in ms, and where it says it
/// is loaded from.
const TICK_MS: u32 = 20;
const TICK_URL: &str = "qrc:/kx-ui-tests/device-tick.qml";
/// The one-shot timer `untick` runs Qt's loop under: its exit code, which neither the ticker nor the loss step's
/// watchdog uses, and its URL.
const STILL_CODE: i32 = 4;
const STILL_URL: &str = "qrc:/kx-ui-tests/device-still.qml";
/// What a second `Ui` object taking the hub's wakes warns.
const SECOND_UI: &str = "second Ui";
/// An adapter no PC has, so no device can be made on it, as when the pinned one is gone.
pub(crate) const GONE: u32 = 99;
/// DXGI's first adapter, which every PC with a graphics adapter has.
pub(crate) const FIRST: u32 = 0;
/// How many ticks a step may take, as `tryVerify` waits in the Qt Quick tests.
pub(crate) const TICKS: usize = 250;
/// The rebuilds the loss and failure steps take: two tries for the first, one for each other.
const REBUILDS: u32 = 7;
/// The wait before the next try at a device, many ticks long, so Qt's loop must run through it.
const RETRY_MS: u64 = 200;

/// The adapter the next pick gives; each pick then falls back to `FIRST`.
static NEXT: AtomicU32 = AtomicU32::new(FIRST);

/// The test's adapter pick, which always finds one.
pub(crate) const PICK: Pick = || Some(NEXT.swap(FIRST, Ordering::SeqCst));

/// Has the next pick give `adapter`.
pub(crate) fn pick_next(adapter: u32) {
    NEXT.store(adapter, Ordering::SeqCst);
}

/// A step's verdict; the error says what failed.
pub(crate) type Outcome = Result<(), String>;

/// What the test drives: the hub, its face, the window guard, the tray, the memory port, the host behind its settings,
/// and the log.
pub(crate) struct Rig {
    /// The hub the app's QML reads.
    pub(crate) hub: Arc<Hub>,
    /// The keyboard face it draws.
    pub(crate) face: Arc<Face>,
    /// The guard that cuts and guards our windows.
    pub(crate) guard: Arc<FakeWindowGuard>,
    /// The tray its icon shows in.
    pub(crate) tray: Arc<FakeTray>,
    /// The memory port that hears each give-back.
    pub(crate) memory: Arc<FakeMemory>,
    /// The host behind the face's settings.
    pub(crate) host: FakeHost,
    /// Every log line so far.
    pub(crate) lines: Lines,
}

impl Rig {
    /// A hub over fresh fakes with the fake face attached, as the `ui` module attaches the keyboard's, room for every
    /// loss step's rebuilds, and a wait between tries.
    fn new(lines: Lines) -> Result<Self, String> {
        let screen = Screen {
            rect: SCREEN,
            dpi: BASE_DPI,
            primary: true,
        };
        let (tray, guard) = (
            Arc::new(FakeTray::new(TRAY_PX)),
            Arc::new(FakeWindowGuard::new(GUARDED)),
        );
        let memory = Arc::new(FakeMemory::new());
        let ports = UiPorts {
            guard: guard.clone(),
            screens: Arc::new(FakeScreens::new(vec![screen], Point { x: 0, y: 0 })),
            look: Arc::new(FakeSystemLook::new()),
            backdrop: Arc::new(FakeBackdrop::new()),
            tray: tray.clone(),
            memory: memory.clone(),
        };
        let (hub, _requests) = Hub::new(ports).map_err(|e| e.to_string())?;
        let face = Arc::new(Face::new(f64::from(board::row()))?);
        let host = FakeHost::new(&MANIFEST);
        let mut rig = Self {
            hub,
            face,
            guard,
            tray,
            memory,
            host,
            lines,
        };
        let s = UiSettings::from_defaults().map_err(|e| e.to_string())?;
        rig.show(UiSettings {
            device_rebuilds: REBUILDS,
            device_retry_ms: RETRY_MS,
            ..s
        });
        Ok(rig)
    }

    /// Hands the hub `settings` with the face, as the `ui` module does when it starts again.
    pub(crate) fn show(&mut self, settings: UiSettings) {
        let face = self.face.clone();
        self.hub.attach(settings, face, self.host.handle());
    }
}

/// QML text `qml`, loaded from `url` into its own engine, which the windows' rebuild leaves alone.
fn engine(qml: &str, url: &str) -> Result<UniquePtr<QQmlApplicationEngine>, String> {
    let mut engine = QQmlApplicationEngine::new();
    let e = engine.as_mut().ok_or("a test engine did not start")?;
    e.load_data(&QByteArray::from(qml), &QUrl::from(url));
    Ok(engine)
}

/// A timer that ends Qt's loop with `code` after `ms`, again and again when `repeat`, in its own engine.
pub(crate) fn timer(
    ms: u32,
    code: i32,
    repeat: bool,
    url: &str,
) -> Result<UniquePtr<QQmlApplicationEngine>, String> {
    let qml = format!(
        "import QtQuick\nTimer {{ interval: {ms}; running: true; repeat: {repeat}; onTriggered: Qt.exit({code}) }}\n"
    );
    engine(&qml, url)
}

/// The ticker.
pub(crate) fn ticker() -> Result<UniquePtr<QQmlApplicationEngine>, String> {
    timer(TICK_MS, 0, true, TICK_URL)
}

/// Stops the ticker, then runs Qt's loop until a one-shot timer of its own ends it. A tick that fired as the loop was
/// ending leaves its exit queued, which would otherwise end the next run of the loop at once.
pub(crate) fn untick(
    app: &mut UniquePtr<QGuiApplication>,
    tick: &mut UniquePtr<QQmlApplicationEngine>,
) -> Outcome {
    *tick = UniquePtr::null();
    for _ in 0..TICKS {
        let _still = timer(TICK_MS, STILL_CODE, false, STILL_URL)?;
        if app.as_mut().map(|a| a.exec()) == Some(STILL_CODE) {
            return Ok(());
        }
    }
    Err("an exit stayed queued after the ticker stopped".into())
}

/// Runs Qt's loop tick by tick until `done` holds; the error names `what` did not happen.
pub(crate) fn wait(
    app: &mut UniquePtr<QGuiApplication>,
    what: &str,
    done: impl Fn() -> bool,
) -> Outcome {
    for _ in 0..TICKS {
        if done() {
            return Ok(());
        }
        app.as_mut().map(|a| a.exec());
    }
    done()
        .then_some(())
        .ok_or_else(|| format!("{what} did not happen"))
}

/// Runs Qt's loop for `n` ticks, so a step can check that nothing happened meanwhile.
pub(crate) fn ticks(app: &mut UniquePtr<QGuiApplication>, n: usize) {
    for _ in 0..n {
        app.as_mut().map(|a| a.exec());
    }
}

/// Starts Qt as the app does, then checks the shared device, a theme picked through the wake, a window hiding before
/// it draws one, the losses, a window that cannot draw, the fonts, losses past the most, sharing turned off, the
/// fallback and the End.
pub(crate) fn run() -> Outcome {
    let lines = Lines::install().ok_or("a log capture was set already")?;
    if !kx_ui::pin_drawing() || !kx_ui::pin_adapter(GONE) {
        return Err("the drawing variables or the adapter were not set".into());
    }
    let mut rig = Rig::new(lines)?;
    if !kx_ui::install(rig.hub.clone()) {
        return Err("a hub was installed already".into());
    }
    let mut app = QGuiApplication::new();
    let mut tick = ticker()?;
    let mut shell = Shell::open(None, PICK).ok_or("the root QML did not load")?;
    if kx_ui::qt_sees(kx_ui::ADAPTER_VAR) {
        return Err("the pin to a gone adapter stayed".into());
    }
    checks::shared(&mut app, &shell, &rig)?;
    checks::relooked(&mut app, &mut rig)?;
    checks::hid(&mut app, &mut rig)?;
    let kept = rebuild::once(&mut app, &mut shell, &rig, &mut tick)?;
    left::away(&mut app, &mut shell, &rig)?;
    left::minimised(&mut app, &mut shell, &rig)?;
    left::theme(&mut app, &mut shell, &mut rig)?;
    left::waiting(&mut app, &mut shell, &mut rig)?;
    rebuild::failed(&mut app, &mut shell, &rig, &mut tick)?;
    checks::fonts(&mut rig)?;
    rebuild::past_most(&mut app, &mut shell, &mut rig, kept)?;
    rebuild::spent(&mut app, &mut shell, &rig)?;
    drop(shell);
    rebuild::unshared(&mut app, &mut rig)?;
    let shell = Shell::open_with(None, PICK, None).ok_or("the root QML did not load again")?;
    checks::own(&mut app, &shell, &rig)?;
    checks::ends(&mut app, &rig)?;
    drop(shell);
    drop(app);
    match rig.lines.count(Level::WARN, SECOND_UI) {
        0 => Ok(()),
        n => Err(format!("{n} Ui objects took the wakes from a live one")),
    }
}
