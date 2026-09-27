//! The language key: asks the app in front to switch to the next installed layout.

use windows::Win32::Foundation::{HWND, LPARAM, WPARAM};
use windows::Win32::UI::Input::KeyboardAndMouse::GetKeyboardLayout;
use windows::Win32::UI::WindowsAndMessaging::{
    GUITHREADINFO, GetGUIThreadInfo, GetWindowThreadProcessId, PostMessageW,
    WM_INPUTLANGCHANGEREQUEST,
};

use crate::layout::installed_layouts;

/// The layout after `cur` in `list`, wrapping and skipping copies of `cur`; `None` when there is none.
pub fn next_layout(list: &[isize], cur: isize) -> Option<isize> {
    let start = list.iter().position(|&l| l == cur).map_or(0, |i| i + 1);
    (0..list.len())
        .map(|k| list[(start + k) % list.len()])
        .find(|&l| l != cur)
}

/// The window that gets `hwnd`'s keys: the focus of its input queue, which may sit on another thread.
fn focus_of(hwnd: HWND) -> Option<HWND> {
    let mut info = GUITHREADINFO {
        cbSize: size_of::<GUITHREADINFO>() as u32,
        ..Default::default()
    };
    // SAFETY: plain queries; `info` is sized for the call.
    unsafe {
        let tid = GetWindowThreadProcessId(hwnd, None);
        if tid == 0 {
            return None;
        }
        let known = GetGUIThreadInfo(tid, &mut info).is_ok() && !info.hwndFocus.is_invalid();
        Some(if known { info.hwndFocus } else { hwnd })
    }
}

/// The layout of the thread that holds `hwnd`'s keyboard focus; `None` when the window is gone.
pub fn layout_of(hwnd: HWND) -> Option<isize> {
    let focus = focus_of(hwnd)?;
    // SAFETY: plain queries; thread 0 is refused, so we never read our own layout by mistake.
    unsafe {
        let tid = GetWindowThreadProcessId(focus, None);
        (tid != 0).then(|| GetKeyboardLayout(tid).0 as isize)
    }
}

/// Asks `hwnd` to switch to the next installed layout; returns the layout asked for.
pub fn ask_next(hwnd: HWND) -> Result<isize, String> {
    let list: Vec<isize> = installed_layouts().iter().map(|h| h.0 as isize).collect();
    let cur = layout_of(hwnd).ok_or("the window is gone")?;
    let next = next_layout(&list, cur).ok_or("only one layout is installed")?;
    ask(hwnd, next)?;
    Ok(next)
}

/// Asks `hwnd`'s focus window to switch to `layout`, the way the system language hotkey does.
pub fn ask(hwnd: HWND, layout: isize) -> Result<(), String> {
    let focus = focus_of(hwnd).ok_or("the window is gone")?;
    // SAFETY: a plain post; the window checks and applies the request itself.
    unsafe {
        PostMessageW(
            Some(focus),
            WM_INPUTLANGCHANGEREQUEST,
            WPARAM(0),
            LPARAM(layout),
        )
    }
    .map_err(|e| format!("layout request: {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    const EN: isize = 0x0409_0409;
    const FR: isize = 0x040C_040C;
    const AR: isize = 0x1C01_1C01;

    #[test]
    fn next_layout_follows_the_list_and_wraps_to_the_first() {
        let list = [EN, FR, AR];
        assert_eq!(next_layout(&list, EN), Some(FR));
        assert_eq!(next_layout(&list, FR), Some(AR));
        assert_eq!(next_layout(&list, AR), Some(EN));
    }

    #[test]
    fn an_unknown_current_layout_goes_to_the_first() {
        assert_eq!(next_layout(&[EN, FR], AR), Some(EN));
    }

    #[test]
    fn a_duplicate_of_the_current_layout_is_skipped() {
        assert_eq!(next_layout(&[EN, EN, FR], EN), Some(FR));
        assert_eq!(next_layout(&[EN, EN], EN), None);
    }

    #[test]
    fn a_window_that_is_gone_has_no_layout() {
        assert_eq!(layout_of(HWND::default()), None);
    }

    #[test]
    fn one_layout_or_none_cannot_switch() {
        assert_eq!(next_layout(&[EN], EN), None);
        assert_eq!(next_layout(&[], EN), None);
    }
}
