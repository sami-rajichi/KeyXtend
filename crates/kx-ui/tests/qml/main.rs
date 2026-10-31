//! The QML module loads: the root file finds `KeyXtend.Ui`, its `Ui` object starts over the hub and follows the
//! system look, the tray icon gets the picture the root draws with the icon provider, and every bundled font loads
//! from the compiled-in resources (the tint test checks the icons). Qt wants its app object on the main thread, so
//! this binary runs without the test harness.

use cxx::UniquePtr;
use cxx_qt_lib::{QByteArray, QGuiApplication, QQmlApplicationEngine, QUrl};
use kx_module_api::ports::{BASE_DPI, Point, Rect, Screen};
use kx_platform::ui::SysLook;
use kx_platform_fake::{
    FakeBackdrop, FakeMemory, FakeReveal, FakeScreens, FakeSpeaker, FakeSystemLook, FakeTray,
    FakeWindowGuard,
};
use kx_ui::{ASSETS, Hub, MAIN_QML, UiPorts};
use std::process::ExitCode;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

/// The fake tray's picture side.
const TRAY_PX: u32 = 16;

/// A hub over fresh fakes, as the app builds over the real adapters, with the fake look and tray it drives.
fn hub() -> Option<(Arc<Hub>, Arc<FakeSystemLook>, Arc<FakeTray>)> {
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
    let look = Arc::new(FakeSystemLook::new());
    let tray = Arc::new(FakeTray::new(TRAY_PX));
    let ports = UiPorts {
        guard: Arc::new(FakeWindowGuard::new(0)),
        screens: Arc::new(FakeScreens::new(vec![screen], Point { x: 0, y: 0 })),
        look: look.clone(),
        backdrop: Arc::new(FakeBackdrop::new()),
        tray: tray.clone(),
        memory: Arc::new(FakeMemory::new()),
        speaker: Arc::new(FakeSpeaker::new()),
        reveal: Arc::new(FakeReveal::new()),
    };
    Hub::new(ports)
        .ok()
        .map(|(hub, _requests)| (hub, look, tray))
}

/// True when the hub draws the dark look, as it does once it follows a dark system.
fn dark(hub: &Hub) -> bool {
    let look: Option<serde_json::Value> = serde_json::from_str(&hub.look_json()).ok();
    look.is_some_and(|v| v["dark"] == true)
}

/// Where the asset check says it is loaded from; it imports only Qt's own modules.
const CHECK_URL: &str = "qrc:/kx-ui-tests/assets.qml";
/// Where the ticker says it is loaded from: it ends the event loop at each tick, so the test can look between ticks.
const TICK_URL: &str = "qrc:/kx-ui-tests/tick.qml";
/// The ticker.
const TICK: &str = r"import QtQuick
Timer { interval: 20; running: true; repeat: true; onTriggered: Qt.exit(0) }
";
/// How many ticks the tray picture may take, as `tryVerify` waits in the Qt Quick tests.
const TICKS: usize = 250;

/// Loads QML into an engine the caller keeps, with `load`; `None` when Qt could not make its object.
fn load(
    load: impl FnOnce(std::pin::Pin<&mut QQmlApplicationEngine>),
) -> Option<UniquePtr<QQmlApplicationEngine>> {
    let mut engine = QQmlApplicationEngine::new();
    let failed = Arc::new(AtomicBool::new(false));
    let seen = Arc::clone(&failed);
    let mut e = engine.as_mut()?;
    e.as_mut()
        .on_object_creation_failed(move |_, _| seen.store(true, Ordering::SeqCst))
        .release();
    load(e);
    (!failed.load(Ordering::SeqCst)).then_some(engine)
}

/// QML that loads every font `Ui.assets()` lists, warns about each one that fails, and exits with their count once
/// the event loop runs.
fn asset_check() -> String {
    format!(
        r#"import QtQuick
QtObject {{
    id: check
    readonly property var assets: {ASSETS}
    function loads(url) {{
        const o = Qt.createQmlObject("import QtQuick; FontLoader {{}}", check);
        o.source = url;
        return o.status === FontLoader.Ready;
    }}
    Component.onCompleted: {{
        const bad = assets.fonts.filter(u => !loads(u));
        bad.forEach(u => console.warn("did not load: " + u));
        Qt.callLater(() => Qt.exit(assets.fonts.length > 0 ? bad.length : 1));
    }}
}}
"#
    )
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

/// Loads the root over the hub, then checks the look, the assets and the tray picture; the error says what failed.
fn run() -> Result<(), String> {
    let (hub, look, tray) = hub().ok_or("the hub did not start")?;
    if !kx_ui::install(Arc::clone(&hub)) {
        return Err("a hub was installed already".into());
    }
    let mut app = QGuiApplication::new();
    let _engine = load(|mut e| {
        kx_ui::serve_icons(e.as_mut());
        kx_ui::serve_clips(e.as_mut());
        kx_ui::serve_shadows(e.as_mut());
        e.load(&QUrl::from(MAIN_QML));
    })
    .ok_or_else(|| format!("{MAIN_QML} did not load"))?;
    look.set(SysLook {
        dark: true,
        ..SysLook::default()
    });
    if !dark(&hub) {
        return Err("the Ui object did not start following the system look".into());
    }
    check_assets(&mut app)?;
    check_tray(&mut app, &tray)
}

/// Runs the asset check to its end; an error when a font failed, or no font is bundled.
fn check_assets(app: &mut UniquePtr<QGuiApplication>) -> Result<(), String> {
    let data = QByteArray::from(asset_check().as_str());
    let _check = load(|e| e.load_data(&data, &QUrl::from(CHECK_URL)))
        .ok_or("the asset check did not load")?;
    if app.as_mut().map(|a| a.exec()) == Some(0) {
        Ok(())
    } else {
        Err("fonts did not load from the resources, or none is bundled".into())
    }
}

/// Runs the event loop tick by tick until the tray holds a picture, then checks its size and tooltip.
fn check_tray(app: &mut UniquePtr<QGuiApplication>, tray: &FakeTray) -> Result<(), String> {
    let tick = QByteArray::from(TICK);
    let _tick =
        load(|e| e.load_data(&tick, &QUrl::from(TICK_URL))).ok_or("the ticker did not load")?;
    for _ in 0..TICKS {
        if !tray.shown().is_empty() {
            break;
        }
        app.as_mut().map(|a| a.exec());
    }
    match tray.shown().first() {
        Some((picture, tip)) if picture.width() == TRAY_PX && !tip.is_empty() => Ok(()),
        Some((picture, _)) => Err(format!(
            "the tray picture is {} pixels wide, not {TRAY_PX}",
            picture.width()
        )),
        None => Err("the root drew no tray picture".into()),
    }
}
