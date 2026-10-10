//! The shared graphics device's glue (ADR-0021): the `QRhi` every Qt Quick window adopts before its first frame,
//! whether it is lost, and which windows draw on it.

use super::harden::unpin_adapter;
use crate::device::share;
use cxx::UniquePtr;

#[cxx_qt::bridge]
mod ffi {
    // SAFETY: `device.h` declares exactly these functions and `Device` members with these types. Every call runs on the
    // Qt GUI thread, which owns the device and the windows, and the app drops every window before the device.
    unsafe extern "C++" {
        include!("kx-ui/cpp/device.h");

        /// The shared `QRhi` and the filter that hands it to each window.
        #[namespace = "kxui"]
        type Device;

        /// A device on the adapter Qt's variables pin; null when none can be made.
        #[namespace = "kxui"]
        fn device_new() -> UniquePtr<Device>;

        /// True when the device is gone.
        fn lost(self: &Device) -> bool;

        /// How many Qt Quick windows draw on the device.
        fn windows(self: &Device) -> usize;

        /// True while every frame on the device has ended on the GUI thread.
        fn gui_only(self: &Device) -> bool;

        /// How many Qt Quick windows draw, on any device.
        #[namespace = "kxui"]
        fn drawing_windows() -> usize;

        /// How many devices those windows draw on.
        #[namespace = "kxui"]
        fn drawing_devices() -> usize;

        /// Deletes the objects that wait for a later delete, so no window outlives its device.
        #[namespace = "kxui"]
        fn flush_deletes();
    }
}

/// The `QRhi` every Qt Quick window shares; drop it only after the windows that draw on it.
pub struct Device(UniquePtr<ffi::Device>);

impl Device {
    /// The shared device on the pinned adapter, or on Qt's default when that adapter is gone; `None` when none can be
    /// made, so each window makes its own. Call it on the GUI thread once Qt's app object exists, before any window.
    #[must_use]
    pub fn make() -> Option<Self> {
        let made = || Some(ffi::device_new()).filter(|d| !d.is_null()).map(Self);
        share(made, unpin_adapter)
    }

    /// True when the device is gone, so the windows must be built again.
    #[must_use]
    pub fn lost(&self) -> bool {
        self.0.as_ref().is_some_and(ffi::Device::lost)
    }

    /// How many Qt Quick windows draw on the device.
    #[must_use]
    pub fn windows(&self) -> usize {
        self.0.as_ref().map_or(0, ffi::Device::windows)
    }

    /// True while every frame on the device has ended on the GUI thread, as one device needs.
    #[must_use]
    pub fn gui_only(&self) -> bool {
        self.0.as_ref().is_some_and(ffi::Device::gui_only)
    }
}

/// How many Qt Quick windows draw, and on how many devices.
#[must_use]
pub fn drawing() -> (usize, usize) {
    (ffi::drawing_windows(), ffi::drawing_devices())
}

/// Deletes the objects that wait for a later delete, as windows a dropped engine left.
pub(crate) fn flush_deletes() {
    ffi::flush_deletes();
}
