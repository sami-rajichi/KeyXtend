//! The watch hooks and ends with its guard; a press is told once, only after the hook counted it, and a callback that
//! panics still hears the next. No test sends a key, so the owner's apps hear nothing.

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, mpsc};

use super::*;
use crate::hook::{KEY_DOWNS, KEY_UPS, VK_ESCAPE};
use crate::keys::TAG;

/// The hand's marks: none.
const HAND: usize = 0;

/// Sets this thread's watch state with `on`, as a watch thread's setup does.
fn set(on: OnEsc) {
    let _ = HERE.with(|h| h.set(here(on)));
}

/// The hook's work for one press of the hand's Esc, posted.
fn press() {
    if heard(KEY_DOWNS[0], VK_ESCAPE, HAND) {
        count();
    }
}

#[test]
fn a_watch_hooks_and_ends_with_its_guard_which_lets_its_callback_go() {
    let calls = Arc::new(AtomicUsize::new(0));
    let counted = Arc::clone(&calls);
    let on: OnEsc = Box::new(move || {
        counted.fetch_add(1, Ordering::SeqCst);
    });
    let guard = WinEscWatch::new().watch(on).unwrap();
    drop(guard);
    assert_eq!(
        Arc::strong_count(&calls),
        1,
        "the callback went with its thread"
    );
}

#[test]
fn only_a_press_the_hook_counted_is_told_and_only_once() {
    let (tx, rx) = mpsc::channel();
    set(Box::new(move || {
        let _ = tx.send(());
    }));
    tell();
    assert!(
        rx.try_recv().is_err(),
        "a post from elsewhere tells nothing"
    );
    press();
    press();
    tell();
    tell();
    assert_eq!(rx.try_iter().count(), 1, "a held key's repeat is no press");
}

#[test]
fn a_callback_that_panics_still_hears_the_next_press() {
    let (tx, rx) = mpsc::channel();
    let calls = AtomicUsize::new(0);
    set(Box::new(move || {
        let first = calls.fetch_add(1, Ordering::SeqCst) == 0;
        assert!(!first, "a callback bug");
        let _ = tx.send(());
    }));
    for _ in 0..2 {
        press();
        tell();
        assert!(
            !heard(KEY_DOWNS[0], VK_ESCAPE, TAG),
            "our own input is no press"
        );
        assert!(!heard(KEY_UPS[0], VK_ESCAPE, HAND), "a release is no press");
    }
    assert_eq!(rx.try_iter().count(), 1, "the second press was told");
}
