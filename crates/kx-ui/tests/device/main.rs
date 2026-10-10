//! One graphics device for every window (ADR-0021): the app's QML over a hub with a fake face, where the keyboard,
//! its shadow and the language list a Loader makes later all draw on the shared device, on the GUI thread, made on
//! Qt's default adapter when the pinned one is gone. Qt wants its app object on the main thread, so this binary runs
//! without the test harness.

#![allow(
    clippy::expect_used,
    reason = "the shared board helper runs only in tests, where a failed setup should fail the test"
)]

#[path = "../shared/board.rs"]
mod board;
mod face;

use cxx::UniquePtr;
use cxx_qt_lib::{QByteArray, QGuiApplication, QQmlApplicationEngine, QUrl};
use face::Face;
use kx_module_api::ports::{BASE_DPI, Point, Rect, Screen};
use kx_platform_fake::{FakeBackdrop, FakeScreens, FakeSystemLook, FakeTray, FakeWindowGuard};
use kx_test_support::host::FakeHost;
use kx_ui::{Hub, MANIFEST, Shell, UiPorts, UiSettings, drawing};
use std::process::ExitCode;
use std::sync::Arc;

/// The fake tray's picture side.
const TRAY_PX: u32 = 16;
/// How many windows the fake guard reports, so each guard run ends at once.
const GUARDED: usize = 2;
/// Where the ticker says it is loaded from: it ends the event loop at each tick, so the test can look between ticks.
const TICK_URL: &str = "qrc:/kx-ui-tests/device-tick.qml";
/// The ticker.
const TICK: &str = r"import QtQuick
Timer { interval: 20; running: true; repeat: true; onTriggered: Qt.exit(0) }
";
/// An adapter no PC has, so the device is made on Qt's default, as when the pinned one is gone.
const GONE: u32 = 99;
/// How many ticks a step may take, as `tryVerify` waits in the Qt Quick tests.
const TICKS: usize = 250;

type Outcome = Result<(), String>;

/// What the test drives: the hub's face and the host behind its settings.
struct Rig {
    face: Arc<Face>,
    _host: FakeHost,
}

/// A hub over fresh fakes with the fake face attached, as the `ui` module attaches the keyboard's.
fn hub() -> Result<(Arc<Hub>, Rig), String> {
    let rect = Rect {
        x: 0,
        y: 0,
        w: 1920,
        h: 1040,
    };
    let screen = Screen {
        rect,
        dpi: BASE_DPI,
        primary: true,
    };
    let ports = UiPorts {
        guard: Arc::new(FakeWindowGuard::new(GUARDED)),
        screens: Arc::new(FakeScreens::new(vec![screen], Point { x: 0, y: 0 })),
        look: Arc::new(FakeSystemLook::new()),
        backdrop: Arc::new(FakeBackdrop::new()),
        tray: Arc::new(FakeTray::new(TRAY_PX)),
    };
    let (hub, _requests) = Hub::new(ports).map_err(|e| e.to_string())?;
    let face = Arc::new(Face::new(f64::from(board::row()))?);
    let mut host = FakeHost::new(&MANIFEST);
    let settings = UiSettings::from_defaults().map_err(|e| e.to_string())?;
    hub.attach(settings, face.clone(), host.handle());
    Ok((hub, Rig { face, _host: host }))
}

/// The ticker, loaded into its own engine, which the windows' rebuild leaves alone.
fn ticker() -> Result<UniquePtr<QQmlApplicationEngine>, String> {
    let mut engine = QQmlApplicationEngine::new();
    let e = engine.as_mut().ok_or("the ticker's engine did not start")?;
    e.load_data(&QByteArray::from(TICK), &QUrl::from(TICK_URL));
    Ok(engine)
}

/// Runs Qt's loop tick by tick until `done` holds; the error names `what` did not happen.
fn wait(app: &mut UniquePtr<QGuiApplication>, what: &str, done: impl Fn() -> bool) -> Outcome {
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

/// Starts Qt as the app does, then checks the shared device.
fn run() -> Outcome {
    if !kx_ui::pin_drawing() || !kx_ui::pin_adapter(GONE) {
        return Err("the drawing variables or the adapter were not set".into());
    }
    let (hub, rig) = hub()?;
    if !kx_ui::install(hub) {
        return Err("a hub was installed already".into());
    }
    let mut app = QGuiApplication::new();
    let _tick = ticker()?;
    let shell = Shell::open(None).ok_or("the root QML did not load")?;
    if kx_ui::qt_sees(kx_ui::ADAPTER_VAR) {
        return Err("the pin to a gone adapter stayed".into());
    }
    shared(&mut app, &shell, &rig)?;
    drop(shell);
    drop(app);
    Ok(())
}

/// The keyboard and its shadow, then the language list and its shadow, which a Loader makes later, all draw on the
/// shared device, and every frame ends on the GUI thread.
fn shared(app: &mut UniquePtr<QGuiApplication>, shell: &Shell, rig: &Rig) -> Outcome {
    let device = shell.device().ok_or("no shared device was made")?;
    wait(app, "the keyboard's first frame", || {
        !rig.face.placed().is_empty() && drawing().0 >= GUARDED
    })?;
    let before = drawing().0;
    rig.face.open_list();
    wait(app, "the language list's first frame", || {
        drawing().0 > before
    })?;
    let (windows, devices) = drawing();
    if device.windows() != windows || devices != 1 {
        return Err(format!(
            "{windows} windows draw on {devices} devices, {} on the shared one",
            device.windows()
        ));
    }
    device
        .gui_only()
        .then_some(())
        .ok_or_else(|| "a frame ended off the GUI thread".into())
}
