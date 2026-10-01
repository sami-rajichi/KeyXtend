//! Starts a program through the shell: as administrator, where this PC shows no prompt, or hidden. Only the shell
//! starts a uiAccess program for a caller without uiAccess.
#![cfg(windows)]

use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;
use std::path::Path;

use windows::Win32::UI::Shell::{
    SEE_MASK_FLAG_NO_UI, SEE_MASK_NOASYNC, SHELLEXECUTEINFOW, ShellExecuteExW,
};
use windows::Win32::UI::WindowsAndMessaging::{SHOW_WINDOW_CMD, SW_HIDE, SW_SHOWNORMAL};
use windows::core::{PCWSTR, w};

use crate::win::size32;

/// The shell verb that asks for administrator rights.
const RUN_AS: PCWSTR = w!("runas");
/// Characters that make an argument need quotes.
const NEEDS_QUOTES: [char; 3] = [' ', '\t', '"'];

/// `arg` quoted by the usual Windows rules: backslashes before a quote are doubled.
fn quote(arg: &str) -> String {
    if !arg.is_empty() && !arg.contains(NEEDS_QUOTES) {
        return arg.to_string();
    }
    let mut out = String::from('"');
    let mut slashes = 0;
    for c in arg.chars() {
        match c {
            '\\' => slashes += 1,
            '"' => {
                out.push_str(&"\\".repeat(slashes * 2 + 1));
                slashes = 0;
            }
            _ => {
                out.push_str(&"\\".repeat(slashes));
                slashes = 0;
            }
        }
        if c != '\\' {
            out.push(c);
        }
    }
    out.push_str(&"\\".repeat(slashes * 2));
    out.push('"');
    out
}

/// `args` as one Windows command line, quoting only the ones that need it.
fn command_line(args: &[String]) -> String {
    args.iter().map(|a| quote(a)).collect::<Vec<_>>().join(" ")
}

/// `s` as a null-terminated wide string.
fn wide(s: &OsStr) -> Vec<u16> {
    s.encode_wide().chain(std::iter::once(0)).collect()
}

/// Starts `exe` with `args` as administrator and returns once it has started.
pub fn run_as(exe: &Path, args: &[String]) -> Result<(), String> {
    let normal = (RUN_AS, SW_SHOWNORMAL, false);
    shell(exe, args, normal).map_err(|e| format!("start {} as administrator: {e}", exe.display()))
}

/// Starts `exe` with `args` and no window, and returns once it has started; a failure shows no dialog.
pub fn run_hidden(exe: &Path, args: &[String]) -> Result<(), String> {
    let hidden = (PCWSTR::null(), SW_HIDE, true);
    shell(exe, args, hidden).map_err(|e| format!("start {}: {e}", exe.display()))
}

/// Starts `exe` with `args` through the shell with a verb (null is the default), a window state, and whether a
/// failure stays quiet.
#[allow(unsafe_code, reason = "The struct is set up as the call needs.")]
fn shell(
    exe: &Path,
    args: &[String],
    (verb, show, quiet): (PCWSTR, SHOW_WINDOW_CMD, bool),
) -> windows::core::Result<()> {
    let file = wide(exe.as_os_str());
    let params = wide(OsStr::new(&command_line(args)));
    let no_ui = if quiet { SEE_MASK_FLAG_NO_UI } else { 0 };
    let mut info = SHELLEXECUTEINFOW {
        cbSize: size32::<SHELLEXECUTEINFOW>(),
        fMask: SEE_MASK_NOASYNC | no_ui,
        lpVerb: verb,
        lpFile: PCWSTR(file.as_ptr()),
        lpParameters: PCWSTR(params.as_ptr()),
        nShow: show.0,
        ..Default::default()
    };
    // SAFETY: `info` has `cbSize` set, and its strings live until the call returns.
    unsafe { ShellExecuteExW(&raw mut info) }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn line(args: &[&str]) -> String {
        command_line(&args.iter().map(ToString::to_string).collect::<Vec<_>>())
    }

    #[test]
    fn command_line_quotes_only_args_that_need_it() {
        assert_eq!(line(&["-w", "new", "powershell"]), "-w new powershell");
        assert_eq!(line(&[r"C:\a b\x.ps1"]), r#""C:\a b\x.ps1""#);
        assert_eq!(line(&["", "x"]), r#""" x"#);
    }

    #[test]
    fn command_line_escapes_quotes_and_the_backslashes_before_them() {
        assert_eq!(line(&[r#"say "hi""#]), r#""say \"hi\"""#);
        assert_eq!(line(&[r"D:\a b\"]), r#""D:\a b\\""#);
        assert_eq!(line(&[r#"a\"b"#]), r#""a\\\"b""#);
        assert_eq!(line(&[r"D:\no\space"]), r"D:\no\space");
    }
}
