//! The app's windows over one shared graphics device (ADR-0021): the device first, then the root QML; at the end the
//! engine and its windows go before the device they draw on.

use crate::MAIN_QML;
use crate::bridges::device::{Device, flush_deletes};
use crate::bridges::harden::{drawing_pinned, pin_imports};
use cxx::UniquePtr;
use cxx_qt_lib::{QQmlApplicationEngine, QUrl};
use std::path::{Path, PathBuf};
use std::pin::Pin;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

/// The QML engine with the app's windows and the device they share; it lives on the Qt GUI thread.
pub struct Shell {
    engine: UniquePtr<QQmlApplicationEngine>,
    device: Option<Device>,
    /// The folder QML imports are pinned to, as the uiAccess build pins them.
    pinned: Option<PathBuf>,
}

impl Shell {
    /// Makes the shared device, when every window draws on the GUI thread, then loads the root QML over the installed
    /// hub, its imports pinned to `pinned` when given. `None` when the root did not load. Call it on the GUI thread once
    /// Qt's app object exists.
    #[must_use]
    pub fn open(pinned: Option<&Path>) -> Option<Self> {
        let device = if drawing_pinned() {
            Device::make()
        } else {
            tracing::warn!("Qt may draw off the GUI thread, so each window makes its own device");
            None
        };
        let mut shell = Self {
            engine: UniquePtr::null(),
            device,
            pinned: pinned.map(Path::to_path_buf),
        };
        shell.load().then_some(shell)
    }

    /// The device the windows share; `None` when each makes its own.
    #[must_use]
    pub fn device(&self) -> Option<&Device> {
        self.device.as_ref()
    }

    /// Loads the root QML into a new engine; true when Qt made its objects. Loading from resources is synchronous.
    fn load(&mut self) -> bool {
        self.engine = QQmlApplicationEngine::new();
        let Some(mut engine) = self.engine.as_mut() else {
            return false;
        };
        if let Some(dir) = &self.pinned {
            pin_imports(engine.as_mut(), dir);
        }
        let loaded = root(engine);
        if !loaded {
            tracing::error!(url = MAIN_QML, "the root QML did not load");
        }
        loaded
    }

    /// Drops the engine and the windows it made, then the device they drew on.
    fn close(&mut self) {
        self.engine = UniquePtr::null();
        flush_deletes();
        self.device = None;
    }
}

impl Drop for Shell {
    fn drop(&mut self) {
        self.close();
    }
}

/// Loads the root QML into `engine`; true when Qt made its objects.
fn root(mut engine: Pin<&mut QQmlApplicationEngine>) -> bool {
    let failed = Arc::new(AtomicBool::new(false));
    let seen = Arc::clone(&failed);
    let watch = engine
        .as_mut()
        .on_object_creation_failed(move |_, _| seen.store(true, Ordering::SeqCst));
    engine.load(&QUrl::from(MAIN_QML));
    drop(watch);
    !failed.load(Ordering::SeqCst)
}
