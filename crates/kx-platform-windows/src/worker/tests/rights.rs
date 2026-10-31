//! The worker's reduced rights, read from outside: its token is restricted, with no uiAccess and Medium integrity, it
//! holds none of our inheritable handles and has no console host, and its job caps it, bars it from shared UI and ends
//! it when asked or let go.

use super::*;
use crate::worker::access::{self, Rights};
use crate::worker::job::{ALONE, ENDED, LIMITS, MIB};
use crate::worker::label::MEDIUM;
use crate::worker::token;
use std::os::windows::io::FromRawHandle;
use std::process::Stdio;
use windows::Win32::Foundation::{CompareObjectHandles, DUPLICATE_SAME_ACCESS, DuplicateHandle};
use windows::Win32::Security::{
    SECURITY_ATTRIBUTES, TOKEN_QUERY, WinBuiltinAdministratorsSid, WinWorldSid,
};
use windows::Win32::System::Diagnostics::ToolHelp::{
    CreateToolhelp32Snapshot, PROCESSENTRY32W, Process32FirstW, Process32NextW, TH32CS_SNAPPROCESS,
};
use windows::Win32::System::JobObjects::{
    IsProcessInJob, JOB_OBJECT_UILIMIT_DESKTOP, JOB_OBJECT_UILIMIT_DISPLAYSETTINGS,
    JOB_OBJECT_UILIMIT_EXITWINDOWS, JOB_OBJECT_UILIMIT_GLOBALATOMS, JOB_OBJECT_UILIMIT_HANDLES,
    JOB_OBJECT_UILIMIT_READCLIPBOARD, JOB_OBJECT_UILIMIT_SYSTEMPARAMETERS,
    JOB_OBJECT_UILIMIT_WRITECLIPBOARD, JOBOBJECT_BASIC_UI_RESTRICTIONS,
    JOBOBJECT_EXTENDED_LIMIT_INFORMATION, JobObjectBasicUIRestrictions,
    JobObjectExtendedLimitInformation, QueryInformationJobObject,
};
use windows::Win32::System::Threading::{CreateEventW, GetCurrentProcess, GetProcessId};
use windows::core::PCWSTR;

/// An event of ours that every child started with inheritance may inherit.
#[allow(unsafe_code, reason = "Makes an unnamed event we own.")]
fn canary() -> OwnedHandle {
    let attrs = SECURITY_ATTRIBUTES {
        nLength: size32::<SECURITY_ATTRIBUTES>(),
        lpSecurityDescriptor: std::ptr::null_mut(),
        bInheritHandle: true.into(),
    };
    // SAFETY: an unnamed event with live attributes; only the owner closes it.
    unsafe {
        let made = CreateEventW(Some(&raw const attrs), true, false, PCWSTR::null()).unwrap();
        OwnedHandle::from_raw_handle(made.0)
    }
}

/// Whether `process` holds our `canary` under the same value, as an inherited handle would be.
#[allow(unsafe_code, reason = "Copies a handle value out of a child of ours.")]
fn holds(process: HANDLE, canary: &OwnedHandle) -> bool {
    let mut copy = HANDLE::default();
    // SAFETY: both process handles are open; a value the child lacks makes the copy fail.
    let copied = unsafe {
        DuplicateHandle(
            process,
            raw(canary),
            GetCurrentProcess(),
            &raw mut copy,
            0,
            false,
            DUPLICATE_SAME_ACCESS,
        )
    };
    if copied.is_err() {
        return false;
    }
    // SAFETY: Windows just made this handle for us alone.
    let copy = unsafe { OwnedHandle::from_raw_handle(copy.0) };
    // SAFETY: both handles are ours and open.
    unsafe { CompareObjectHandles(raw(&copy), raw(canary)) }.as_bool()
}

/// A second handle to `process` that outlives its owner.
#[allow(unsafe_code, reason = "Copies our own process handle.")]
fn watch(process: HANDLE) -> OwnedHandle {
    let mut copy = HANDLE::default();
    // SAFETY: our live process handle, copied within our own process.
    unsafe {
        let me = GetCurrentProcess();
        DuplicateHandle(
            me,
            process,
            me,
            &raw mut copy,
            0,
            false,
            DUPLICATE_SAME_ACCESS,
        )
        .unwrap();
        OwnedHandle::from_raw_handle(copy.0)
    }
}

/// The ids of the running processes whose parent is `process`, from a snapshot of them all.
#[allow(unsafe_code, reason = "Walks a process snapshot we own.")]
fn children(process: HANDLE) -> Vec<u32> {
    // SAFETY: our live process handle, and a new snapshot that goes straight to an owner.
    let (pid, snap) = unsafe {
        let snap = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0).unwrap();
        (GetProcessId(process), OwnedHandle::from_raw_handle(snap.0))
    };
    let mut entry = PROCESSENTRY32W {
        dwSize: size32::<PROCESSENTRY32W>(),
        ..Default::default()
    };
    let mut found = Vec::new();
    // SAFETY: our live snapshot, and a local entry whose size it states.
    let mut more = unsafe { Process32FirstW(raw(&snap), &raw mut entry) }.is_ok();
    while more {
        if entry.th32ParentProcessID == pid {
            found.push(entry.th32ProcessID);
        }
        // SAFETY: as above.
        more = unsafe { Process32NextW(raw(&snap), &raw mut entry) }.is_ok();
    }
    found
}

/// The rights the worker runs with.
const RESTRICTED: Rights = Rights {
    ui_access: false,
    level: MEDIUM,
    admin: false,
    privileged: false,
};

#[test]
fn the_worker_runs_restricted_without_ui_access_at_medium_integrity() {
    let worker = spawn();
    let rights = access::rights(raw(&worker.child().process)).unwrap();
    assert_eq!(rights, RESTRICTED);
}

#[test]
fn the_token_made_for_the_worker_reads_back_restricted() {
    let made = token::reduced().unwrap();
    assert_eq!(access::of(&made).unwrap(), RESTRICTED);
}

#[test]
fn the_worker_token_denies_administrators_alone() {
    let made = token::reduced().unwrap();
    let admins = access::can_use(&made, WinBuiltinAdministratorsSid);
    assert_eq!(admins, Ok(false));
    assert_eq!(
        access::can_use(&made, WinWorldSid),
        Ok(true),
        "Everyone stays"
    );
}

#[test]
#[allow(unsafe_code, reason = "Takes our own process's pseudo handle.")]
fn our_own_token_holds_privileges_the_worker_drops() {
    // SAFETY: the pseudo handle of our own process needs no closing.
    let ours = token::open(unsafe { GetCurrentProcess() }, TOKEN_QUERY).unwrap();
    assert!(
        access::of(&ours).unwrap().privileged,
        "so the check sees them"
    );
}

#[test]
fn the_worker_inherits_only_its_pipes() {
    let canary = canary();
    let mut plain = Command::new(worker())
        .args(media())
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .unwrap();
    let seen = holds(HANDLE(plain.as_raw_handle()), &canary);
    drop(plain.stdin.take());
    plain.wait().unwrap();
    assert!(seen, "a plain child inherits it, so the check can see it");
    let worker = spawn();
    assert!(!holds(raw(&worker.child().process), &canary));
}

#[test]
fn the_worker_starts_no_console_host() {
    let worker = spawn();
    let started = children(raw(&worker.child().process));
    assert!(
        started.is_empty(),
        "a console host would need a desktop the worker may not reach: {started:?}"
    );
}

#[test]
#[allow(unsafe_code, reason = "Reads the limits and members of our own job.")]
fn the_job_caps_the_worker_alone_in_it() {
    let worker = spawn();
    let child = worker.child();
    let mut limits = JOBOBJECT_EXTENDED_LIMIT_INFORMATION::default();
    let size = size32::<JOBOBJECT_EXTENDED_LIMIT_INFORMATION>();
    let mut inside = false.into();
    // SAFETY: our live job and process handles, and locals of the sizes passed.
    unsafe {
        let info = (&raw mut limits).cast();
        let class = JobObjectExtendedLimitInformation;
        QueryInformationJobObject(Some(child.job.handle()), class, info, size, None).unwrap();
        IsProcessInJob(
            raw(&child.process),
            Some(child.job.handle()),
            &raw mut inside,
        )
        .unwrap();
    }
    assert!(inside.as_bool());
    let basic = limits.BasicLimitInformation;
    assert_eq!(basic.LimitFlags.0 & LIMITS.0, LIMITS.0);
    assert_eq!(basic.ActiveProcessLimit, ALONE);
    let mb = usize::try_from(CFG.memory_mb).unwrap();
    assert_eq!(limits.ProcessMemoryLimit, mb * MIB);
}

#[test]
#[allow(unsafe_code, reason = "Reads the UI limits of our own job.")]
fn the_job_bars_the_worker_from_shared_ui() {
    let worker = spawn();
    let mut ui = JOBOBJECT_BASIC_UI_RESTRICTIONS::default();
    let size = size32::<JOBOBJECT_BASIC_UI_RESTRICTIONS>();
    // SAFETY: our live job handle and a local of the size passed.
    unsafe {
        let info = (&raw mut ui).cast();
        let job = Some(worker.child().job.handle());
        QueryInformationJobObject(job, JobObjectBasicUIRestrictions, info, size, None).unwrap();
    }
    let barred = [
        JOB_OBJECT_UILIMIT_HANDLES,
        JOB_OBJECT_UILIMIT_READCLIPBOARD,
        JOB_OBJECT_UILIMIT_WRITECLIPBOARD,
        JOB_OBJECT_UILIMIT_GLOBALATOMS,
        JOB_OBJECT_UILIMIT_DESKTOP,
        JOB_OBJECT_UILIMIT_DISPLAYSETTINGS,
        JOB_OBJECT_UILIMIT_SYSTEMPARAMETERS,
        JOB_OBJECT_UILIMIT_EXITWINDOWS,
    ];
    for limit in barred {
        assert!(ui.UIRestrictionsClass.contains(limit), "{:#x}", limit.0);
    }
}

#[test]
fn end_stops_a_waiting_worker_with_the_ended_code() {
    let media = media();
    let words: Vec<&str> = media.iter().map(String::as_str).collect();
    let (_input, _output, mut child) = start(&words);
    assert_eq!(child.code(Duration::ZERO), None, "it waits for its input");
    child.end();
    assert_eq!(child.code(WAIT), Some(ENDED));
}

#[test]
#[allow(unsafe_code, reason = "Waits on a copy of our own process handle.")]
fn the_job_ends_the_worker_when_its_host_lets_go() {
    let media = media();
    let words: Vec<&str> = media.iter().map(String::as_str).collect();
    let (_input, _output, child) = start(&words);
    let process = watch(raw(&child.process));
    drop(child);
    let ms = u32::try_from(WAIT.as_millis()).unwrap();
    // SAFETY: our own live copy of the handle; the wait ends by `ms`.
    let ended = unsafe { WaitForSingleObject(raw(&process), ms) };
    assert_eq!(
        ended, WAIT_OBJECT_0,
        "its input stays open, so only the job ends it"
    );
}
