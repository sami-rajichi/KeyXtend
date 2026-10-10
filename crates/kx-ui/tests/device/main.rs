//! One graphics device for every window (ADR-0021): the app's QML over a hub with a fake face, where the keyboard,
//! its shadow and the language list a Loader makes later all draw on the shared device, on the GUI thread, made on
//! Qt's default adapter when the pinned one is gone; a lost device brings the windows back over the same hub; Qt's
//! font list takes each theme's bundled fonts; without a shared device, each window draws on its own. Qt wants its app
//! object on the main thread, so this binary runs without the test harness.

#![allow(
    clippy::expect_used,
    reason = "the shared board helper runs only in tests, where a failed setup should fail the test"
)]

#[path = "../shared/board.rs"]
mod board;
mod checks;
mod face;
mod logs;
#[cfg(debug_assertions)]
mod rebuild;

use cxx::UniquePtr;
use cxx_qt_lib::{QByteArray, QGuiApplication, QQmlApplicationEngine, QUrl};
use face::Face;
use kx_module_api::ports::{BASE_DPI, Point, Rect, Screen};
use kx_platform_fake::{FakeBackdrop, FakeScreens, FakeSystemLook, FakeTray, FakeWindowGuard};
use kx_test_support::host::FakeHost;
use kx_ui::{Hub, MANIFEST, Shell, UiPorts, UiSettings};
use logs::Lines;
use std::process::ExitCode;
use std::sync::Arc;
use tracing::Level;

/// The fake tray's picture side.
const TRAY_PX: u32 = 16;
/// How many windows the fake guard reports, so each guard run ends at once.
pub(crate) const GUARDED: usize = 2;
/// Where the ticker says it is loaded from: it ends the event loop at each tick, so the test can look between ticks.
const TICK_URL: &str = "qrc:/kx-ui-tests/device-tick.qml";
/// The ticker.
const TICK: &str = r"import QtQuick
Timer { interval: 20; running: true; repeat: true; onTriggered: Qt.exit(0) }
";
/// What a second `Ui` object taking the hub's wakes warns.
const SECOND_UI: &str = "second Ui";
/// An adapter no PC has, so the device is made on Qt's default, as when the pinned one is gone.
const GONE: u32 = 99;
/// How many ticks a step may take, as `tryVerify` waits in the Qt Quick tests.
const TICKS: usize = 250;

/// A step's verdict; the error says what failed.
pub(crate) type Outcome = Result<(), String>;

/// What the test drives: the hub, its face, the tray, the host behind its settings, and the log.
pub(crate) struct Rig {
    /// The hub the app's QML reads.
    pub(crate) hub: Arc<Hub>,
    /// The keyboard face it draws.
    pub(crate) face: Arc<Face>,
    /// The tray its icon shows in.
    pub(crate) tray: Arc<FakeTray>,
    /// The host behind the face's settings.
    pub(crate) host: FakeHost,
    /// Every log line so far.
    pub(crate) lines: Lines,
}

impl Rig {
    /// A hub over fresh fakes with the fake face attached, as the `ui` module attaches the keyboard's.
    fn new(lines: Lines) -> Result<Self, String> {
        let screen = Screen {
            rect: Rect {
                x: 0,
                y: 0,
                w: 1920,
                h: 1040,
            },
            dpi: BASE_DPI,
            primary: true,
        };
        let tray = Arc::new(FakeTray::new(TRAY_PX));
        let ports = UiPorts {
            guard: Arc::new(FakeWindowGuard::new(GUARDED)),
            screens: Arc::new(FakeScreens::new(vec![screen], Point { x: 0, y: 0 })),
            look: Arc::new(FakeSystemLook::new()),
            backdrop: Arc::new(FakeBackdrop::new()),
            tray: tray.clone(),
        };
        let (hub, _requests) = Hub::new(ports).map_err(|e| e.to_string())?;
        let face = Arc::new(Face::new(f64::from(board::row()))?);
        let host = FakeHost::new(&MANIFEST);
        let mut rig = Self {
            hub,
            face,
            tray,
            host,
            lines,
        };
        rig.show(UiSettings::from_defaults().map_err(|e| e.to_string())?);
        Ok(rig)
    }

    /// Hands the hub `settings` with the face, as the `ui` module does when it starts again.
    pub(crate) fn show(&mut self, settings: UiSettings) {
        let face = self.face.clone();
        self.hub.attach(settings, face, self.host.handle());
    }
}

/// The ticker, loaded into its own engine, which the windows' rebuild leaves alone.
fn ticker() -> Result<UniquePtr<QQmlApplicationEngine>, String> {
    let mut engine = QQmlApplicationEngine::new();
    let e = engine.as_mut().ok_or("the ticker's engine did not start")?;
    e.load_data(&QByteArray::from(TICK), &QUrl::from(TICK_URL));
    Ok(engine)
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

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(why) => {
            eprintln!("{why}");
            ExitCode::FAILURE
        }
    }
}

/// Starts Qt as the app does, then checks the shared device, a loss, the fonts, losses past the most, the fallback and
/// the End.
fn run() -> Outcome {
    let lines = Lines::install().ok_or("a log capture was set already")?;
    if !kx_ui::pin_drawing() || !kx_ui::pin_adapter(GONE) {
        return Err("the drawing variables or the adapter were not set".into());
    }
    let mut rig = Rig::new(lines)?;
    if !kx_ui::install(rig.hub.clone()) {
        return Err("a hub was installed already".into());
    }
    let mut app = QGuiApplication::new();
    let _tick = ticker()?;
    #[cfg_attr(
        not(debug_assertions),
        allow(unused_mut, reason = "only the loss steps change it")
    )]
    let mut shell = Shell::open(None).ok_or("the root QML did not load")?;
    if kx_ui::qt_sees(kx_ui::ADAPTER_VAR) {
        return Err("the pin to a gone adapter stayed".into());
    }
    checks::shared(&mut app, &shell, &rig)?;
    #[cfg(debug_assertions)]
    let kept = rebuild::once(&mut app, &mut shell, &rig)?;
    checks::fonts(&mut rig)?;
    #[cfg(debug_assertions)]
    rebuild::past_most(&mut app, &mut shell, &mut rig, kept)?;
    drop(shell);
    let shell = Shell::open_with(None, None).ok_or("the root QML did not load again")?;
    checks::own(&mut app, &shell, &rig)?;
    checks::ends(&mut app, &rig)?;
    drop(shell);
    drop(app);
    match rig.lines.count(Level::WARN, SECOND_UI) {
        0 => Ok(()),
        n => Err(format!("{n} Ui objects took the wakes from a live one")),
    }
}
