//! The watcher: a hidden top-level window, since Windows broadcasts look changes only to top-level windows.
//! The callback runs on the watcher's own thread, never on the caller's.

use std::cell::{OnceCell, RefCell};
use std::panic::{AssertUnwindSafe, catch_unwind};

use kx_platform::ui::{SysLook, UiError, WatchGuard};
use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::UI::WindowsAndMessaging::{
    DefWindowProcW, WM_DWMCOLORIZATIONCOLORCHANGED, WM_SETTINGCHANGE, WM_SYSCOLORCHANGE,
    WM_THEMECHANGED,
};

use super::changed;
use crate::hidden::{Hidden, Kind};
use crate::{sysui, wide};

/// Class of the watcher window, local to our process.
const CLASS: &str = "KeyXtend.LookWatch";

/// Messages Windows sends when the mode, accent, colours or animations may have changed.
const CHANGES: [u32; 4] = [
    WM_SETTINGCHANGE,
    WM_SYSCOLORCHANGE,
    WM_THEMECHANGED,
    WM_DWMCOLORIZATIONCOLORCHANGED,
];

/// Called with each new look.
type OnChange = Box<dyn Fn(SysLook) + Send>;

thread_local! {
    /// The last look seen, on the watcher thread only.
    static LAST: RefCell<Option<SysLook>> = const { RefCell::new(None) };
    /// The callback, set once per watcher thread and never borrowed mutably, so it may run while another message comes.
    static ON_CHANGE: OnceCell<OnChange> = const { OnceCell::new() };
}

/// Starts a watcher thread that calls `on_change` there, once per real change, until the guard drops.
///
/// # Errors
/// `Refused` when Windows will not make the watcher window.
pub(crate) fn start(on_change: OnChange) -> Result<WatchGuard, UiError> {
    let setup = move || {
        LAST.set(Some(sysui::read()));
        let _ = ON_CHANGE.with(|f| f.set(on_change));
    };
    let hidden = Hidden::start(wide(CLASS), Kind::TopLevel, Some(proc), setup)?;
    Ok(WatchGuard::new(move || hidden.stop()))
}

/// Reads the look again on each change message and reports it when it really changed.
#[allow(unsafe_code, reason = "The window procedure of our watcher window.")]
unsafe extern "system" fn proc(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    if CHANGES.contains(&msg) {
        let now = sysui::read();
        if let Some(look) = LAST.with(|l| changed(&mut l.borrow_mut(), now)) {
            ON_CHANGE.with(|f| {
                if let Some(f) = f.get() {
                    // A panic must not cross into Windows, which would end the program.
                    let _ = catch_unwind(AssertUnwindSafe(|| f(look)));
                }
            });
        }
    }
    // SAFETY: default handling of a message for our own window.
    unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use std::sync::atomic::{AtomicUsize, Ordering};

    #[test]
    fn a_watch_starts_and_stops_when_its_guard_drops() {
        let calls = Arc::new(AtomicUsize::new(0));
        let count = Arc::clone(&calls);
        let guard = start(Box::new(move |_| {
            count.fetch_add(1, Ordering::SeqCst);
        }));
        let guard = guard.unwrap();
        drop(guard);
        assert_eq!(
            Arc::strong_count(&calls),
            1,
            "the callback went with its thread"
        );
    }

    #[test]
    fn two_watches_run_side_by_side() {
        let first = start(Box::new(|_| {})).unwrap();
        let second = start(Box::new(|_| {})).unwrap();
        drop(first);
        drop(second);
    }
}
