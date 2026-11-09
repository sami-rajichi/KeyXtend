//! The classic console started with no standard handles. conhost takes the handles it is given as a pseudo console's
//! pipes and then opens no window of its own, so the gate gives it none.
#![cfg(windows)]

use std::ffi::OsStr;
use std::iter::repeat_n;
use std::path::{Path, PathBuf};

use windows::Win32::Foundation::CloseHandle;
use windows::Win32::System::Threading::{
    CreateProcessW, PROCESS_CREATION_FLAGS, PROCESS_INFORMATION, STARTF_USESTDHANDLES, STARTUPINFOW,
};
use windows::core::{PCWSTR, PWSTR};

use crate::proc::Process;
use crate::win::size32;

/// The quote that wraps a part of a command line.
const QUOTE: char = '"';
/// The backslash a part's quote is escaped with.
const SLASH: char = '\\';
/// What the gate puts between a command line's parts.
const SEP: &str = " ";
/// What splits a command line into parts.
const SPLITS: [char; 2] = [' ', '\t'];

/// `arg` as one part of a command line: as it is when nothing in it splits or quotes, else quoted with each quote
/// and each backslash before a quote escaped.
fn quoted(arg: &str) -> String {
    if !arg.is_empty() && !arg.contains(SPLITS) && !arg.contains(QUOTE) {
        return arg.to_string();
    }
    let mut out = String::from(QUOTE);
    let mut slashes = 0;
    for c in arg.chars() {
        if c == SLASH {
            slashes += 1;
            continue;
        }
        let n = if c == QUOTE { slashes * 2 + 1 } else { slashes };
        out.extend(repeat_n(SLASH, n));
        out.push(c);
        slashes = 0;
    }
    out.extend(repeat_n(SLASH, slashes * 2));
    out.push(QUOTE);
    out
}

/// `exe` and `args` as one Windows command line, each part quoted so programs split it back the same.
fn line(exe: &str, args: &[String]) -> String {
    let parts = std::iter::once(exe).chain(args.iter().map(String::as_str));
    parts.map(quoted).collect::<Vec<_>>().join(SEP)
}

/// What became of conhost when no window showed: its exit code, or that it still ran.
pub fn ended(code: Option<u32>) -> String {
    code.map_or_else(
        || "conhost still ran".into(),
        |c| format!("conhost ended with {c:#010X}"),
    )
}

/// Program `exe` itself when its path is full, else the first file of that name in a full-path folder of `path`.
fn found_in(exe: &str, path: &OsStr) -> Option<PathBuf> {
    if Path::new(exe).is_absolute() {
        return Some(PathBuf::from(exe));
    }
    let folders = std::env::split_paths(path).filter(|d| d.is_absolute());
    folders.map(|d| d.join(exe)).find(|f| f.is_file())
}

/// Starts `exe` with `args`, found in `PATH`'s full-path folders only, with no standard handles and none inherited.
#[allow(unsafe_code, reason = "Starts a process with no standard handles.")]
pub fn start(exe: &str, args: &[String]) -> Result<Process, String> {
    let path = std::env::var_os("PATH").unwrap_or_default();
    let program = found_in(exe, &path).ok_or_else(|| format!("{exe}: not found on PATH"))?;
    let program = program.to_string_lossy().into_owned();
    let app: Vec<u16> = program.encode_utf16().chain([0]).collect();
    let mut wide: Vec<u16> = line(&program, args).encode_utf16().chain([0]).collect();
    let si = STARTUPINFOW {
        cb: size32::<STARTUPINFOW>(),
        dwFlags: STARTF_USESTDHANDLES,
        ..Default::default()
    };
    let mut pi = PROCESS_INFORMATION::default();
    // SAFETY: `app` and `wide` are live null-ended buffers, `wide` writable, and `si` and `pi` are locals that outlive
    // the call.
    unsafe {
        CreateProcessW(
            PCWSTR(app.as_ptr()),
            Some(PWSTR(wide.as_mut_ptr())),
            None,
            None,
            false,
            PROCESS_CREATION_FLAGS(0),
            None,
            PCWSTR::null(),
            &raw const si,
            &raw mut pi,
        )
    }
    .map_err(|e| format!("{exe}: {e}"))?;
    // SAFETY: both handles are new and ours; the thread's is not needed, and the process's goes to `Process`.
    unsafe {
        let _ = CloseHandle(pi.hThread);
        Ok(Process::own(pi.hProcess))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn each_part_is_quoted_only_when_it_must_be_and_splits_back_the_same() {
        let args = ["-ForceNoHandoff", "a b", r#"c\"d"#, r"e f\", "", r"g\h"].map(String::from);
        let want = r#"conhost.exe -ForceNoHandoff "a b" "c\\\"d" "e f\\" "" g\h"#;
        assert_eq!(line("conhost.exe", &args), want);
    }

    #[test]
    fn a_bare_name_is_found_only_in_a_full_path_folder_never_the_current_one() {
        let root = std::env::var_os("SystemRoot").expect("Windows names its folder");
        let system = Path::new(&root).join("System32");
        let found = found_in("conhost.exe", system.as_os_str());
        assert_eq!(found, Some(system.join("conhost.exe")));
        assert_eq!(
            found_in("conhost.exe", OsStr::new(".;System32")),
            None,
            "relative folders"
        );
        let whole = system.join("conhost.exe");
        assert_eq!(
            found_in(&whole.to_string_lossy(), OsStr::new("")),
            Some(whole)
        );
    }

    #[test]
    fn a_console_with_no_window_tells_its_exit_code_or_that_it_still_ran() {
        assert_eq!(ended(Some(0x6D)), "conhost ended with 0x0000006D");
        assert_eq!(ended(None), "conhost still ran");
    }
}
