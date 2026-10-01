use super::*;
use kx_module_api::ports::{BASE_DPI, Point, Rect, Screen};
use kx_platform_fake::{FakeBackdrop, FakeScreens, FakeSystemLook, FakeTray, FakeWindowGuard};
use kx_ui::{UiPorts, Wake};
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::sync::Mutex;

/// The fake tray's picture side.
const TRAY_PX: u32 = 16;

/// A hub over fresh fakes, with a waker that keeps every wake.
fn hub() -> (Arc<Hub>, Arc<Mutex<Vec<Wake>>>) {
    let screen = Screen {
        rect: Rect {
            x: 0,
            y: 0,
            w: 1920,
            h: 1040,
        },
        dpi: BASE_DPI,
        primary: true,
    };
    let ports = UiPorts {
        guard: Arc::new(FakeWindowGuard::new(0)),
        screens: Arc::new(FakeScreens::new(vec![screen], Point { x: 0, y: 0 })),
        look: Arc::new(FakeSystemLook::new()),
        backdrop: Arc::new(FakeBackdrop::new()),
        tray: Arc::new(FakeTray::new(TRAY_PX)),
    };
    let (hub, _requests) = Hub::new(ports).unwrap();
    let wakes = Arc::new(Mutex::new(Vec::new()));
    let seen = Arc::clone(&wakes);
    hub.set_waker(Box::new(move |w| seen.lock().unwrap().push(w)));
    (hub, wakes)
}

#[test]
fn a_panic_while_stopping_still_ends_the_ui_once() {
    let (hub, wakes) = hub();
    let stopped = catch_unwind(AssertUnwindSafe(|| {
        let _end = EndUi(&hub);
        panic!("a module would not stop");
    }));
    assert!(stopped.is_err());
    assert_eq!(*wakes.lock().unwrap(), [Wake::End]);
}
