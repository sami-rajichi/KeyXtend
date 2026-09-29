//! Starts a program as administrator; on this PC no prompt shows.
#![cfg(windows)]

use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;
use std::path::Path;

use windows::Win32::UI::Shell::{SEE_MASK_NOASYNC, SHELLEXECUTEINFOW, ShellExecuteExW};
use windows::Win32::UI::WindowsAndMessaging::SW_SHOWNORMAL;
use windows::core::{PCWSTR, w};

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
#[allow(unsafe_code, reason = "The struct is set up as the call needs.")]
pub fn run_as(exe: &Path, args: &[String]) -> Result<(), String> {
    let file = wide(exe.as_os_str());
    let params = wide(OsStr::new(&command_line(args)));
    let mut info = SHELLEXECUTEINFOW {
        cbSize: size_of::<SHELLEXECUTEINFOW>() as u32,
        fMask: SEE_MASK_NOASYNC,
        lpVerb: RUN_AS,
        lpFile: PCWSTR(file.as_ptr()),
        lpParameters: PCWSTR(params.as_ptr()),
        nShow: SW_SHOWNORMAL.0,
        ..Default::default()
    };
    // SAFETY: `info` has `cbSize` set, and its strings live until the call returns.
    unsafe { ShellExecuteExW(&mut info) }
        .map_err(|e| format!("start {} as administrator: {e}", exe.display()))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn line(args: &[&str]) -> String {
        command_line(&args.iter().map(|a| a.to_string()).collect::<Vec<_>>())
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
