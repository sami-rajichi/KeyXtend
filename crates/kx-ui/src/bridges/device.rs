//! The shared graphics device's glue (ADR-0021): the `QRhi` every Qt Quick window adopts before its first frame,
//! whether a frame found it lost or drew off the GUI thread, the watch for a window that could not start drawing, and
//! the release of Qt's cached graphics after a look change, which then gives the freed memory back (ADR-0023). Debug
//! builds can mark the device lost or a window failed, and count which windows draw on it, for tests.

use super::harden::unpin_adapter;
use super::installed;
use crate::device::share;
use cxx::UniquePtr;

#[cxx_qt::bridge]
mod ffi {
    // SAFETY: `device.h`, the `watch.h` it includes and `release.h` declare exactly these functions, types and members
    // with these types, and the function `release_after_frame` keeps is a plain Rust one, valid for the whole run.
    // `device_new` and `watch_new` make nothing off the thread of Qt's app object; every other call runs on that GUI
    // thread, which owns the device, the watch and the windows, and the app drops every window before the device.
    unsafe extern "C++" {
        include!("kx-ui/cpp/device.h");
        include!("kx-ui/cpp/release.h");

        /// The shared `QRhi` and the filter that hands it to each window.
        #[namespace = "kxui"]
        type Device;

        /// A device on the adapter Qt's variables pin; null before Qt's app object, off its thread, or when none can
        /// be made.
        #[namespace = "kxui"]
        fn device_new() -> UniquePtr<Device>;

        /// True once a frame found the device gone or ran off the GUI thread; that frame asked Qt's loop to end.
        fn lost(self: &Device) -> bool;

        /// True while every frame on the device has ended on the GUI thread.
        fn gui_only(self: &Device) -> bool;

        /// How many Qt Quick windows draw on the device.
        #[cfg(debug_assertions)]
        fn windows(self: &Device) -> usize;

        /// Marks the device gone and asks each window for a frame, as a driver reset would.
        #[cfg(debug_assertions)]
        fn lose(self: Pin<&mut Device>);

        /// Has the next frame count as drawn off the GUI thread, and asks each window for one.
        #[cfg(debug_assertions)]
        fn stray(self: Pin<&mut Device>);

        /// How many Qt Quick windows draw, on any device.
        #[cfg(debug_assertions)]
        #[namespace = "kxui"]
        fn drawing_windows() -> usize;

        /// How many devices those windows draw on.
        #[cfg(debug_assertions)]
        #[namespace = "kxui"]
        fn drawing_devices() -> usize;

        /// How many of those windows have `sceneGraphError` connected.
        #[cfg(debug_assertions)]
        #[namespace = "kxui"]
        fn wired_windows() -> usize;

        /// Has each shown window drop Qt's cached graphics once its next frame ends; once none waits, `then` runs.
        #[namespace = "kxui"]
        fn release_after_frame(then: fn());

        /// How many windows dropped Qt's cached graphics after a frame so far.
        #[cfg(debug_assertions)]
        #[namespace = "kxui"]
        fn released_windows() -> usize;

        /// Deletes the objects that wait for a later delete, so no window outlives its device.
        #[namespace = "kxui"]
        fn flush_deletes();

        /// True on the thread of Qt's app object, once it exists.
        #[namespace = "kxui"]
        fn on_app_thread() -> bool;

        /// Watches each Qt Quick window for a scene graph that could not start, and times the next try at a device.
        #[namespace = "kxui"]
        type Watch;

        /// A watch over every Qt Quick window; null before Qt's app object, or off its thread.
        #[namespace = "kxui"]
        fn watch_new() -> UniquePtr<Watch>;

        /// True once a window's scene graph could not start since the last call, which clears it.
        fn take_failed(self: Pin<&mut Watch>) -> bool;

        /// Ends Qt's loop after `ms`, for the next try at a device.
        fn retry(self: Pin<&mut Watch>, ms: u64);

        /// True once the timer of the last `retry` ran out.
        fn ready(self: &Watch) -> bool;

        /// Has a window report a scene graph that could not start, as a failed device would.
        #[cfg(debug_assertions)]
        fn fail(self: Pin<&mut Watch>);
    }
}

/// Watches each Qt Quick window for a scene graph that could not start, which Qt would otherwise end the app for, and
/// times the next try at a new device while Qt's loop runs.
pub(crate) struct Watch(UniquePtr<ffi::Watch>);

impl Watch {
    /// A watch over every Qt Quick window; it watches nothing before Qt's app object, or off its thread.
    pub(crate) fn new() -> Self {
        Self(ffi::watch_new())
    }

    /// True once a window's scene graph could not start since the last call, which clears it.
    pub(crate) fn take_failed(&mut self) -> bool {
        self.0.as_mut().is_some_and(ffi::Watch::take_failed)
    }

    /// Ends Qt's loop after `ms`, for the next try at a device, while the loop runs; false when nothing can time it.
    pub(crate) fn retry(&mut self, ms: u64) -> bool {
        self.0.as_mut().map(|w| w.retry(ms)).is_some()
    }

    /// True once the time of the last `retry` ran out.
    pub(crate) fn ready(&self) -> bool {
        self.0.as_ref().is_some_and(ffi::Watch::ready)
    }

    /// Has a window report a scene graph that could not start; debug builds only, for tests.
    #[cfg(debug_assertions)]
    pub(crate) fn fail(&mut self) {
        if let Some(watch) = self.0.as_mut() {
            watch.fail();
        }
    }
}

/// The `QRhi` every Qt Quick window shares; drop it only after the windows that draw on it.
pub struct Device(UniquePtr<ffi::Device>);

impl Device {
    /// The shared device on the pinned adapter, or on Qt's default when that adapter is gone; `None` when none can be
    /// made, so each window makes its own. Call it on the GUI thread once Qt's app object exists, before any window;
    /// elsewhere it makes nothing and leaves the pin.
    #[must_use]
    pub fn make() -> Option<Self> {
        if !ffi::on_app_thread() {
            return None;
        }
        share(Self::pinned, unpin_adapter)
    }

    /// The shared device on the adapter Qt's variables pin, if any, and nowhere else; `None` when none can be made,
    /// before Qt's app object exists, or off its thread.
    pub(crate) fn pinned() -> Option<Self> {
        Some(ffi::device_new()).filter(|d| !d.is_null()).map(Self)
    }

    /// True once a frame found the device gone or drawing off the GUI thread, so the windows must be made again; it
    /// asked Qt's loop to end.
    #[must_use]
    pub fn lost(&self) -> bool {
        self.0.as_ref().is_some_and(ffi::Device::lost)
    }

    /// True while every frame on the device has ended on the GUI thread, as one device needs.
    #[must_use]
    pub fn gui_only(&self) -> bool {
        self.0.as_ref().is_some_and(ffi::Device::gui_only)
    }

    /// How many Qt Quick windows draw on the device; debug builds only, for tests.
    #[cfg(debug_assertions)]
    #[must_use]
    pub fn windows(&self) -> usize {
        self.0.as_ref().map_or(0, ffi::Device::windows)
    }

    /// Marks the device gone, as a driver reset would, so the next frame finds it; debug builds only, for tests.
    #[cfg(debug_assertions)]
    pub fn lose(&mut self) {
        if let Some(device) = self.0.as_mut() {
            device.lose();
        }
    }

    /// Has the next frame count as drawn off the GUI thread, which ends sharing; debug builds only, for tests.
    #[cfg(debug_assertions)]
    pub fn stray(&mut self) {
        if let Some(device) = self.0.as_mut() {
            device.stray();
        }
    }
}

/// How many Qt Quick windows draw, and on how many devices; debug builds only, for tests.
#[cfg(debug_assertions)]
#[must_use]
pub fn drawing() -> (usize, usize) {
    (ffi::drawing_windows(), ffi::drawing_devices())
}

/// How many drawing Qt Quick windows have `sceneGraphError` connected, so Qt cannot end the app over it; debug builds
/// only, for tests.
#[cfg(debug_assertions)]
#[must_use]
pub fn wired() -> usize {
    ffi::wired_windows()
}

/// Has each shown Qt Quick window drop Qt's cached graphics, such as the last look's pipelines and upload data, once
/// its next frame ends, then gives the freed heap pages back (ADR-0023); call it on the GUI thread after a look change.
pub(crate) fn release_after_frame() {
    ffi::release_after_frame(settled);
}

/// Every window dropped what it cached for the last look, so the hub gives the freed memory back.
fn settled() {
    if let Some(hub) = installed() {
        hub.decommit();
    }
}

/// How many windows dropped Qt's cached graphics after a frame so far; debug builds only, for tests.
#[cfg(debug_assertions)]
#[must_use]
pub fn released() -> usize {
    ffi::released_windows()
}

/// Deletes the objects that wait for a later delete, as windows a dropped engine left.
pub(crate) fn flush_deletes() {
    ffi::flush_deletes();
}
