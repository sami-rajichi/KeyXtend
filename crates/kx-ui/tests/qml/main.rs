//! The QML module loads: the root file finds `KeyXtend.Ui`, and its `Ui` and `Keyboard` objects start over a hub.
//! Qt wants its app object on the main thread, so this binary runs without the test harness.

use cxx_qt_lib::{QGuiApplication, QQmlApplicationEngine, QUrl};
use kx_module_api::ports::{BASE_DPI, Point, Rect, Screen};
use kx_platform_fake::{FakeBackdrop, FakeScreens, FakeSystemLook, FakeWindowGuard};
use kx_ui::{Hub, MAIN_QML, UiPorts};
use std::process::ExitCode;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

/// A hub over fresh fakes, as the app builds over the real adapters.
fn hub() -> Option<Arc<Hub>> {
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
    let ports = UiPorts {
        guard: Arc::new(FakeWindowGuard::new(0)),
        screens: Arc::new(FakeScreens::new(vec![screen], Point { x: 0, y: 0 })),
        look: Arc::new(FakeSystemLook::new()),
        backdrop: Arc::new(FakeBackdrop::new()),
    };
    Hub::new(ports).ok().map(|(hub, _requests)| hub)
}

fn main() -> ExitCode {
    let Some(hub) = hub() else {
        eprintln!("the hub did not start");
        return ExitCode::FAILURE;
    };
    if !kx_ui::install(hub) {
        eprintln!("a hub was installed already");
        return ExitCode::FAILURE;
    }
    let _app = QGuiApplication::new();
    let mut engine = QQmlApplicationEngine::new();
    let failed = Arc::new(AtomicBool::new(false));
    let seen = Arc::clone(&failed);
    let Some(mut e) = engine.as_mut() else {
        eprintln!("Qt made no QML engine");
        return ExitCode::FAILURE;
    };
    e.as_mut()
        .on_object_creation_failed(move |_, _| seen.store(true, Ordering::SeqCst))
        .release();
    e.load(&QUrl::from(MAIN_QML));
    if failed.load(Ordering::SeqCst) {
        eprintln!("{MAIN_QML} did not load");
        return ExitCode::FAILURE;
    }
    ExitCode::SUCCESS
}
