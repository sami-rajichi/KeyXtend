//! How the shared graphics device is made (ADR-0021), apart from Qt: at once on the pinned adapter, or on Qt's default
//! when that adapter is gone, with one warning.

#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    reason = "these helpers run only in tests, where a failed setup should fail the test"
)]

#[path = "hub/logs.rs"]
mod logs;

use kx_ui::device::share;
use logs::warnings_of;
use std::cell::Cell;

/// What the fake maker makes.
const DEVICE: u8 = 7;

/// A maker that fails `fails` times, then makes `DEVICE`, and counts its tries.
fn maker(fails: u32, tries: &Cell<u32>) -> impl FnMut() -> Option<u8> + '_ {
    move || {
        tries.set(tries.get() + 1);
        (tries.get() > fails).then_some(DEVICE)
    }
}

#[test]
fn a_device_made_at_once_keeps_the_pin_and_warns_of_nothing() {
    let tries = Cell::new(0);
    let unpinned = Cell::new(false);
    let mut made = None;
    let warned = warnings_of(|| {
        made = share(maker(0, &tries), || {
            unpinned.set(true);
            true
        });
    });
    assert_eq!(made, Some(DEVICE));
    assert_eq!(tries.get(), 1);
    assert!(!unpinned.get() && warned.is_empty(), "{warned:?}");
}

#[test]
fn a_gone_pinned_adapter_is_dropped_and_the_device_made_on_the_default_with_one_warning() {
    let tries = Cell::new(0);
    let mut made = None;
    let warned = warnings_of(|| made = share(maker(1, &tries), || true));
    assert_eq!(made, Some(DEVICE));
    assert_eq!(tries.get(), 2, "one try on the pin, one on the default");
    assert_eq!(warned.len(), 1, "{warned:?}");
    assert!(warned[0].contains("default"), "{warned:?}");
}
