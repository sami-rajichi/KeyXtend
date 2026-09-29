//! The app in front: the window that has its keyboard, that window's layout, and its program's file name.
//! The layout of the app in front comes from `WinLayouts::foreground`, the one reader of it.

#[cfg(windows)]
pub use os::{app, app_of, focus, focus_of, thread_layout};

/// The separators in a Windows program path.
const PATH_SEPARATORS: [char; 2] = ['\\', '/'];

/// The file name at the end of program path `path`, such as `notepad.exe`; `None` when there is none.
#[must_use]
pub fn app_name(path: &str) -> Option<String> {
    let name = path.rsplit(PATH_SEPARATORS).next()?;
    (!name.is_empty()).then(|| name.to_owned())
}

#[cfg(windows)]
mod os {
    use windows::Win32::Foundation::CloseHandle;
    use windows::Win32::Foundation::HWND;
    use windows::Win32::System::Threading::{
        OpenProcess, PROCESS_NAME_WIN32, PROCESS_QUERY_LIMITED_INFORMATION,
        QueryFullProcessImageNameW,
    };
    use windows::Win32::UI::Input::KeyboardAndMouse::{GetKeyboardLayout, HKL};
    use windows::Win32::UI::WindowsAndMessaging::{
        GUITHREADINFO, GetForegroundWindow, GetGUIThreadInfo, GetWindowThreadProcessId,
    };
    use windows::core::PWSTR;

    use super::app_name;

    /// Longest program path read, in UTF-16 units: Windows' own path limit.
    const PATH_CAP: usize = 32_767;

    /// The window with the keyboard in the app in front; `None` with no app in front.
    #[must_use]
    #[allow(unsafe_code, reason = "A plain window query.")]
    pub fn focus() -> Option<HWND> {
        // SAFETY: a plain query; no window in front gives a null window, which `focus_of` refuses.
        focus_of(unsafe { GetForegroundWindow() })
    }

    /// The window that gets `window`'s keys: the focus of its input queue, which may sit on another
    /// thread, else `window` itself; `None` when the window is gone.
    #[must_use]
    #[allow(unsafe_code, reason = "Plain thread queries into a local struct.")]
    pub fn focus_of(window: HWND) -> Option<HWND> {
        // SAFETY: a plain query; a gone or null window gives thread 0.
        let thread = unsafe { GetWindowThreadProcessId(window, None) };
        if thread == 0 {
            return None;
        }
        let mut info = GUITHREADINFO {
            cbSize: u32::try_from(size_of::<GUITHREADINFO>()).ok()?,
            ..GUITHREADINFO::default()
        };
        // SAFETY: `info` is a local sized for the call, which fills it in.
        let known = unsafe { GetGUIThreadInfo(thread, &raw mut info) }.is_ok();
        let focus = info.hwndFocus;
        Some(if known && !focus.is_invalid() {
            focus
        } else {
            window
        })
    }

    /// The keyboard layout of `window`'s thread; `None` when the window is gone.
    #[must_use]
    #[allow(unsafe_code, reason = "Plain thread and layout queries.")]
    pub fn thread_layout(window: HWND) -> Option<HKL> {
        // SAFETY: a plain query; a gone window gives thread 0, refused so this thread is never read by mistake.
        let thread = unsafe { GetWindowThreadProcessId(window, None) };
        // SAFETY: a plain query of a live thread's layout.
        (thread != 0).then(|| unsafe { GetKeyboardLayout(thread) })
    }

    /// The program file name of the app in front, or `None` when Windows refuses.
    #[must_use]
    #[allow(unsafe_code, reason = "Plain window and process-id queries.")]
    pub fn app() -> Option<String> {
        let mut pid = 0u32;
        // SAFETY: plain queries; `pid` is a local that outlives the call.
        unsafe { GetWindowThreadProcessId(GetForegroundWindow(), Some(&raw mut pid)) };
        app_of(pid)
    }

    /// The program file name of process `pid`, or `None` when Windows refuses.
    #[must_use]
    #[allow(
        unsafe_code,
        reason = "Opens a process for a name query, then closes it."
    )]
    pub fn app_of(pid: u32) -> Option<String> {
        let mut buf = vec![0u16; PATH_CAP];
        let mut len = u32::try_from(buf.len()).ok()?;
        // SAFETY: asks only for the right to read the name; the handle is closed below.
        let process = unsafe { OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid) }.ok()?;
        let name = PWSTR(buf.as_mut_ptr());
        // SAFETY: `buf` holds `len` units and outlives the call, which writes back the units used.
        let read =
            unsafe { QueryFullProcessImageNameW(process, PROCESS_NAME_WIN32, name, &raw mut len) };
        // SAFETY: closes the handle opened above, once.
        let _ = unsafe { CloseHandle(process) };
        read.ok()?;
        let path = String::from_utf16(buf.get(..usize::try_from(len).ok()?)?).ok()?;
        app_name(&path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_app_name_is_the_last_part_of_the_path() {
        let path = r"C:\Program Files\Notepad++\notepad++.exe";
        assert_eq!(app_name(path).as_deref(), Some("notepad++.exe"));
        assert_eq!(app_name("C:/Tools/game.exe").as_deref(), Some("game.exe"));
        assert_eq!(app_name("code.exe").as_deref(), Some("code.exe"));
    }

    #[test]
    fn a_path_without_a_file_name_has_no_app_name() {
        for path in ["", r"C:\Tools\", "C:/"] {
            assert_eq!(app_name(path), None, "{path:?}");
        }
    }

    #[cfg(windows)]
    #[test]
    fn our_own_process_is_named_after_this_test_program() {
        let exe = std::env::current_exe().unwrap();
        let want = exe.file_name().and_then(|n| n.to_str()).map(str::to_owned);
        assert!(want.is_some());
        assert_eq!(app_of(std::process::id()), want);
    }

    #[cfg(windows)]
    #[test]
    fn no_process_has_no_name_and_the_app_in_front_is_a_bare_name() {
        assert_eq!(app_of(0), None);
        if let Some(name) = app() {
            assert!(
                !name.is_empty() && !name.contains(PATH_SEPARATORS),
                "{name}"
            );
        }
    }

    #[cfg(windows)]
    #[test]
    fn a_gone_window_has_no_focus_and_no_layout() {
        use windows::Win32::Foundation::HWND;
        assert_eq!(focus_of(HWND::default()), None);
        assert_eq!(thread_layout(HWND::default()), None);
    }

    #[cfg(windows)]
    #[test]
    fn the_window_with_the_keyboard_has_a_real_layout() {
        if let Some(window) = focus() {
            assert!(thread_layout(window).is_some_and(|l| !l.is_invalid()));
        }
    }
}
