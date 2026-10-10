//! The guard's owner chains: an outside drop brings a whole family back in order, and the guard follows only owners
//! on its own thread, as another thread would have to answer inside our window procedure.

use std::sync::{PoisonError, mpsc};

use windows::Win32::Foundation::{HWND, LPARAM, WPARAM};
use windows::Win32::System::Threading::GetCurrentThreadId;
use windows::Win32::UI::WindowsAndMessaging::{
    DispatchMessageW, GetMessageW, HWND_NOTOPMOST, MSG, PostThreadMessageW, TranslateMessage,
    WM_QUIT,
};

use super::super::guard::guard;
use super::super::top::chain;
use super::super::win::guard_these;
use super::Hidden;
use super::z_order::{Z_ORDER, above, listed, put_after, topmost};

/// Shows a guarded owner on a thread of its own, which answers messages until it is told to quit.
fn other_thread() -> (HWND, u32, std::thread::JoinHandle<()>) {
    let (tx, rx) = mpsc::channel();
    let thread = std::thread::spawn(move || {
        let owner = Hidden::tip("KeyXtend test: another thread's owner", None);
        guard(owner.0).unwrap();
        // SAFETY: a plain query of the calling thread.
        tx.send((owner.0.0.addr(), unsafe { GetCurrentThreadId() }))
            .unwrap();
        let mut msg = MSG::default();
        // SAFETY: the thread's own message loop, until WM_QUIT.
        while unsafe { GetMessageW(&raw mut msg, None, 0, 0) }.as_bool() {
            // SAFETY: hands on a message this thread just took.
            unsafe {
                let _ = TranslateMessage(&raw const msg);
                DispatchMessageW(&raw const msg);
            }
        }
    });
    let (raw, id) = rx.recv().unwrap();
    (HWND(std::ptr::without_provenance_mut(raw)), id, thread)
}

/// An outside drop of any window in the shadow's family brings the family back in order: the keyboard above its
/// shadow, and the keyboard's tip and its list, over the list's own shadow, above it.
#[test]
fn a_keyboards_tip_and_list_stay_above_it_after_an_outside_drop_in_its_family() {
    let _z = Z_ORDER.lock().unwrap_or_else(PoisonError::into_inner);
    for dropped in 0..5 {
        let shadow = Hidden::tip("KeyXtend test: family shadow", None);
        let kb = Hidden::topmost("KeyXtend test: family keyboard", Some(shadow.0));
        let tip = Hidden::tip("KeyXtend test: family tip", Some(kb.0));
        let glow = Hidden::tip("KeyXtend test: family list shadow", Some(kb.0));
        let list = Hidden::topmost("KeyXtend test: family list", Some(glow.0));
        let all = [&shadow, &kb, &tip, &glow, &list];
        guard_these(listed(&all), None, None).unwrap();
        put_after(all[dropped], HWND_NOTOPMOST);
        assert!(all.iter().all(|w| topmost(w)), "{dropped}: all stay");
        assert!(
            above(&kb, &shadow),
            "{dropped}: the keyboard above its shadow"
        );
        assert!(above(&tip, &kb), "{dropped}: its tip above it");
        assert!(
            above(&glow, &kb) && above(&list, &glow),
            "{dropped}: and its list"
        );
    }
}

#[test]
fn a_chain_follows_guarded_owners_on_this_thread_only() {
    let _z = Z_ORDER.lock().unwrap_or_else(PoisonError::into_inner);
    let near = Hidden::tip("KeyXtend test: this thread's owner", None);
    let kb = Hidden::topmost("KeyXtend test: owned here", Some(near.0));
    guard(near.0).unwrap();
    guard(kb.0).unwrap();
    assert_eq!(chain(kb.0), [kb.0, near.0], "an owner on this thread");
    let (far, id, thread) = other_thread();
    let owned = Hidden::topmost("KeyXtend test: owned across threads", Some(far));
    guard(owned.0).unwrap();
    assert_eq!(chain(owned.0), [owned.0], "never another thread's owner");
    drop(owned);
    // SAFETY: asks the test's own thread to leave its message loop.
    unsafe { PostThreadMessageW(id, WM_QUIT, WPARAM(0), LPARAM(0)) }.unwrap();
    thread.join().unwrap();
}
