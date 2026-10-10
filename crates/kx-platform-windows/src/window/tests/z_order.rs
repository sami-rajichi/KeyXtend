//! The tests that read the z-order or the topmost style, on hidden windows only.

use std::cell::Cell;
use std::sync::{Mutex, PoisonError};

use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::UI::WindowsAndMessaging::{
    GWL_EXSTYLE, GWLP_WNDPROC, GetWindowLongPtrW, HWND_BOTTOM, HWND_NOTOPMOST, MA_NOACTIVATE,
    SWP_NOACTIVATE, SWP_NOMOVE, SWP_NOSIZE, SendMessageW, SetWindowLongPtrW, SetWindowPos,
    WM_MOUSEACTIVATE, WM_WINDOWPOSCHANGED, WS_EX_TOPMOST,
};

use super::super::KEPT;
use super::super::guard::{Proc, front, guard};
use super::super::win::{guard_these, top_level};
use super::{Hidden, plain};

/// Keeps these tests apart: one test's lift can drop another's hidden window.
static Z_ORDER: Mutex<()> = Mutex::new(());

thread_local! {
    /// The window `dropper` drops once, as the guard lifts its own window back.
    static TO_DROP: Cell<Option<HWND>> = const { Cell::new(None) };
}

/// A subclass behind the guard that drops `TO_DROP` inside its window's lift, as the keyboard's lift moves its shadow.
/// It never panics, since a panic cannot leave a window procedure; a refused drop fails the test's checks instead.
unsafe extern "system" fn dropper(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    // SAFETY: a plain query of the test's own window.
    let ex = u32::try_from(unsafe { GetWindowLongPtrW(hwnd, GWL_EXSTYLE) }).unwrap_or_default();
    let lifted = msg == WM_WINDOWPOSCHANGED && ex & WS_EX_TOPMOST.0 != 0;
    if let Some(other) = lifted.then(|| TO_DROP.take()).flatten() {
        let flags = SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE;
        // SAFETY: a z-order change on the test's own hidden window.
        let _ = unsafe { SetWindowPos(other, Some(HWND_NOTOPMOST), 0, 0, 0, 0, flags) };
    }
    // SAFETY: hands the message to the test windows' own procedure.
    unsafe { plain(hwnd, msg, wparam, lparam) }
}

/// True when window `a` is above window `b`, read from one snapshot of the z-order.
fn above(a: &Hidden, b: &Hidden) -> bool {
    let z = top_level();
    let at = |w: &Hidden| z.iter().position(|&x| x == w.0).unwrap();
    at(a) < at(b)
}

/// Moves `window` after `after` in the z-order as another program could, without moving or activating it.
fn put_after(window: &Hidden, after: HWND) {
    let flags = SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE;
    // SAFETY: a z-order change on the test's own hidden window.
    unsafe { SetWindowPos(window.0, Some(after), 0, 0, 0, 0, flags) }.unwrap();
}

/// True while `window` is in the topmost band.
fn topmost(window: &Hidden) -> bool {
    window.ex_style() & WS_EX_TOPMOST.0 != 0
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

/// Another program's move out of the topmost band drops an unguarded window, such as Settings, but a guarded one and
/// the windows it owns come straight back.
#[test]
fn a_guarded_window_stays_topmost_when_another_program_drops_it() {
    let _z = Z_ORDER.lock().unwrap_or_else(PoisonError::into_inner);
    let plain = Hidden::new("KeyXtend test: plain below");
    let drops = [
        ("no topmost", HWND_NOTOPMOST),
        ("bottom", HWND_BOTTOM),
        ("after a plain window", plain.0),
    ];
    for (how, after) in drops {
        let open = Hidden::topmost("KeyXtend test: unguarded", None);
        let kept = Hidden::topmost("KeyXtend test: dropped", None);
        let tip = Hidden::tip("KeyXtend test: dropped's tip", Some(kept.0));
        guard(kept.0).unwrap();
        put_after(&open, after);
        put_after(&kept, after);
        assert!(!topmost(&open), "{how}: an unguarded window drops");
        assert!(topmost(&kept), "{how}: the guarded one stays");
        assert!(topmost(&tip), "{how}: and the window it owns");
    }
}

/// A guard run guards a shown tip too; another program's move of the tip alone out of the topmost band can take its
/// owner along, and both come straight back.
#[test]
fn a_guarded_owner_and_its_tip_stay_topmost_when_the_tip_alone_is_dropped() {
    let _z = Z_ORDER.lock().unwrap_or_else(PoisonError::into_inner);
    let plain = Hidden::new("KeyXtend test: plain below a tip");
    let drops = [
        ("no topmost", HWND_NOTOPMOST),
        ("bottom", HWND_BOTTOM),
        ("after a plain window", plain.0),
    ];
    for (how, after) in drops {
        let owner = Hidden::topmost("KeyXtend test: tip's owner", None);
        let tip = Hidden::tip("KeyXtend test: dropped tip", Some(owner.0));
        guard(owner.0).unwrap();
        guard(tip.0).unwrap();
        put_after(&tip, after);
        assert!(topmost(&tip), "{how}: the tip stays");
        assert!(topmost(&owner), "{how}: and its owner");
    }
}

/// The keyboard's shadow owns the keyboard; another program drops the keyboard, then the shadow, and both come back.
#[test]
fn a_guarded_shadow_and_its_keyboard_stay_topmost_when_both_are_dropped_in_turn() {
    let _z = Z_ORDER.lock().unwrap_or_else(PoisonError::into_inner);
    let shadow = Hidden::tip("KeyXtend test: shadow", None);
    let kb = Hidden::topmost("KeyXtend test: shadowed keyboard", Some(shadow.0));
    guard_these(listed(&[&kb, &shadow]), None, None).unwrap();
    put_after(&kb, HWND_NOTOPMOST);
    put_after(&shadow, HWND_NOTOPMOST);
    assert!(topmost(&kb), "the keyboard stays");
    assert!(topmost(&shadow), "and its shadow");
}

/// The G9 bug: while the guard lifts one window back, another window dropped inside that lift is lifted back too.
#[test]
fn a_window_dropped_inside_another_windows_lift_is_lifted_back_too() {
    let _z = Z_ORDER.lock().unwrap_or_else(PoisonError::into_inner);
    let kb = Hidden::topmost("KeyXtend test: lifted keyboard", None);
    let shadow = Hidden::tip("KeyXtend test: shadow dropped in the lift", None);
    let ours: Proc = dropper;
    // SAFETY: puts the test subclass behind the guard on the test's own window; it hands every message on.
    unsafe { SetWindowLongPtrW(kb.0, GWLP_WNDPROC, (ours as usize).cast_signed()) };
    guard(kb.0).unwrap();
    guard(shadow.0).unwrap();
    TO_DROP.set(Some(shadow.0));
    put_after(&kb, HWND_NOTOPMOST);
    assert!(
        TO_DROP.take().is_none(),
        "the shadow dropped inside the lift"
    );
    assert!(topmost(&kb), "the keyboard stays");
    assert!(topmost(&shadow), "and the shadow came back");
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
