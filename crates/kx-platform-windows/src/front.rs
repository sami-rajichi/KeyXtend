//! The app in front: its keyboard layout and its program's file name.

#[cfg(windows)]
pub use os::{app, app_of, layout};

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
    use windows::Win32::System::Threading::{
        OpenProcess, PROCESS_NAME_WIN32, PROCESS_QUERY_LIMITED_INFORMATION,
        QueryFullProcessImageNameW,
    };
    use windows::Win32::UI::Input::KeyboardAndMouse::{GetKeyboardLayout, HKL};
    use windows::Win32::UI::WindowsAndMessaging::{GetForegroundWindow, GetWindowThreadProcessId};
    use windows::core::PWSTR;

    use super::app_name;

    /// Longest program path read, in UTF-16 units: Windows' own path limit.
    const PATH_CAP: usize = 32_767;

    /// The keyboard layout of the app in front; this thread's layout when nothing is in front.
    #[must_use]
    #[allow(unsafe_code, reason = "Plain window, thread and layout queries.")]
    pub fn layout() -> HKL {
        // SAFETY: plain queries; no window in front gives thread 0, which means this thread.
        unsafe { GetKeyboardLayout(GetWindowThreadProcessId(GetForegroundWindow(), None)) }
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
    fn the_layout_in_front_is_a_real_layout() {
        assert!(!layout().is_invalid());
    }
}
