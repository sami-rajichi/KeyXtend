//! The frozen screens laid under our other windows (ADR-0059), on hidden windows only: a window made last lies on top
//! of the band, as Qt makes each new frozen screen, so the bar, the bubble and the keyboard go up over it.

use std::cell::Cell;
use std::sync::PoisonError;

use kx_platform::ui::{UiError, WindowGuard};
use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::UI::WindowsAndMessaging::{
    CallWindowProcW, GWLP_WNDPROC, SetWindowLongPtrW, WM_WINDOWPOSCHANGING,
};

use super::super::guard::{Proc, next};
use super::super::top::{front, lay_low_with};
use super::z_order::{Z_ORDER, above, topmost};
use super::{Hidden, WinGuard};

/// The test's frozen screens.
const SCREEN: &str = "KeyXtend test: frozen screen";

thread_local! {
    /// The moves `counted` heard on this thread.
    static MOVES: Cell<usize> = const { Cell::new(0) };
    /// The procedure `counted` hands messages to.
    static BEHIND: Cell<isize> = const { Cell::new(0) };
}

/// A subclass that counts each move Windows asks of its window, then hands the message on.
unsafe extern "system" fn counted(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    if msg == WM_WINDOWPOSCHANGING {
        MOVES.set(MOVES.get() + 1);
    }
    let behind = BEHIND.get().cast_unsigned();
    // SAFETY: hands the message to the procedure this subclass replaced, as a subclass must.
    unsafe { CallWindowProcW(next(behind), hwnd, msg, wparam, lparam) }
}

/// Puts `counted` in front of `window`'s own procedure.
fn count(window: &Hidden) {
    let ours: Proc = counted;
    // SAFETY: puts the test subclass in front on the test's own window; it hands every message on.
    BEHIND.set(unsafe { SetWindowLongPtrW(window.0, GWLP_WNDPROC, (ours as usize).cast_signed()) });
}

#[test]
fn a_frozen_screen_made_last_lies_over_ours_until_laid_under_them_in_their_order() {
    let _z = Z_ORDER.lock().unwrap_or_else(PoisonError::into_inner);
    let bar = Hidden::topmost("KeyXtend test: low bar", None);
    let bubble = Hidden::topmost("KeyXtend test: low bubble", None);
    let screen = Hidden::topmost(SCREEN, None);
    assert!(
        above(&screen, &bar) && above(&screen, &bubble),
        "made last, it is on top"
    );
    assert!(above(&bubble, &bar));
    assert_eq!(WinGuard.lay_low(SCREEN), Ok(()));
    assert!(above(&bar, &screen) && above(&bubble, &screen));
    assert!(above(&bubble, &bar), "ours keep their order");
    assert!(topmost(&screen) && topmost(&bar), "all stay in the band");
}

/// The bar is a shadow that owns its window, both titled as the bar; each goes up alone, the window above its shadow.
#[test]
fn an_owned_bar_and_its_shadow_both_go_over_the_frozen_screen() {
    let _z = Z_ORDER.lock().unwrap_or_else(PoisonError::into_inner);
    let shadow = Hidden::tip("KeyXtend test: low bar pair", None);
    let bar = Hidden::topmost("KeyXtend test: low bar pair", Some(shadow.0));
    let screen = Hidden::topmost(SCREEN, None);
    assert!(above(&screen, &bar) && above(&bar, &shadow));
    assert_eq!(WinGuard.lay_low(SCREEN), Ok(()));
    assert!(above(&shadow, &screen), "the shadow");
    assert!(above(&bar, &shadow), "the bar over its shadow");
}

/// The lifts go back to front, so the bar is tried first; Windows refusing it must still let the bubble go up.
#[test]
fn a_refused_lift_leaves_the_rest_to_go_over_the_frozen_screen_and_its_error_is_kept() {
    let _z = Z_ORDER.lock().unwrap_or_else(PoisonError::into_inner);
    let bar = Hidden::topmost("KeyXtend test: low refused bar", None);
    let bubble = Hidden::topmost("KeyXtend test: low refused bubble", None);
    let screen = Hidden::topmost(SCREEN, None);
    let refusal = UiError::Refused(5);
    let lift = |w: HWND| if w == bar.0 { Err(refusal) } else { front(w) };
    assert_eq!(
        lay_low_with(SCREEN, lift),
        Err(refusal),
        "the refusal is kept"
    );
    assert!(
        above(&bubble, &screen),
        "the lift after the refusal still ran"
    );
    assert!(above(&screen, &bar), "the refused one stays");
}

#[test]
fn every_frozen_screen_goes_under_and_a_window_off_the_band_stays_off_it() {
    let _z = Z_ORDER.lock().unwrap_or_else(PoisonError::into_inner);
    let settings = Hidden::new("KeyXtend test: low settings");
    let bar = Hidden::topmost("KeyXtend test: low bar two", None);
    let (one, two) = (Hidden::topmost(SCREEN, None), Hidden::topmost(SCREEN, None));
    assert_eq!(WinGuard.lay_low(SCREEN), Ok(()));
    assert!(above(&bar, &one) && above(&bar, &two));
    assert!(above(&two, &one), "the screens keep their order");
    assert!(
        !topmost(&settings),
        "Settings is never lifted into the band"
    );
}

/// The thread's own IME window, which Windows makes in the band beside the first one here, is not ours to lift.
#[test]
fn an_order_that_holds_moves_no_window() {
    let _z = Z_ORDER.lock().unwrap_or_else(PoisonError::into_inner);
    let screen = Hidden::topmost(SCREEN, None);
    let bar = Hidden::topmost("KeyXtend test: low bar three", None);
    count(&bar);
    MOVES.set(0);
    assert_eq!(WinGuard.lay_low(SCREEN), Ok(()));
    assert_eq!(MOVES.get(), 0, "nothing to lift");
    assert!(above(&bar, &screen));
}

#[test]
fn a_frozen_screen_of_another_thread_or_none_is_no_window() {
    let _screen = Hidden::topmost(SCREEN, None);
    let elsewhere = std::thread::spawn(|| WinGuard.lay_low(SCREEN)).join();
    assert_eq!(
        elsewhere.unwrap(),
        Err(UiError::NoWindow),
        "only the UI thread's"
    );
    let none = WinGuard.lay_low("KeyXtend test: no frozen screen");
    assert_eq!(none, Err(UiError::NoWindow));
}
