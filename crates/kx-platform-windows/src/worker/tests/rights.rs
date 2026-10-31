//! The worker's reduced rights, read from outside: its token has no uiAccess and Medium integrity, it holds none of
//! our inheritable handles, and its job caps it, ends it on demand and ends it when the host lets go.

use super::*;
use crate::worker::job::{ALONE, ENDED, LIMITS, MIB};
use crate::worker::token::{self, MEDIUM, Rights};
use std::os::windows::io::FromRawHandle;
use std::process::Stdio;
use windows::Win32::Foundation::{CompareObjectHandles, DUPLICATE_SAME_ACCESS, DuplicateHandle};
use windows::Win32::Security::SECURITY_ATTRIBUTES;
use windows::Win32::System::JobObjects::{
    IsProcessInJob, JOBOBJECT_EXTENDED_LIMIT_INFORMATION, JobObjectExtendedLimitInformation,
    QueryInformationJobObject,
};
use windows::Win32::System::SystemServices::SECURITY_MANDATORY_HIGH_RID;
use windows::Win32::System::Threading::{CreateEventW, GetCurrentProcess};
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

#[test]
fn the_worker_runs_without_ui_access_at_medium_integrity() {
    let worker = spawn();
    let rights = token::rights(raw(&worker.child().process)).unwrap();
    assert!(!rights.ui_access);
    assert_eq!(rights.level, MEDIUM);
    assert!(rights.reduced());
    let high = SECURITY_MANDATORY_HIGH_RID.cast_unsigned();
    let kept = [(true, MEDIUM), (false, high)];
    for (ui_access, level) in kept {
        assert!(
            !Rights { ui_access, level }.reduced(),
            "{ui_access} {level}"
        );
    }
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
