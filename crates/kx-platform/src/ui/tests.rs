//! The UI ports' shared types: the watch guard, the focus token, the default look and the errors.

use super::*;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

#[test]
fn a_watch_guard_stops_once_when_dropped() {
    let stops = Arc::new(AtomicUsize::new(0));
    let count = Arc::clone(&stops);
    let guard = WatchGuard::new(move || {
        count.fetch_add(1, Ordering::SeqCst);
    });
    assert_eq!(stops.load(Ordering::SeqCst), 0);
    drop(guard);
    assert_eq!(stops.load(Ordering::SeqCst), 1);
}

/// Compiles only for types a `Send + Sync` service may hold.
fn shared<T: Send + Sync>() {}

#[test]
fn a_watch_guard_can_live_in_a_shared_service() {
    shared::<WatchGuard>();
}

#[test]
fn a_focus_token_keeps_its_raw_value() {
    assert_eq!(FocusToken::new(u64::MAX).raw(), u64::MAX);
}

#[test]
fn the_default_look_is_light_with_nothing_set_and_the_usual_pointer() {
    let look = SysLook::default();
    assert!(!look.dark && !look.anim_off);
    assert!(look.accent.is_none() && look.contrast.is_none());
    assert_eq!(look.pointer_px, SysLook::POINTER_PX);
    assert_eq!(SysLook::POINTER_PX, 32, "Windows' default pointer size");
}

#[test]
fn errors_say_what_happened() {
    assert!(Running.to_string().contains("running"));
    assert!(UiError::Refused(5).to_string().contains('5'));
}
