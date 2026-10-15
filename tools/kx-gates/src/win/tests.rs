//! Tests of the window helpers that need no window of their own.

use super::*;

#[test]
fn poll_until_returns_the_first_value_at_once() {
    let mut calls = 0;
    let got = poll_until(0, 0, || {
        calls += 1;
        Some(7)
    });
    assert_eq!((got, calls), (Some(7), 1));
}

#[test]
fn a_window_that_does_not_exist_reads_as_nothing() {
    let none = HWND::default();
    assert_eq!((title(none), class(none)), (String::new(), String::new()));
    assert_eq!(program_name(none), None);
}

#[test]
fn ms_since_counts_up_from_its_start() {
    let t = Instant::now();
    assert!(ms_since(t) <= ms_since(t));
}

#[test]
fn poll_until_gives_up_after_the_timeout_without_waiting() {
    let mut calls = 0;
    let got = poll_until::<()>(0, u64::MAX, || {
        calls += 1;
        None
    });
    assert_eq!((got, calls), (None, 1));
}
