//! Fake UI ports on one full-HD screen, shared by the app's test binaries and its unit tests.

use kx_module_api::ports::{BASE_DPI, Point, Rect, Screen};
use kx_platform_fake::{
    FakeBackdrop, FakeMemory, FakeScreens, FakeSystemLook, FakeTray, FakeWindowGuard,
};
use kx_ui::UiPorts;
use std::sync::Arc;

/// The fake tray's picture side.
const TRAY_PX: u32 = 16;

/// A fake tray with no icon.
pub fn tray() -> Arc<FakeTray> {
    Arc::new(FakeTray::new(TRAY_PX))
}

/// Fake UI ports on one full-HD screen, over `tray`.
pub fn ports(tray: Arc<FakeTray>) -> UiPorts {
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
    UiPorts {
        guard: Arc::new(FakeWindowGuard::new(0)),
        screens: Arc::new(FakeScreens::new(vec![screen], Point { x: 0, y: 0 })),
        look: Arc::new(FakeSystemLook::new()),
        backdrop: Arc::new(FakeBackdrop::new()),
        tray,
        memory: Arc::new(FakeMemory::new()),
    }
}
