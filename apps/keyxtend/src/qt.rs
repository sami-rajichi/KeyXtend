//! Qt on the main thread: the app object, the root QML over the installed hub, and the event loop.

use cxx_qt_lib::{QGuiApplication, QQmlApplicationEngine, QUrl};
use kx_ui::MAIN_QML;
use std::pin::Pin;
use std::process::ExitCode;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

/// Loads the root QML and runs Qt's loop until the UI ends; `end` runs after the loop, while Qt's objects still live.
pub fn run(end: impl FnOnce()) -> ExitCode {
    let mut app = QGuiApplication::new();
    let mut engine = QQmlApplicationEngine::new();
    let code = if engine.as_mut().is_some_and(load) {
        app.as_mut().map(QGuiApplication::exec)
    } else {
        tracing::error!(url = MAIN_QML, "the root QML did not load");
        None
    };
    end();
    drop(engine);
    drop(app);
    if code == Some(0) {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    }
}

/// Loads the root QML; true when Qt made its objects. Loading from resources is synchronous.
fn load(mut engine: Pin<&mut QQmlApplicationEngine>) -> bool {
    let failed = Arc::new(AtomicBool::new(false));
    let seen = Arc::clone(&failed);
    let watch = engine
        .as_mut()
        .on_object_creation_failed(move |_, _| seen.store(true, Ordering::SeqCst));
    engine.load(&QUrl::from(MAIN_QML));
    drop(watch);
    !failed.load(Ordering::SeqCst)
}
