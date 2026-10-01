//! The stop after Qt's loop: whatever state the kernel's thread is in, the tray icon goes and the thread is joined.

use crate::rig::{Rig, watch};
use keyxtend::drain::{self, stop};
use kx_platform::ui::TrayImage;
use kx_ui::Wake;
use std::sync::Arc;
use std::thread;

/// A one-pixel picture for the tray icon.
fn picture() -> TrayImage {
    TrayImage::new(1, 1, vec![u8::MAX; 4]).unwrap()
}

#[test]
fn a_running_kernel_thread_quits_and_the_icon_goes() {
    let rig = Rig::new();
    let mut boot = rig.load();
    boot.start().unwrap();
    let wakes = watch(&boot.hub);
    assert!(boot.hub.tray_show(&picture()) && !rig.tray.shown().is_empty());
    let hub = Arc::clone(&boot.hub);
    let kernel = drain::spawn(boot.kernel, Arc::clone(&hub), boot.requests).unwrap();
    assert!(stop(&hub, kernel), "the thread was joined");
    assert!(rig.tray.shown().is_empty(), "the icon went");
    assert!(wakes.lock().unwrap().contains(&Wake::End), "the UI ended");
}

#[test]
fn a_quit_sent_already_still_stops_cleanly() {
    let rig = Rig::new();
    let mut boot = rig.load();
    boot.start().unwrap();
    assert!(boot.hub.tray_show(&picture()));
    let hub = Arc::clone(&boot.hub);
    let kernel = drain::spawn(boot.kernel, Arc::clone(&hub), boot.requests).unwrap();
    assert!(hub.quit(), "the tray's Quit came first");
    assert!(stop(&hub, kernel));
    assert!(rig.tray.shown().is_empty());
}

#[test]
fn a_kernel_thread_that_panicked_is_joined_and_the_icon_goes() {
    let rig = Rig::new();
    let boot = rig.load();
    assert!(boot.hub.tray_show(&picture()));
    let kernel = thread::spawn(|| panic!("a module would not stop"));
    assert!(!stop(&boot.hub, kernel), "the panic is reported");
    assert!(rig.tray.shown().is_empty());
}
