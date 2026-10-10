//! The app's windows over one shared graphics device (ADR-0021): the device and the theme's fonts first, then the root
//! QML; at the end the engine and its windows go before the device they draw on. A lost device's windows are made
//! again over the same hub (ADR-0018), on a new device on the lowest-power adapter found anew.

use crate::bridges::device::{Device, flush_deletes};
use crate::bridges::fonts::QtFontBook;
use crate::bridges::harden::{drawing_pinned, pin_adapter, pin_imports, unpin_adapter};
use crate::bridges::installed;
use crate::device::{Rebuilds, Remake};
use crate::{MAIN_QML, Resize};
use cxx::UniquePtr;
use cxx_qt_lib::{QQmlApplicationEngine, QUrl};
use std::path::{Path, PathBuf};
use std::pin::Pin;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

/// How the shell finds the graphics adapter again after a loss, and waits between tries at a new device.
#[derive(Clone, Copy, Debug)]
pub struct Gpu {
    /// The lowest-power hardware adapter's place in DXGI's list now; `None` lets Qt pick.
    pub pick: fn() -> Option<u32>,
    /// Waits as long as it is told.
    pub wait: fn(Duration),
}

/// What follows the end of Qt's loop.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum After {
    /// A lost device's windows were made again, so the loop runs again.
    Again,
    /// No device was lost, or the UI ended: the app ends.
    End,
    /// A lost device's windows could not be made again: the app ends with a failure.
    Failed,
}

/// The QML engine with the app's windows and the device they share; it lives on the Qt GUI thread.
pub struct Shell {
    engine: UniquePtr<QQmlApplicationEngine>,
    device: Option<Device>,
    /// The folder QML imports are pinned to, as the uiAccess build pins them.
    pinned: Option<PathBuf>,
    gpu: Gpu,
    rebuilds: Rebuilds,
}

impl Shell {
    /// Makes the shared device, when every window draws on the GUI thread, then loads the root QML over the installed
    /// hub, its imports pinned to `pinned` when given; `gpu` finds the adapter again after a loss. `None` when the root
    /// did not load. Call it on the GUI thread once Qt's app object exists.
    #[must_use]
    pub fn open(pinned: Option<&Path>, gpu: Gpu) -> Option<Self> {
        Self::open_with(pinned, gpu, shared())
    }

    /// Loads the theme's fonts, then the root QML over `device`, or with `None` over a device each window makes, as Qt
    /// does alone; `pinned` and `gpu` as in `open`. Call it on the GUI thread once Qt's app object exists.
    #[must_use]
    pub fn open_with(pinned: Option<&Path>, gpu: Gpu, device: Option<Device>) -> Option<Self> {
        if let Some(hub) = installed() {
            hub.set_fonts(Box::new(QtFontBook));
        }
        let mut shell = Self {
            engine: UniquePtr::null(),
            device,
            pinned: pinned.map(Path::to_path_buf),
            gpu,
            rebuilds: Rebuilds::default(),
        };
        shell.load().then_some(shell)
    }

    /// The device the windows share; `None` when each makes its own.
    #[must_use]
    pub fn device(&self) -> Option<&Device> {
        self.device.as_ref()
    }

    /// After Qt's loop ends: when a frame found the shared device lost and the UI has not ended, the windows are made
    /// again over the same hub, a resize ended first, on a new device while `[ui] device_rebuilds` lasts.
    pub fn rebuilt(&mut self) -> After {
        let lost = self.device.as_ref().filter(|d| d.lost());
        let Some(gui_only) = lost.map(Device::gui_only) else {
            return After::End;
        };
        let Some(hub) = installed().filter(|h| !h.ended()) else {
            return After::End;
        };
        let s = hub.settings();
        self.close();
        let _ = hub.resize(Resize::End);
        let mut again = Again {
            gpu: self.gpu,
            pause: Duration::from_millis(s.device_retry_ms),
        };
        self.device = self
            .rebuilds
            .remake(s.device_rebuilds, gui_only, &mut again);
        if self.load() {
            After::Again
        } else {
            After::Failed
        }
    }

    /// Marks the shared device lost, as a driver reset would, so the next frame finds it; debug builds only, for tests.
    #[cfg(debug_assertions)]
    pub fn lose(&mut self) {
        if let Some(device) = self.device.as_mut() {
            device.lose();
        }
    }

    /// Loads the theme's fonts, as a wake lost meanwhile may have changed it, then the root QML into a new engine; true
    /// when Qt made its objects. Loading from resources is synchronous.
    fn load(&mut self) -> bool {
        if let Some(hub) = installed() {
            let _ = hub.sync_fonts();
        }
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

    /// Lets the hub's waker go, then drops the engine and the windows it made, then the device they drew on.
    fn close(&mut self) {
        if let Some(hub) = installed() {
            hub.drop_waker();
        }
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

/// Makes a lost device again through Qt, on the adapter `gpu` picks, waiting `pause` between tries.
struct Again {
    gpu: Gpu,
    pause: Duration,
}

impl Remake for Again {
    type Device = Device;

    fn repin(&mut self) {
        match (self.gpu.pick)() {
            Some(index) if pin_adapter(index) => {
                tracing::info!(
                    adapter = index,
                    "the lowest-power graphics adapter is pinned again"
                );
            }
            Some(index) => tracing::warn!(
                adapter = index,
                "the graphics adapter could not be pinned again"
            ),
            None => {
                let _ = unpin_adapter();
                tracing::info!("no graphics adapter was found to pin, so Qt picks one");
            }
        }
    }

    fn make(&mut self) -> Option<Device> {
        Device::pinned()
    }

    fn pause(&mut self) {
        (self.gpu.wait)(self.pause);
    }
}

/// The device every window shares, when every window draws on the GUI thread; else, or when none can be made, `None`
/// and each window makes its own.
fn shared() -> Option<Device> {
    if drawing_pinned() {
        return Device::make();
    }
    tracing::warn!("Qt may draw off the GUI thread, so each window makes its own device");
    None
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
