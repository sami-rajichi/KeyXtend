//! Qt on the main thread: hardened first, kept on the lowest-power graphics adapter and drawing every window on this
//! thread, then the app object, the shared graphics device with the root QML over the installed hub, and the event
//! loop.

use cxx_qt_lib::QGuiApplication;
use kx_ui::{Shell, harden_qt, pin_adapter, pin_drawing};
use std::path::Path;
use std::process::ExitCode;

/// Loads the root QML and runs Qt's loop until the UI ends; `end` runs after the loop, while Qt's objects still live.
/// With `pinned`, Qt loads plugins and QML modules only from that folder; with `adapter`, it draws on that one.
pub fn run(pinned: Option<&Path>, adapter: Option<u32>, end: impl FnOnce()) -> ExitCode {
    let cleared = harden_qt(pinned);
    if cleared > 0 {
        tracing::info!(cleared, "Qt variables were cleared before Qt started");
    }
    pin(adapter);
    if !pin_drawing() {
        tracing::warn!("Qt's drawing variables could not be set, so each window draws as Qt picks");
    }
    let mut app = QGuiApplication::new();
    let shell = Shell::open(pinned);
    let code = shell
        .as_ref()
        .and_then(|_| app.as_mut().map(QGuiApplication::exec));
    end();
    drop(shell);
    drop(app);
    if code == Some(0) {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    }
}

/// Has Qt draw on `adapter`, its place in DXGI's list; with none, Qt picks its own.
fn pin(adapter: Option<u32>) {
    let pinned = adapter.map(|index| (index, pin_adapter(index)));
    match pinned {
        Some((index, true)) => {
            tracing::info!(
                adapter = index,
                "Qt draws on the lowest-power graphics adapter"
            );
        }
        Some((index, false)) => {
            tracing::warn!(adapter = index, "the graphics adapter could not be pinned");
        }
        None => tracing::info!("no graphics adapter was found to pin, so Qt picks one"),
    }
}
