//! Fake UI ports on one full-HD screen, shared by the app's test binaries.

use kx_module_api::ports::{BASE_DPI, Point, Rect, Screen};
use kx_platform_fake::{FakeBackdrop, FakeScreens, FakeSystemLook, FakeTray, FakeWindowGuard};
use kx_ui::UiPorts;
use std::sync::Arc;

/// The fake tray's picture side.
const TRAY_PX: u32 = 16;

/// Fake UI ports on one full-HD screen.
pub fn ports() -> UiPorts {
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
        tray: Arc::new(FakeTray::new(TRAY_PX)),
    }
}
