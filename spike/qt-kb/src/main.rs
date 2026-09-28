//! Qt 6 Quick face of the P1 spike: the stage-2 keyboard in QML, driven by spike-core.
// Release builds open no console window, which would take the foreground.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod board;
mod bridge;
mod panel;
mod tools;
mod voice;

use std::pin::Pin;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use cxx_qt_lib::{QGuiApplication, QQmlApplicationEngine, QUrl};
use spike_core::{config, theme, window};

/// Resource URL of `main.qml`, set once by `build.rs`.
const MAIN_QML: &str = env!("QT_KB_MAIN_QML");
/// Error reported when `main.qml` gives no window.
const NO_ROOT: &str = "main.qml created no root object";

fn main() {
    let cfg = config::load().unwrap_or_else(|err| fail(&err));
    let themes = theme::load().unwrap_or_else(|err| fail(&err));
    cfg.look
        .check_themes(&themes.list)
        .unwrap_or_else(|err| fail(&err));
    // Taken before our window exists, so `guard` can hand focus back to it.
    bridge::init(cfg, themes, window::foreground());

    let mut app = QGuiApplication::new();
    let mut engine = QQmlApplicationEngine::new();
    if !engine.as_mut().is_some_and(load) {
        fail(NO_ROOT);
    }
    let code = app.as_mut().map_or(1, |app| app.exec());
    drop(engine);
    drop(app);
    std::process::exit(code);
}

/// Reports `err` (console and TEMP file) and exits with code 1.
fn fail(err: &str) -> ! {
    config::report_error(bridge::FACE, err);
    std::process::exit(1);
}

/// Loads `main.qml`; true when the engine created its root object.
fn load(mut engine: Pin<&mut QQmlApplicationEngine>) -> bool {
    let created = Arc::new(AtomicBool::new(false));
    let seen = Arc::clone(&created);
    // Loading from qrc is synchronous, so the signal fires before `load` returns.
    let _watch = engine.as_mut().on_object_created(move |_, root, _| {
        seen.store(!root.is_null(), Ordering::Relaxed);
    });
    engine.load(&QUrl::from(MAIN_QML));
    created.load(Ordering::Relaxed)
}
