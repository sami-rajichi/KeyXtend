//! The freeze's Windows calls that read no pixel: the bitmap it asks for, a bad size refused before any copy, the thread
//! reading physical pixels only while held, and the window list; the live grabs are in `worker/tests/snip/freeze.rs`.
#![allow(
    unsafe_code,
    reason = "The tests read DPI awareness and make their own hidden window."
)]

use super::aware::Aware;
use super::frames::{Window, frame, seen, shell, windows};
use super::grab::{grab, top_down};
use super::*;
use crate::window::THROUGH;
use kx_module_api::ports::{BASE_DPI as DPI, BGRX_BYTES, Rect};
use kx_platform::snip::{Looks, Seen};
use windows::Win32::Foundation::{E_INVALIDARG, HWND};
use windows::Win32::Graphics::Gdi::BI_RGB;
use windows::Win32::UI::HiDpi::{
    AreDpiAwarenessContextsEqual, DPI_AWARENESS_CONTEXT,
    DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2, DPI_AWARENESS_CONTEXT_UNAWARE,
    GetThreadDpiAwarenessContext,
};
use windows::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DestroyWindow, WINDOW_EX_STYLE, WS_POPUP,
};
use windows::core::w;

/// A box of a test window's on the main screen.
const AT: Rect = Rect {
    x: 10,
    y: 20,
    w: 300,
    h: 200,
};

/// Whether this thread reads in `context` now.
fn reads_in(context: DPI_AWARENESS_CONTEXT) -> bool {
    // SAFETY: plain queries of this thread's own context and a given one.
    unsafe { AreDpiAwarenessContextsEqual(GetThreadDpiAwarenessContext(), context).as_bool() }
}

/// Whether this thread reads physical pixels now.
fn physical_now() -> bool {
    reads_in(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2)
}

#[test]
fn the_bitmap_asked_for_is_top_down_with_bgrx_pixels() {
    let info = top_down(3, 2).bmiHeader;
    let top_first = (info.biWidth, info.biHeight);
    assert_eq!(
        top_first,
        (3, -2),
        "a negative height puts the top row first"
    );
    assert_eq!(usize::from(info.biBitCount), BGRX_BYTES * 8);
    assert_eq!((info.biPlanes, info.biCompression), (1, BI_RGB.0));
}

#[test]
fn a_screen_with_no_pixels_or_too_many_is_refused_before_any_copy() {
    let at = |w, h| Rect { x: 0, y: 0, w, h };
    for rect in [at(0, 5), at(-3, 5), at(i32::MAX, i32::MAX)] {
        let refused = Err(FreezeError::Refused(E_INVALIDARG.0));
        assert_eq!(grab(&[(rect, DPI)]), refused, "{rect:?}");
    }
    assert_eq!(grab(&[]), Ok(Vec::new()), "nothing asked, nothing grabbed");
}

#[test]
fn the_thread_reads_physical_pixels_only_while_held() {
    let before = physical_now();
    {
        let _held = Aware::physical().unwrap();
        assert!(physical_now());
    }
    assert_eq!(physical_now(), before, "put back as it was");
}

#[test]
fn a_freeze_is_refused_when_the_thread_cannot_read_physical_pixels() {
    let unaware = Aware::to(DPI_AWARENESS_CONTEXT_UNAWARE).unwrap();
    let refused = Aware::to(DPI_AWARENESS_CONTEXT::default());
    assert!(
        refused.is_none() && reads_in(DPI_AWARENESS_CONTEXT_UNAWARE),
        "nothing changed"
    );
    let mut ran = false;
    let freeze = held(refused, || {
        ran = true;
        Ok(())
    });
    assert_eq!((freeze, ran), (Err(NOT_PHYSICAL), false));
    assert_eq!(held(Aware::physical(), || Ok(physical_now())), Ok(true));
    drop(unaware);
}

/// A window of the test thread at `at`, of a system class, never shown; destroyed when dropped.
struct Hidden(HWND);

impl Hidden {
    fn new(at: Rect) -> Self {
        Self::styled(at, WINDOW_EX_STYLE::default())
    }

    /// One with the extended styles `ex`.
    fn styled(at: Rect, ex: WINDOW_EX_STYLE) -> Self {
        let Rect { x, y, w, h } = at;
        let (class, name) = (w!("STATIC"), w!("KeyXtend test: snip"));
        // SAFETY: a window of a system class with no parent, menu or creation data, never shown.
        let made = unsafe {
            CreateWindowExW(
                ex, class, name, WS_POPUP, x, y, w, h, None, None, None, None,
            )
        };
        Self(made.unwrap())
    }
}

impl Drop for Hidden {
    fn drop(&mut self) {
        // SAFETY: the test's own window, destroyed once on its own thread.
        let _ = unsafe { DestroyWindow(self.0) };
    }
}

#[test]
fn the_system_desktop_and_a_hidden_window_of_ours_are_never_offered() {
    let desktop = shell();
    if !desktop.is_invalid() {
        assert_eq!(seen(desktop, desktop), Seen::Desktop);
    }
    let hidden = Hidden::new(AT);
    assert_eq!(seen(hidden.0, desktop), Seen::Hidden);
}

#[test]
fn a_click_through_window_looks_it_and_a_plain_one_does_not() {
    let through = Hidden::styled(AT, WINDOW_EX_STYLE(THROUGH));
    let plain = Hidden::new(AT);
    let looks = |w: &Hidden| Window::new(w.0, shell()).through();
    assert_eq!((looks(&through), looks(&plain)), (true, false));
}

#[test]
fn a_frame_is_in_physical_pixels_even_on_an_unaware_thread() {
    let window = {
        let _px = Aware::physical().unwrap();
        Hidden::new(AT)
    };
    let _unaware = Aware::to(DPI_AWARENESS_CONTEXT_UNAWARE).unwrap();
    assert_eq!(
        frame(window.0),
        AT,
        "DWM's bounds, never scaled as the window box is"
    );
}

#[test]
fn only_a_shown_windows_frame_is_read() {
    let _ours = Hidden::new(AT);
    let listed = windows();
    assert!(!listed.is_empty(), "our hidden window is listed");
    let unread = listed.iter().filter(|&&(_, how)| how != Seen::Shown);
    assert!(unread.clone().all(|&(frame, _)| frame == Rect::default()));
}

#[test]
fn the_freeze_port_is_shared_across_threads() {
    fn shared<T: Send + Sync>() {}
    shared::<WinScreenFreeze>();
}
