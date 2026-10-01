//! Starts a program through the shell: as administrator, where this PC shows no prompt, shown or hidden and kept, or
//! shown and kept to measure. Only the shell starts a uiAccess program for a caller without uiAccess.
#![cfg(windows)]

use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;
use std::path::Path;

use windows::Win32::Foundation::HANDLE;
use windows::Win32::UI::Shell::{
    SEE_MASK_FLAG_NO_UI, SEE_MASK_NOASYNC, SEE_MASK_NOCLOSEPROCESS, SHELLEXECUTEINFOW,
    ShellExecuteExW,
};
use windows::Win32::UI::WindowsAndMessaging::{
    SHOW_WINDOW_CMD, SW_HIDE, SW_SHOWNOACTIVATE, SW_SHOWNORMAL,
};
use windows::core::{PCWSTR, w};

use crate::proc::Process;
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
pub fn command_line(args: &[String]) -> String {
    args.iter().map(|a| quote(a)).collect::<Vec<_>>().join(" ")
}

/// `s` as a null-terminated wide string.
fn wide(s: &OsStr) -> Vec<u16> {
    s.encode_wide().chain(std::iter::once(0)).collect()
}

/// Starts `exe` with `args` as administrator and returns once it has started.
pub fn run_as(exe: &Path, args: &[String]) -> Result<(), String> {
    let started = shell(exe, args, (RUN_AS, SW_SHOWNORMAL, 0)).map(drop);
    started.map_err(|e| format!("start {} as administrator: {e}", exe.display()))
}

/// Starts `exe` with `args` as administrator and no window, so it covers no key and takes no focus; its process.
#[allow(
    unsafe_code,
    reason = "Hands the shell's handle to the process that closes it."
)]
pub fn run_as_hidden(exe: &Path, args: &[String]) -> Result<Process, String> {
    let hidden = (RUN_AS, SW_HIDE, SEE_MASK_NOCLOSEPROCESS);
    let h = kept(exe, args, hidden)
        .map_err(|e| format!("start {} as administrator: {e}", exe.display()))?;
    // SAFETY: the shell gave `h` to us alone, and nothing else closes it.
    Ok(unsafe { Process::own(h) })
}

/// Starts `exe` with `args`, shown without taking the focus; its process, with the handle the shell gave.
#[allow(
    unsafe_code,
    reason = "Hands the shell's handle to the process that closes it."
)]
pub fn start_shown(exe: &Path, args: &[String]) -> Result<Process, String> {
    let shown = (
        PCWSTR::null(),
        SW_SHOWNOACTIVATE,
        SEE_MASK_FLAG_NO_UI | SEE_MASK_NOCLOSEPROCESS,
    );
    let h = kept(exe, args, shown).map_err(|e| format!("start {}: {e}", exe.display()))?;
    // SAFETY: the shell gave `h` to us alone, and nothing else closes it.
    Ok(unsafe { Process::own(h) })
}

/// Starts `exe` with `args` through the shell with `how` (verb, window state, mask) and keeps the process it gives.
fn kept(
    exe: &Path,
    args: &[String],
    how: (PCWSTR, SHOW_WINDOW_CMD, u32),
) -> Result<HANDLE, String> {
    let shown = shell(exe, args, how).map_err(|e| e.to_string())?;
    let gave = shown.filter(|h| !h.is_invalid());
    gave.ok_or_else(|| "the shell gave no process".to_string())
}

/// Starts `exe` with `args` through the shell with a verb (null is the default), a window state and extra mask bits;
/// the process handle when the mask asked for one.
#[allow(unsafe_code, reason = "The struct is set up as the call needs.")]
fn shell(
    exe: &Path,
    args: &[String],
    (verb, show, mask): (PCWSTR, SHOW_WINDOW_CMD, u32),
) -> windows::core::Result<Option<HANDLE>> {
    let file = wide(exe.as_os_str());
    let params = wide(OsStr::new(&command_line(args)));
    let mut info = SHELLEXECUTEINFOW {
        cbSize: size32::<SHELLEXECUTEINFOW>(),
        fMask: SEE_MASK_NOASYNC | mask,
        lpVerb: verb,
        lpFile: PCWSTR(file.as_ptr()),
        lpParameters: PCWSTR(params.as_ptr()),
        nShow: show.0,
        ..Default::default()
    };
    // SAFETY: `info` has `cbSize` set, and its strings live until the call returns.
    unsafe { ShellExecuteExW(&raw mut info) }?;
    Ok((mask & SEE_MASK_NOCLOSEPROCESS != 0).then_some(info.hProcess))
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
