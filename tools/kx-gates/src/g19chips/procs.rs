//! The processes running now, each with its parent and program name, from one system snapshot; which of them are
//! browsers newly started, not children of a browser already running, as a page that opened would start one; which
//! a closed terminal left running; and how many windows are new.
#![cfg(windows)]

use std::collections::HashSet;

use windows::Win32::Foundation::CloseHandle;
use windows::Win32::System::Diagnostics::ToolHelp::{
    CreateToolhelp32Snapshot, PROCESSENTRY32W, Process32FirstW, Process32NextW, TH32CS_SNAPPROCESS,
};

use crate::apps::same_name;
use crate::win::size32;

/// One process.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Proc {
    /// Its id.
    pub pid: u32,
    /// Its parent's id.
    pub parent: u32,
    /// Its program's file name.
    pub exe: String,
}

/// True when `exe` is one of `browsers`, in any letter case.
pub fn browser(exe: &str, browsers: &[String]) -> bool {
    browsers.iter().any(|b| same_name(b, exe))
}

/// The processes of `after` not in `before` whose program is a browser and whose parent is no browser: a browser
/// started anew, not a running one's helper.
pub fn launches(before: &[Proc], after: &[Proc], browsers: &[String]) -> Vec<Proc> {
    let new = |p: &&Proc| find(before, p.pid).is_none();
    let helper = |p: &Proc| find(after, p.parent).is_some_and(|q| browser(&q.exe, browsers));
    let started = |p: &&Proc| browser(&p.exe, browsers) && !helper(p);
    after.iter().filter(new).filter(started).cloned().collect()
}

/// The process `pid` in `all`.
pub fn find(all: &[Proc], pid: u32) -> Option<&Proc> {
    all.iter().find(|p| p.pid == pid)
}

/// The processes of program `exe` in `after` that were not in `before`: what a closed terminal left running.
pub fn stayed(before: &[Proc], after: &[Proc], exe: &str) -> Vec<Proc> {
    let new = |p: &&Proc| find(before, p.pid).is_none();
    let named = |p: &&Proc| same_name(&p.exe, exe);
    after.iter().filter(new).filter(named).cloned().collect()
}

/// How many of the windows `now` were not among `before`.
pub fn fresh(before: &HashSet<isize>, now: &[isize]) -> usize {
    now.iter().filter(|w| !before.contains(w)).count()
}

/// Every process running now; empty when the system gives no snapshot.
#[allow(unsafe_code, reason = "Reads the system's process list.")]
pub fn now() -> Vec<Proc> {
    let mut all = Vec::new();
    // SAFETY: a snapshot this call owns and closes; each entry is a sized struct the system fills.
    unsafe {
        let Ok(snap) = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0) else {
            return all;
        };
        let mut e = PROCESSENTRY32W {
            dwSize: size32::<PROCESSENTRY32W>(),
            ..Default::default()
        };
        let mut more = Process32FirstW(snap, &raw mut e).is_ok();
        while more {
            let end = e
                .szExeFile
                .iter()
                .position(|&c| c == 0)
                .unwrap_or(e.szExeFile.len());
            all.push(Proc {
                pid: e.th32ProcessID,
                parent: e.th32ParentProcessID,
                exe: String::from_utf16_lossy(&e.szExeFile[..end]),
            });
            more = Process32NextW(snap, &raw mut e).is_ok();
        }
        let _ = CloseHandle(snap);
    }
    all
}

#[cfg(test)]
mod tests {
    use super::*;

    fn p(pid: u32, parent: u32, exe: &str) -> Proc {
        Proc {
            pid,
            parent,
            exe: exe.to_string(),
        }
    }

    #[test]
    fn only_a_browser_started_anew_counts_as_a_launch() {
        let browsers = vec!["chrome.exe".to_string(), "msedge.exe".to_string()];
        let before = [p(4, 0, "explorer.exe"), p(10, 4, "chrome.exe")];
        let mut after = before.to_vec();
        after.push(p(11, 10, "chrome.exe"));
        after.push(p(12, 4, "notepad.exe"));
        assert!(
            launches(&before, &after, &browsers).is_empty(),
            "a helper and another app"
        );
        after.push(p(13, 4, "MSEdge.EXE"));
        assert_eq!(
            launches(&before, &after, &browsers),
            [p(13, 4, "MSEdge.EXE")]
        );
        after.push(p(14, 99, "chrome.exe"));
        assert_eq!(
            launches(&before, &after, &browsers).len(),
            2,
            "a parent already gone"
        );
        assert_eq!(
            find(&after, 12).map(|q| q.exe.as_str()),
            Some("notepad.exe")
        );
    }

    #[test]
    fn a_ping_left_after_its_terminal_closed_is_found_and_an_older_one_is_not() {
        let before = [p(4, 0, "explorer.exe"), p(20, 4, "PING.EXE")];
        let mut after = before.to_vec();
        assert!(
            stayed(&before, &after, "ping.exe").is_empty(),
            "the owner's own"
        );
        after.push(p(21, 30, "PING.EXE"));
        after.push(p(22, 30, "conhost.exe"));
        assert_eq!(stayed(&before, &after, "ping.exe"), [p(21, 30, "PING.EXE")]);
    }

    #[test]
    fn only_windows_not_there_before_count_as_new() {
        let before: HashSet<isize> = [1, 2].into_iter().collect();
        assert_eq!(fresh(&before, &[1, 2]), 0);
        assert_eq!(fresh(&before, &[2, 3, 4]), 2);
        assert_eq!(fresh(&before, &[]), 0, "a window closed");
    }
}
