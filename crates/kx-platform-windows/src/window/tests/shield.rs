//! The shield on hidden windows of the test thread: kept out of captures and let back in by title, never shown; and
//! on scripted windows, none left half shielded when one refuses.

use kx_platform::ui::{UiError, WindowGuard};
use windows::Win32::Foundation::HWND;
use windows::Win32::UI::WindowsAndMessaging::{
    GetWindowDisplayAffinity, WDA_EXCLUDEFROMCAPTURE, WDA_NONE, WINDOW_DISPLAY_AFFINITY,
};

use super::super::WinGuard;
use super::super::shield::all_or_none;
use super::Hidden;

/// A scripted refusal's code.
const REFUSED: UiError = UiError::Refused(5);

/// Shields or lets in the scripted windows `ours` while `on`, refusing `refuse`; answers the result and every call.
fn scripted(
    ours: &[u8],
    on: bool,
    refuse: (u8, WINDOW_DISPLAY_AFFINITY),
) -> (Result<(), UiError>, Vec<(u8, u32)>) {
    let mut calls = Vec::new();
    let set = |w: u8, affinity: WINDOW_DISPLAY_AFFINITY| {
        calls.push((w, affinity.0));
        if (w, affinity) == refuse {
            Err(REFUSED)
        } else {
            Ok(())
        }
    };
    let result = all_or_none(ours, on, set);
    (result, calls)
}

#[test]
fn when_one_window_refuses_the_shield_every_one_is_let_back_in() {
    let (on, off) = (WDA_EXCLUDEFROMCAPTURE.0, WDA_NONE.0);
    let (result, calls) = scripted(&[1, 2, 3], true, (2, WDA_EXCLUDEFROMCAPTURE));
    assert_eq!(result, Err(REFUSED));
    assert_eq!(calls, [(1, on), (2, on), (1, off), (2, off), (3, off)]);
}

#[test]
fn a_refused_let_in_still_tries_every_window() {
    let off = WDA_NONE.0;
    let (result, calls) = scripted(&[1, 2, 3], false, (2, WDA_NONE));
    assert_eq!(result, Err(REFUSED));
    assert_eq!(calls, [(1, off), (2, off), (1, off), (2, off), (3, off)]);
}

#[test]
fn when_every_window_takes_the_shield_none_is_set_twice() {
    let on = WDA_EXCLUDEFROMCAPTURE.0;
    let (result, calls) = scripted(&[1, 2], true, (0, WDA_NONE));
    assert_eq!((result, calls), (Ok(()), vec![(1, on), (2, on)]));
}

/// The capture affinity Windows keeps for `hwnd`.
fn affinity(hwnd: HWND) -> u32 {
    let mut now = 0;
    // SAFETY: a plain query of the test's own window into a local.
    unsafe { GetWindowDisplayAffinity(hwnd, &raw mut now) }.unwrap();
    now
}

#[test]
fn a_hidden_window_of_ours_is_shielded_by_title_and_let_back_in() {
    let title = "KeyXtend test: shield";
    let window = Hidden::new(title);
    assert_eq!(WinGuard.shield(title, true), Ok(()));
    assert_eq!(affinity(window.0), WDA_EXCLUDEFROMCAPTURE.0);
    assert_eq!(WinGuard.shield(title, true), Ok(()), "twice is harmless");
    assert_eq!(WinGuard.shield(title, false), Ok(()));
    assert_eq!(affinity(window.0), WDA_NONE.0);
}

#[test]
fn every_window_of_ours_with_the_title_is_shielded_and_no_other() {
    let title = "KeyXtend test: shield both";
    let (one, two) = (Hidden::new(title), Hidden::new(title));
    let other = Hidden::new("KeyXtend test: shield other");
    assert_eq!(WinGuard.shield(title, true), Ok(()));
    let shielded = [&one, &two, &other].map(|w| affinity(w.0) == WDA_EXCLUDEFROMCAPTURE.0);
    assert_eq!(shielded, [true, true, false]);
}

#[test]
fn a_title_with_no_window_of_ours_is_no_window() {
    let none = WinGuard.shield("KeyXtend test: no such window", true);
    assert_eq!(none, Err(UiError::NoWindow));
}
