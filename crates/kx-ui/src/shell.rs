//! The app's windows over one shared graphics device (ADR-0021): the device and the theme's fonts first, then the root
//! QML; at the end the engine and its windows go before the device they draw on. A lost device's windows, or windows
//! that could not start drawing, are made again over the same hub (ADR-0018), on a new device on the lowest-power
//! adapter found anew, one try per turn of Qt's loop.

use crate::bridges::clip::serve_clips;
use crate::bridges::device::{Device, Watch, flush_deletes};
use crate::bridges::fonts::QtFontBook;
use crate::bridges::harden::{drawing_pinned, pin_adapter, pin_imports, unpin_adapter};
use crate::bridges::installed;
use crate::bridges::shadows::serve_shadows;
use crate::bridges::tint::serve_icons;
use crate::device::{Cause, Next, Rebuilds, Remake};
use crate::hub::Hub;
use crate::{MAIN_QML, Resize};
use cxx::UniquePtr;
use cxx_qt_lib::{QQmlApplicationEngine, QUrl};
use std::path::{Path, PathBuf};
use std::pin::Pin;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

/// Finds the lowest-power hardware adapter's place in DXGI's list now; `None` lets Qt pick.
pub type Pick = fn() -> Option<u32>;

/// What follows the end of Qt's loop.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum After {
    /// The windows were made again, so the loop runs again.
    Again,
    /// No device could be made yet, so the loop runs until the next try, after `[ui] device_retry_ms`.
    Wait,
    /// No device was lost and no window failed, or the UI ended: the app ends.
    End,
    /// The windows could not be made again: the app ends with a failure.
    Failed,
}

/// The QML engine with the app's windows and the device they share; it lives on the Qt GUI thread.
pub struct Shell {
    engine: UniquePtr<QQmlApplicationEngine>,
    device: Option<Device>,
    watch: Watch,
    /// The folder QML imports are pinned to, as the uiAccess build pins them.
    pinned: Option<PathBuf>,
    pick: Pick,
    rebuilds: Rebuilds,
    /// Why the windows wait for the next try at a device, while they do.
    waiting: Option<Cause>,
    /// False once sharing is off for the run: Qt may draw off the GUI thread, or a frame did.
    shares: bool,
}

impl Shell {
    /// Makes the shared device, when every window draws on the GUI thread, then loads the root QML over the installed
    /// hub, its imports pinned to `pinned` when given; `pick` finds the adapter again after a loss. `None` when the
    /// root did not load. Call it on the GUI thread once Qt's app object exists.
    #[must_use]
    pub fn open(pinned: Option<&Path>, pick: Pick) -> Option<Self> {
        Self::open_with(pinned, pick, shared())
    }

    /// Loads the theme's fonts, then the root QML over `device`, or with `None` over a device each window makes, as Qt
    /// does alone; `pinned` and `pick` as in `open`. Call it on the GUI thread once Qt's app object exists.
    #[must_use]
    pub fn open_with(pinned: Option<&Path>, pick: Pick, device: Option<Device>) -> Option<Self> {
        if let Some(hub) = installed() {
            hub.set_fonts(Box::new(QtFontBook));
        }
        let mut shell = Self {
            engine: UniquePtr::null(),
            device,
            watch: Watch::new(),
            pinned: pinned.map(Path::to_path_buf),
            pick,
            rebuilds: Rebuilds::default(),
            waiting: None,
            shares: drawing_pinned(),
        };
        shell.load().then_some(shell)
    }

    /// The device the windows share; `None` when each makes its own.
    #[must_use]
    pub fn device(&self) -> Option<&Device> {
        self.device.as_ref()
    }

    /// After Qt's loop ends: when the device was lost or a window could not start drawing, and the UI has not ended,
    /// the windows are made again over the same hub, a resize ended first, one try at a new device each time, while
    /// `[ui] device_rebuilds` lasts. It never waits itself: a failed try asks for another after `device_retry_ms`.
    pub fn rebuilt(&mut self) -> After {
        let Some(hub) = installed().filter(|h| !h.ended()) else {
            return After::End;
        };
        let cause = match self.waiting.take() {
            Some(cause) if !self.watch.ready() => {
                self.waiting = Some(cause);
                return After::Wait;
            }
            Some(cause) => cause,
            None => {
                let Some(cause) = self.cause() else {
                    return After::End;
                };
                self.close();
                let _ = hub.resize(Resize::End);
                cause
            }
        };
        self.attempt(&hub, cause)
    }

    /// Marks the shared device lost, as a driver reset would, so the next frame finds it; debug builds only, for tests.
    #[cfg(debug_assertions)]
    pub fn lose(&mut self) {
        if let Some(device) = self.device.as_mut() {
            device.lose();
        }
    }

    /// Has a window report that its scene graph could not start, as when it cannot make its device during a reset;
    /// debug builds only, for tests.
    #[cfg(debug_assertions)]
    pub fn fail_window(&mut self) {
        self.watch.fail();
    }

    /// Has the next frame on the shared device count as drawn off the GUI thread, which ends sharing; debug builds
    /// only, for tests.
    #[cfg(debug_assertions)]
    pub fn stray(&mut self) {
        if let Some(device) = self.device.as_mut() {
            device.stray();
        }
    }

    /// Why the windows must be made again, if they must: the shared device was lost, or a window could not draw.
    fn cause(&mut self) -> Option<Cause> {
        let failed = self.watch.take_failed();
        let lost = self.device.as_ref().filter(|d| d.lost());
        let lost = lost.map(|d| Cause::Lost {
            gui_only: d.gui_only(),
        });
        lost.or(failed.then_some(Cause::Failed))
    }

    /// One try at a new device for `cause`, then the windows on it or on their own; a failed try asks Qt's loop for
    /// the next one, or makes it at once when nothing can time it. A frame off the GUI thread ends sharing for the run.
    fn attempt(&mut self, hub: &Hub, cause: Cause) -> After {
        let s = hub.settings();
        self.shares &= cause != Cause::OFF_THREAD;
        let mut again = Again {
            pick: self.pick,
            shares: self.shares,
        };
        loop {
            match self.rebuilds.next(s.device_rebuilds, cause, &mut again) {
                Next::Shared(device) => return self.reload(Some(device)),
                Next::Own => return self.reload(None),
                Next::End => return After::Failed,
                Next::Later if self.watch.retry(s.device_retry_ms) => {
                    self.waiting = Some(cause);
                    return After::Wait;
                }
                Next::Later => {}
            }
        }
    }

    /// The windows again over `device`, or over their own with `None`.
    fn reload(&mut self, device: Option<Device>) -> After {
        self.device = device;
        if self.load() {
            After::Again
        } else {
            After::Failed
        }
    }

    /// Loads the theme's fonts, as a wake lost meanwhile may have changed it, then the root QML into a new engine that
    /// serves the icons and the soft shadows' adopter; true when Qt made its objects. Loading from resources is
    /// synchronous.
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
        serve_icons(engine.as_mut());
        serve_clips(engine.as_mut());
        serve_shadows(engine.as_mut());
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

/// Makes a lost device again through Qt, on the adapter `pick` finds, and makes none while sharing is off.
struct Again {
    pick: Pick,
    shares: bool,
}

impl Remake for Again {
    type Device = Device;

    fn repin(&mut self) {
        match (self.pick)() {
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
        self.shares.then(Device::pinned).flatten()
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
