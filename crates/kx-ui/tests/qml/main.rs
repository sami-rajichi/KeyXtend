//! The QML module loads: the root file finds `KeyXtend.Ui`, and its `Ui` object starts over the hub and follows the
//! system look. Qt wants its app object on the main thread, so this binary runs without the test harness.

use cxx::UniquePtr;
use cxx_qt_lib::{QGuiApplication, QQmlApplicationEngine, QUrl};
use kx_module_api::ports::{BASE_DPI, Point, Rect, Screen};
use kx_platform::ui::SysLook;
use kx_platform_fake::{FakeBackdrop, FakeScreens, FakeSystemLook, FakeWindowGuard};
use kx_ui::{Hub, MAIN_QML, UiPorts};
use std::process::ExitCode;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

/// A hub over fresh fakes, as the app builds over the real adapters, with the fake look it follows.
fn hub() -> Option<(Arc<Hub>, Arc<FakeSystemLook>)> {
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
    let ports = UiPorts {
        guard: Arc::new(FakeWindowGuard::new(0)),
        screens: Arc::new(FakeScreens::new(vec![screen], Point { x: 0, y: 0 })),
        look: look.clone(),
        backdrop: Arc::new(FakeBackdrop::new()),
    };
    Hub::new(ports).ok().map(|(hub, _requests)| (hub, look))
}

/// True when the hub draws the dark look, as it does once it follows a dark system.
fn dark(hub: &Hub) -> bool {
    let look: Option<serde_json::Value> = serde_json::from_str(&hub.look_json()).ok();
    look.is_some_and(|v| v["dark"] == true)
}

/// Loads the root file into an engine the caller keeps; `None` when Qt could not make it.
fn load() -> Option<UniquePtr<QQmlApplicationEngine>> {
    let mut engine = QQmlApplicationEngine::new();
    let failed = Arc::new(AtomicBool::new(false));
    let seen = Arc::clone(&failed);
    let mut e = engine.as_mut()?;
    e.as_mut()
        .on_object_creation_failed(move |_, _| seen.store(true, Ordering::SeqCst))
        .release();
    e.load(&QUrl::from(MAIN_QML));
    (!failed.load(Ordering::SeqCst)).then_some(engine)
}

fn main() -> ExitCode {
    let Some((hub, look)) = hub() else {
        eprintln!("the hub did not start");
        return ExitCode::FAILURE;
    };
    if !kx_ui::install(Arc::clone(&hub)) {
        eprintln!("a hub was installed already");
        return ExitCode::FAILURE;
    }
    let _app = QGuiApplication::new();
    let Some(_engine) = load() else {
        eprintln!("{MAIN_QML} did not load");
        return ExitCode::FAILURE;
    };
    look.set(SysLook {
        dark: true,
        ..SysLook::default()
    });
    if !dark(&hub) {
        eprintln!("the Ui object did not start following the system look");
        return ExitCode::FAILURE;
    }
    ExitCode::SUCCESS
}
