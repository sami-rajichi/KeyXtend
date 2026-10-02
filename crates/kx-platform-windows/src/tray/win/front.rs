//! The front-window hook: tells the icon when the user brings a window outside the taskbar to the front.

use std::cell::Cell;

use kx_platform::ui::TrayEvent;
use windows::Win32::Foundation::HWND;
use windows::Win32::UI::Accessibility::{HWINEVENTHOOK, SetWinEventHook, UnhookWinEvent};
use windows::Win32::UI::WindowsAndMessaging::{
    EVENT_SYSTEM_FOREGROUND, GetClassNameW, WINEVENT_OUTOFCONTEXT, WINEVENT_SKIPOWNPROCESS,
};

use super::HERE;
use crate::tray::elsewhere;

/// Longest class name Windows keeps, in UTF-16 units.
const CLASS_CAP: usize = 256;

thread_local! {
    /// The hook, on the icon window's thread only.
    static HOOK: Cell<HWINEVENTHOOK> = const { Cell::new(HWINEVENTHOOK(std::ptr::null_mut())) };
}

/// Watches which window comes to the front, from any program but ours, on this thread's message loop. When Windows
/// refuses the hook, no `Away` comes, and the menu closes on a pick, another click or a press on the keyboard.
#[allow(unsafe_code, reason = "Sets a front-window hook.")]
pub(super) fn hook() {
    let flags = WINEVENT_OUTOFCONTEXT | WINEVENT_SKIPOWNPROCESS;
    let only = EVENT_SYSTEM_FOREGROUND;
    // SAFETY: an out-of-context hook, so `front` runs on this thread's message loop, never inside another program.
    let hook = unsafe { SetWinEventHook(only, only, None, Some(front), 0, 0, flags) };
    unhook();
    HOOK.set(hook);
}

/// Removes this thread's hook, if it has one.
#[allow(unsafe_code, reason = "Removes our front-window hook.")]
pub(super) fn unhook() {
    let hook = HOOK.replace(HWINEVENTHOOK::default());
    if !hook.is_invalid() {
        // SAFETY: a hook this thread set, removed once.
        let _ = unsafe { UnhookWinEvent(hook) };
    }
}

/// Tells the icon when a window outside the taskbar comes to the front.
#[allow(unsafe_code, reason = "The callback of our front-window hook.")]
unsafe extern "system" fn front(
    _hook: HWINEVENTHOOK,
    _event: u32,
    hwnd: HWND,
    _object: i32,
    _child: i32,
    _thread: u32,
    _time: u32,
) {
    if let Some(e) = away(hwnd) {
        HERE.with(|h| h.get().map(|here| here.tell(e)));
    }
}

/// `Away` when `hwnd` is a window outside the taskbar; `None` for the taskbar or a window already gone.
#[allow(unsafe_code, reason = "Reads a window's class name.")]
pub(super) fn away(hwnd: HWND) -> Option<TrayEvent> {
    if hwnd.is_invalid() {
        return None;
    }
    let mut name = [0u16; CLASS_CAP];
    // SAFETY: fills a local buffer of its given length; a window gone gives 0.
    let len = unsafe { GetClassNameW(hwnd, &mut name) };
    elsewhere(&class_text(&name, len)?).then_some(TrayEvent::Away)
}

/// The class name in `name`, `len` units long as Windows counted it; none for 0 or a count past the buffer.
fn class_text(name: &[u16], len: i32) -> Option<String> {
    let len = usize::try_from(len).ok().filter(|n| *n > 0)?;
    name.get(..len).map(String::from_utf16_lossy)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_class_count_past_the_buffer_or_zero_reads_nothing() {
        let name = [u16::from(b'A'), u16::from(b'B')];
        assert_eq!(class_text(&name, 2).as_deref(), Some("AB"));
        assert_eq!(class_text(&name, 3), None, "never a panic in the callback");
        assert_eq!(class_text(&name, 0), None);
        assert_eq!(class_text(&name, -1), None);
    }
}
