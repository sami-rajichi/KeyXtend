//! The tests that read the z-order or the topmost style, on hidden windows only.

use std::sync::{Mutex, PoisonError};

use windows::Win32::Foundation::HWND;
use windows::Win32::UI::WindowsAndMessaging::{
    GWL_EXSTYLE, MA_NOACTIVATE, SendMessageW, SetWindowLongPtrW, WM_MOUSEACTIVATE, WS_EX_TOPMOST,
};

use super::super::KEPT;
use super::super::guard::{front, guard};
use super::super::win::{guard_these, top_level};
use super::Hidden;

/// Keeps these tests apart: one test's lift can drop another's hidden window.
static Z_ORDER: Mutex<()> = Mutex::new(());

/// True when window `a` is above window `b`, read from one snapshot of the z-order.
fn above(a: &Hidden, b: &Hidden) -> bool {
    let z = top_level();
    let at = |w: &Hidden| z.iter().position(|&x| x == w.0).unwrap();
    at(a) < at(b)
}

/// `windows` front first, as a guard run lists the shown ones.
fn listed(windows: &[&Hidden]) -> Vec<HWND> {
    let ours: Vec<_> = windows.iter().map(|w| w.0).collect();
    top_level()
        .into_iter()
        .filter(|w| ours.contains(w))
        .collect()
}

/// The window starts topmost, as Qt makes ours: guarding a plain hidden window can leave it below the topmost band.
#[test]
fn a_guarded_window_keeps_its_styles_and_never_activates() {
    let _z = Z_ORDER.lock().unwrap_or_else(PoisonError::into_inner);
    let window = Hidden::make("KeyXtend test: guarded", WS_EX_TOPMOST, None);
    guard(window.0).unwrap();
    assert_eq!(
        window.ex_style() & KEPT,
        KEPT,
        "no focus and no taskbar button"
    );
    // SAFETY: tries to clear the test window's extended style.
    unsafe { SetWindowLongPtrW(window.0, GWL_EXSTYLE, 0) };
    assert_eq!(window.ex_style() & KEPT, KEPT, "the guard put them back");
    // SAFETY: asks the test window how a click would activate it.
    let answer = unsafe { SendMessageW(window.0, WM_MOUSEACTIVATE, None, None) };
    assert_eq!(answer.0, isize::try_from(MA_NOACTIVATE).unwrap());
}

/// The tip bug: a hidden window shows again where it was, so only a tip its window owns stays above it after a lift.
#[test]
fn a_tip_stays_above_its_window_after_the_guard_lifts_it_only_when_owned() {
    let _z = Z_ORDER.lock().unwrap_or_else(PoisonError::into_inner);
    let kb = Hidden::topmost("KeyXtend test: tip owner", None);
    let owned = Hidden::tip("KeyXtend test: owned tip", Some(kb.0));
    let loose = Hidden::tip("KeyXtend test: loose tip", None);
    assert!(above(&loose, &kb), "a new tip starts above the keyboard");
    guard(kb.0).unwrap();
    assert!(above(&kb, &loose), "the lift covers an unowned tip");
    assert!(above(&owned, &kb), "an owned tip stays above it");
}

/// The guard lifts a banner or the list after the keyboard, so a top-bar tip shows under it until it is lifted alone,
/// which leaves the keyboard under the banner.
#[test]
fn a_tip_lifted_alone_passes_a_later_pop_up_and_leaves_its_window_below_it() {
    let _z = Z_ORDER.lock().unwrap_or_else(PoisonError::into_inner);
    let kb = Hidden::topmost("KeyXtend test: keyboard", None);
    let tip = Hidden::tip("KeyXtend test: bar tip", Some(kb.0));
    let banner = Hidden::topmost("KeyXtend test: banner", None);
    guard(kb.0).unwrap();
    guard(banner.0).unwrap();
    assert!(
        above(&banner, &tip),
        "the banner lifted last covers the tip"
    );
    front(tip.0).unwrap();
    assert!(above(&tip, &banner), "the tip comes in front of the banner");
    assert!(above(&banner, &kb), "the keyboard stays under the banner");
    assert!(above(&tip, &kb), "and under its tip");
}

/// A hidden pop-up shows again where it was, under the keyboard the guard lifted meanwhile; guarded as `top`, it lands
/// in front.
#[test]
fn a_pop_up_shown_again_lands_in_front_when_guarded_as_top() {
    let _z = Z_ORDER.lock().unwrap_or_else(PoisonError::into_inner);
    let menu = Hidden::topmost("KeyXtend test: menu", None);
    let kb = Hidden::topmost("KeyXtend test: menu's keyboard", None);
    guard(kb.0).unwrap();
    assert!(above(&kb, &menu), "the keyboard lifted while the menu hid");
    assert_eq!(guard_these(listed(&[&kb, &menu]), None, None), Ok(2));
    assert!(above(&kb, &menu), "a run with no top keeps the order");
    let top = Some("KeyXtend test: menu");
    assert_eq!(guard_these(listed(&[&kb, &menu]), None, top), Ok(2));
    assert!(above(&menu, &kb), "the menu lands in front");
}

/// A tip lifted alone in front of the banner must hide before a guard run: guarded with it, its keyboard comes along.
#[test]
fn a_guard_run_keeps_the_banner_above_the_keyboard_only_while_the_tip_hides() {
    let _z = Z_ORDER.lock().unwrap_or_else(PoisonError::into_inner);
    let kb = Hidden::topmost("KeyXtend test: run's keyboard", None);
    let tip = Hidden::tip("KeyXtend test: run's tip", Some(kb.0));
    let banner = Hidden::topmost("KeyXtend test: run's banner", None);
    guard(kb.0).unwrap();
    guard(banner.0).unwrap();
    front(tip.0).unwrap();
    guard_these(listed(&[&kb, &banner]), None, None).unwrap();
    assert!(above(&banner, &kb), "the tip hid: the banner stays above");
    front(tip.0).unwrap();
    guard_these(listed(&[&kb, &banner, &tip]), None, None).unwrap();
    assert!(
        above(&kb, &banner),
        "a shown tip pulls its keyboard over it"
    );
}
