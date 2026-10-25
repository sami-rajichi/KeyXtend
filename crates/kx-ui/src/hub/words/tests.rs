//! The handle the hub keeps for chip taps holds only the `ui` module's grants, so the bus's gate holds for the UI too.

use super::super::{Hub, UiPorts};
use crate::ui_module::MANIFEST;
use kx_module_api::ports::{BASE_DPI, ChipPicked, Point, Rect, Screen};
use kx_module_api::{Bus, Capability, Event, EventError};
use kx_platform_fake::{
    FakeBackdrop, FakeMemory, FakeScreens, FakeSpeaker, FakeSystemLook, FakeTray, FakeWindowGuard,
};
use kx_test_support::host::FakeCore;
use std::sync::Arc;

/// A capability the `ui` module is never granted.
const OUTSIDE: Capability = Capability::InjectInput;

/// An event gated by `OUTSIDE`.
struct Outside;

impl Event for Outside {
    const NAME: &'static str = "outside";
    const CAPABILITY: Option<Capability> = Some(OUTSIDE);
}

/// A hub over fake ports on one plain screen.
fn hub() -> Arc<Hub> {
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
    let ports = UiPorts {
        guard: Arc::new(FakeWindowGuard::new(0)),
        screens: Arc::new(FakeScreens::new(vec![screen], Point { x: 0, y: 0 })),
        look: Arc::new(FakeSystemLook::new()),
        backdrop: Arc::new(FakeBackdrop::new()),
        tray: Arc::new(FakeTray::new(1)),
        memory: Arc::new(FakeMemory::new()),
        speaker: Arc::new(FakeSpeaker::new()),
    };
    Hub::new(ports).expect("the hub starts").0
}

#[test]
fn the_kept_handle_refuses_an_event_outside_the_ui_grants() {
    assert!(
        !MANIFEST.capabilities.contains(&OUTSIDE),
        "the ui never holds it"
    );
    let hub = hub();
    let bus = Bus::new(Arc::new(FakeCore::default()));
    assert_eq!(
        bus.check::<Outside>(),
        Ok(()),
        "the kernel's handle has no limit"
    );
    let _heard = hub.hear_words(&bus).expect("the lane is heard");
    let kept = hub.words().bus.clone().expect("the handle is kept");
    let refused = EventError::NotGranted(Outside::NAME, OUTSIDE);
    assert_eq!(kept.check::<Outside>(), Err(refused));
    assert_eq!(kept.check::<ChipPicked>(), Ok(()), "its own taps go");
}
